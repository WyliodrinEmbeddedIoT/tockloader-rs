use async_trait::async_trait;

use crate::attributes::system_attributes::SystemAttributes;
use crate::board_settings::BoardSettings;
use crate::bootloader_serial::{issue_command, Command, Response};
use crate::connection::Connection;
use crate::connection::SerialConnection;
use crate::errors::InternalError;
use crate::errors::TockloaderError;
use crate::tabs::tab::Tab;
use crate::CommandInstall;
use tbf_parser::parse::parse_tbf_header_lengths;

#[async_trait]
impl CommandInstall for SerialConnection {
    async fn install_app(
        &mut self,
        settings: &BoardSettings,
        tab_file: Tab,
    ) -> Result<(), TockloaderError> {
        if !self.is_open() {
            return Err(TockloaderError::Internal(InternalError::ConnectionNotOpen));
        }

        let stream = self.stream.as_mut().expect(
            "Expected serial stream to be initialized. This should not happen if setup is correct.",
        );

        let system_attributes = SystemAttributes::read_system_attributes_serial(stream).await?;

        let board = system_attributes
            .board
            .ok_or("No board name found.".to_owned())
            .map_err(|e| TockloaderError::Internal(InternalError::MisconfiguredBoardSettings(e)))?;
        //TODO: handle the case when board is not set
        let kernel_version = system_attributes
            .kernel_version
            .ok_or("No kernel version found.".to_owned())
            .map_err(|e| TockloaderError::Internal(InternalError::MisconfiguredBoardSettings(e)))?;

        if tab_file.is_compatible_with_board(&board) {
            log::info!("Specified tab is compatible with board.");
        } else {
            //TODO: replace with appropriate error
            panic!("Specified tab is not compatible with board.");
        }
        if tab_file.is_compatible_with_kernel_verison(kernel_version as u32) {
            log::info!("Specified tab is compatible with your kernel version.");
        } else {
            log::info!("Specified tab is not compatible with your kernel version.");
        }

        let mut address = match system_attributes.appaddr {
            Some(addr) => {
                log::info!("App start address found in system attributes.");
                addr
            }
            None => {
                log::info!(
                    "No start address found in system attributes. Falling back to board settings."
                );
                settings.start_address
            }
        };

        loop {
            // Read a block of 200 8-bit words
            let mut pkt = (address as u32).to_le_bytes().to_vec();
            let length = (200_u16).to_le_bytes().to_vec();
            pkt.extend(length);

            let (_, message) = issue_command(
                stream,
                Command::ReadRange,
                pkt,
                true,
                200,
                Response::ReadRange,
            )
            .await?;

            // TODO: handle error properly

            // let (ver, header_len, whole_len) = parse_tbf_header_lengths(
            //     &message[0..8]
            //         .try_into()
            //         .expect("Buffer length must be at least 8 bytes long."),
            //     //TODO: select an appropriate error
            // )
            // .unwrap();

            // if header_len == 0 {
            //     break; // No more apps
            // }

            let (_ver, _header_len, whole_len) = match parse_tbf_header_lengths(
                &message[0..8]
                    .try_into()
                    .expect("Buffer length must be at least 8 bytes long."),
            ) {
                Ok((ver, header_len, whole_len)) if header_len != 0 => (ver, header_len, whole_len),
                _ => break, // No more apps
            };

            address += whole_len as u64;
        }

        let arch = system_attributes
            .arch
            .ok_or("No architecture found.".to_owned())
            .map_err(|e| TockloaderError::Internal(InternalError::MisconfiguredBoardSettings(e)))?;

        let mut binary = tab_file.extract_binary(&arch.clone())?;

        let size = binary.len() as u64;

        let multiple = address / size;

        let (mut new_address, _gap_size) = if multiple * size != address {
            let new_address = ((address + size) / size) * size;
            let gap_size = new_address - address;
            (new_address, gap_size)
        } else {
            (address, 0)
        };

        // Make sure the binary is a multiple of the page size by padding 0xFFs
        // TODO(Micu Ana): check if the page-size differs
        let page_size = 512;

        let remaining = page_size - (binary.len() % page_size);
        let padding = vec![0xFF; remaining % page_size];
        binary.extend(padding);

        let binary_len = binary.len();

        // Get indices of pages that have valid data to write
        let mut valid_pages: Vec<u8> = Vec::new();

        valid_pages.extend(
            (0..(binary_len / page_size))
                .filter(|&i| {
                    binary[(i * page_size)..((i + 1) * page_size)]
                        .iter()
                        .any(|&b| b != 0)
                })
                .map(|i| i as u8),
        );

        // If there are no pages valid, all pages would have been removed, so we write them all
        // Fallback that ensures old data is cleared and that aren't any partially written apps
        if valid_pages.is_empty() {
            valid_pages.extend((0..(binary_len / page_size)).map(|i| i as u8));
        }

        // Include a blank page (if exists) after the end of a valid page. There might be a usable 0 on the next page
        let existing_pages = valid_pages.clone();

        let ending_pages = existing_pages.iter().map(|&i| i + 1).filter(|&next| {
            next < (binary_len / page_size) as u8 && !existing_pages.contains(&next)
        });

        valid_pages.extend(ending_pages);

        for i in valid_pages {
            // Create the packet that we send to the bootloader
            // First four bytes are the address of the page
            let mut pkt = (new_address as u32 + (i as usize * page_size) as u32)
                .to_le_bytes()
                .to_vec();
            // Then the bytes that go into the page
            pkt.extend(&binary[(i as usize * page_size)..((i + 1) as usize * page_size)]);

            // Write to bootloader
            let (_, _) =
                issue_command(stream, Command::WritePage, pkt, true, 0, Response::OK).await?;
        }

        new_address += binary.len() as u64;

        let pkt = (new_address as u32).to_le_bytes().to_vec();

        // Empty page marks the end of the apps list
        let _ = issue_command(stream, Command::ErasePage, pkt, true, 0, Response::OK).await?;

        Ok(())
    }
}

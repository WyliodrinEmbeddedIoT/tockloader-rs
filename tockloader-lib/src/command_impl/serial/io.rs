use async_trait::async_trait;

use crate::{
    attributes::{app_attributes::AppAttributes, system_attributes::SystemAttributes},
    board_settings::BoardSettings,
    bootloader_serial::{issue_command, ping_bootloader_and_wait_for_response, Command, Response},
    command_impl::reshuffle_apps::PAGE_SIZE,
    connection::{Connection, SerialConnection},
    errors::{InternalError, TockloaderError},
    IOCommands,
};

#[async_trait]
impl IOCommands for SerialConnection {
    async fn read(&mut self, address: u64, size: usize) -> Result<Vec<u8>, TockloaderError> {
        let mut pkt = (address as u32).to_le_bytes().to_vec();
        let length = (size as u16).to_le_bytes().to_vec();
        for i in length {
            pkt.push(i);
        }
        let stream = self.stream.as_mut().expect("Board must be open");

        let (_, appdata) = issue_command(
            stream,
            Command::ReadRange,
            pkt,
            true,
            size,
            Response::ReadRange,
        )
        .await?;

        // this might be very specific, but when I was testing, i got 8191 bytes instead of 8192
        // this is because of the consecutive ESCAPE_CHAR rule inside issue_command function

        if appdata.len() < size {
            panic!("Appdata was not read correctly?? We should not reach here");
        }
        Ok(appdata)
    }

    async fn write(&mut self, address: u64, pkt: Vec<u8>) -> Result<(), TockloaderError> {
        let stream = self.stream.as_mut().expect("Board must be open");
        let mut binary = pkt.clone();
        while !binary.len().is_multiple_of(PAGE_SIZE as usize) {
            binary.push(0u8);
        }

        let mut page_number = 0;
        while (page_number + 1) * PAGE_SIZE <= binary.len() as u32 {
            let mut pkt = (address as u32 + page_number * PAGE_SIZE)
                .to_le_bytes()
                .to_vec();
            pkt.append(
                &mut binary
                    [(page_number * PAGE_SIZE) as usize..((page_number + 1) * PAGE_SIZE) as usize]
                    .to_vec(),
            );
            let _ = issue_command(stream, Command::WritePage, pkt, true, 0, Response::OK).await?;
            page_number += 1;
        }

        let pkt = (address as u32 + binary.len() as u32)
            .to_le_bytes()
            .to_vec();

        let _ = issue_command(stream, Command::ErasePage, pkt, true, 0, Response::OK).await?;
        Ok(())
    }

    async fn list_apps(
        &mut self,
        settings: &BoardSettings,
    ) -> Result<Vec<AppAttributes>, TockloaderError> {
        if !self.is_open() {
            return Err(InternalError::ConnectionNotOpen.into());
        }
        let stream = self.stream.as_mut().expect("Board must be open");

        ping_bootloader_and_wait_for_response(stream).await?;

        AppAttributes::read_apps_data_serial(stream, settings.start_address).await
    }

    async fn read_system_attributes(&mut self) -> Result<SystemAttributes, TockloaderError> {
        if !self.is_open() {
            return Err(InternalError::ConnectionNotOpen.into());
        }
        let stream = self.stream.as_mut().expect("Board must be open");

        ping_bootloader_and_wait_for_response(stream).await?;

        let system_attributes = SystemAttributes::read_system_attributes_serial(stream).await?;
        Ok(system_attributes)
    }
}

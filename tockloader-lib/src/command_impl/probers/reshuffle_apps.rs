use async_trait::async_trait;
use itertools::Itertools;
use probe_rs::flashing::DownloadOptions;
use probe_rs::MemoryInterface;
use tbf_parser::parse::{parse_tbf_footer, parse_tbf_header, parse_tbf_header_lengths};

use crate::attributes::app_attributes::{AppAttributes, TbfFooter};
use crate::board_settings::BoardSettings;
use crate::connection::{Connection, ProbeRSConnection};
use crate::errors::{InternalError, TockloaderError};
use crate::tabs::tab::Tab;
use crate::UtilityReshuffleApps;

const ALIGNMENT: u64 = 1024;
const PAGE_SIZE: u32 = 512;

#[async_trait]
impl UtilityReshuffleApps for ProbeRSConnection {
    async fn reshuffle_apps(
        &mut self,
        settings: &BoardSettings,
        tab: Option<Tab>,
    ) -> Result<(), TockloaderError> {
        if !self.is_open() {
            return Err(InternalError::ConnectionNotOpen.into());
        }

        let session = self.session.as_mut().expect("Board must be open");


        // -- 1. GET EXISTING APPS -- //

        // get the already installed apps
        let mut installed_apps: Vec<AppAttributes>;
        {
            let mut core = session.core(self.target_info.core)?;
            installed_apps =
                AppAttributes::read_apps_data_probe(&mut core, settings.start_address).unwrap();
        }

        // -- 2. GET BINARIES -- //

        // if a tab is provided, reconstruct the app and add it to the list
        if let Some(mut app) = reconstruct_app(&tab, settings) {
            app.index = installed_apps.len() as u8;
            installed_apps.push(app.clone());
        }

        // save the binaries separately
        let mut app_binaries: Vec<Vec<u8>> = Vec::new();

        for app in installed_apps.iter() {
            match app.installed {
                true => {
                    let mut core = session.core(self.target_info.core)?;
                    let mut appdata = vec![0u8; app.size as usize];
                    core.read(app.address, &mut appdata)?;
                    app_binaries.push(appdata);
                }
                false => {
                    app_binaries.push(
                        tab.as_ref()
                            .unwrap()
                            .extract_binary(settings.arch.as_ref().unwrap().as_str())
                            .unwrap(),
                    );
                }
            }
        }



        // -- 3. REORDER -- //

        // separate the apps
        let mut rust_apps: Vec<AppAttributes> = Vec::new();
        let mut c_apps: Vec<AppAttributes> = Vec::new();
        for app in installed_apps.iter() {
            match app.tbf_header.get_fixed_address_flash() {
                Some(_) => {
                    rust_apps.push(app.clone());
                }
                None => {
                    c_apps.push(app.clone());
                }
            }
        }

        // this is necessary. If a rust app is already installed, for example: at 0x48000
        // and we want to install another one at 0x40000, reorder them first
        rust_apps.sort_by_key(|app| app.address);

        // make permutations only for the c apps, as their order can be changed
        let mut permutations = (0..c_apps.len()).permutations(c_apps.len());

        let mut min_padding = usize::MAX;
        let mut saved_configuration: Vec<AppAttributes> = Vec::new();

        for _ in 0..100_000 {
            // use just 100k permutations, or else we'll be here for a while
            match permutations.next() {
                Some(order) => {
                    let mut total_padding: usize = 0;
                    let mut permutation_index: usize = 0;
                    let mut rust_index: usize = 0;
                    let mut reordered_apps: Vec<AppAttributes> = Vec::new();
                    loop {
                        let insert_c: bool; // every iteration will insert an app, or break if there are none left

                        // start either where the last app ends, or at start address if there are no apps
                        let address = match reordered_apps.last() {
                            Some(app) => app.address + app.size as u64,
                            None => settings.start_address,
                        };

                        match order.get(permutation_index) {
                            Some(_) => {
                                // we have a C app
                                match rust_apps.get(rust_index) {
                                    Some(_) => {
                                        // we also have a rust app, insert only if it fits
                                        insert_c = c_apps[order[permutation_index]].size
                                            <= (rust_apps[rust_index].address - address) as u32;
                                    }
                                    None => {
                                        // we have only a C app, insert it accordingly
                                        insert_c = true;
                                    }
                                }
                            }
                            None => {
                                // we don't have a c app
                                match rust_apps.get(rust_index) {
                                    Some(_) => {
                                        // we have a rust app, insert it
                                        insert_c = false;
                                    }
                                    None => {
                                        // we don't have any app, break?
                                        break;
                                    }
                                }
                            }
                        }

                        let mut start_address: u64;
                        if reordered_apps.is_empty() {
                            // is padding needed when starting from settings.start_address?
                            start_address = settings.start_address;
                        } else {
                            // start the padding where last app ends
                            let last_app = reordered_apps.last().unwrap();
                            start_address = last_app.address + last_app.size as u64;
                        }
                        let needed_padding: u32 = if insert_c {
                            if !start_address.is_multiple_of(PAGE_SIZE as u64) {
                                PAGE_SIZE - start_address as u32 % PAGE_SIZE // c app needs to be inserted at a multiple of page_size
                            } else {
                                0
                            }
                        } else {
                            if rust_apps[rust_index].address < start_address {
                                // the program wants to insert a rust app where another rust app already exists
                                panic!("Can't insert the rust app, space is already occupied by another rust app");
                                // we can't change the start address, so panic
                            }
                            (rust_apps[rust_index].address - start_address) as u32
                            // rust app needs to be inserted at a fixed address, pad until there
                        };
                        if needed_padding > 0 {
                            // insert a padding
                            total_padding += needed_padding as usize;
                            reordered_apps.push(installed_apps[0].clone());
                            reordered_apps.last_mut().unwrap().address = start_address;
                            reordered_apps.last_mut().unwrap().size = needed_padding;
                            reordered_apps.last_mut().unwrap().is_padding = true;
                            start_address += needed_padding as u64;
                        }
                        if insert_c {
                            // insert the c app, also change its address
                            reordered_apps.push(c_apps[order[permutation_index]].clone());
                            reordered_apps.last_mut().unwrap().address = start_address;
                            permutation_index += 1;
                        } else {
                            // insert the rust app, don't change its address because it is fixed
                            reordered_apps.push(rust_apps[rust_index].clone());
                            rust_index += 1;
                        }
                    }

                    // (adi): this block can be used in the future for testing

                    // log::info!("PERMUTATION");
                    // log::info!("PERMUTATION");
                    // log::info!("PERMUTATION");
                    // log::info!("PERMUTATION");
                    // log::info!("PERMUTATION");
                    // log::info!("PERMUTATION");
                    // let mut correct: bool = true;
                    // for (index, item) in reordered_apps.iter().enumerate() {
                    //     // log::info!("index {index} len {}", reordered_apps.len());
                    //     if index < reordered_apps.len() - 1 {
                    //         if item.address + item.size as u64 > reordered_apps[index + 1].address {
                    //             log::info!("Checking item address {:#x} and size {} and computed address {:#x} with next address {:#x}", item.address, item.size, item.address + item.size as u64, reordered_apps[index + 1].address);
                    //             log::info!("WRONG");
                    //             correct = false;
                    //         }
                    //     }
                    // log::info!(
                    //     "app name {}, start address {:#x}, size {}, rustapp {}, end address {:#x}",
                    //     item.tbf_header.get_package_name().unwrap_or(""),
                    //     item.address,
                    //     item.size,
                    //     match item.tbf_header.get_fixed_address_flash() {
                    //         Some(_) => {
                    //             true
                    //         }
                    //         None => {
                    //             false
                    //         }
                    //     },
                    //     item.address + item.size as u64,
                    // );
                    // }
                    // panic!();
                    // if correct == false {
                    //     panic!();
                    // }

                    // find the configuration that uses the minimum padding
                    if total_padding < min_padding {
                        min_padding = total_padding;
                        saved_configuration = reordered_apps.clone();
                    }
                }
                None => break,
            }
        }

        log::debug!("min padding is {min_padding}");
        // panic!();
        let mut index = 0;
        for item in saved_configuration.iter() {
            if item.is_padding {
                log::debug!(
                    "-----PADDING APP------, start:{:#x}, size {}, end {:#x}",
                    item.address,
                    item.size,
                    item.address + item.size as u64
                );
            } else {
                log::debug!(
                    "{}. {}, start address {:#x}, size {}, rustapp {}, end address {:#x}",
                    index,
                    item.tbf_header.get_package_name().unwrap_or(""),
                    item.address,
                    item.size,
                    match item.tbf_header.get_fixed_address_flash() {
                        Some(_) => {
                            true
                        }
                        None => {
                            false
                        }
                    },
                    item.address + item.size as u64,
                );
                index += 1;
            }
        }

        // -- 4. WRITE -- //


        let mut pkt: Vec<u8> = Vec::new();
        for item in saved_configuration.iter() {
            // i have to find a better way of determining if i used a padding or not, this is not okay
            if item.is_padding {
                // write padding binary
                let mut buf = create_padding(item.size);
                pkt.append(&mut buf);
            } else {
                pkt.append(&mut app_binaries[item.index as usize]);
            }
        }
        let mut loader = session.target().flash_loader();

        loader.add_data(settings.start_address, &pkt)?;

        let mut options = DownloadOptions::default();
        options.keep_unwritten_bytes = true;

        // Finally, the data can be programmed
        // TODO(george-cosma): Can we move this outside the loop? Commit once?
        // (adi): yes, it worked
        loader.commit(session, options)?;
        Ok(())
    }
}

// this function takes a tab and turns it into an AppAttributes instance
fn reconstruct_app(tab: &Option<Tab>, settings: &BoardSettings) -> Option<AppAttributes> {
    if !tab.is_none() {
        let arch = settings
            .arch
            .as_ref()
            .ok_or(InternalError::MisconfiguredBoardSettings(
                "architechture".to_owned(),
            ))
            .unwrap();

        // extract the binary
        let binary = tab
            .as_ref()
            .unwrap()
            .extract_binary(arch)
            .expect("invalid arch");

        // extract relevant data from the header
        let (tbf_version, header_len, total_size) = match parse_tbf_header_lengths(
            &binary[0..8]
                .try_into()
                .expect("Buffer length must be at least 8 bytes long."),
        ) {
            Ok((tbf_version, header_len, total_size)) if header_len != 0 => {
                (tbf_version, header_len, total_size)
            }
            _ => return None,
        };

        // turn the buffer slice into a TbfHeader instance
        let header =
            parse_tbf_header(&binary[0..header_len as usize], tbf_version).expect("invalid header");
        let binary_end_offset = header.get_binary_end();

        // obtain the footers
        let mut footers: Vec<TbfFooter> = vec![];
        let mut footer_offset = binary_end_offset;
        let mut footer_number = 0;

        while footer_offset < total_size {
            let mut appfooter: Vec<u8> = binary[footer_offset as usize..].to_vec();
            let byte1 = *appfooter.get(2).unwrap();
            let byte2 = *appfooter.get(3).unwrap();
            let expected_size = u16::from_le_bytes([byte1, byte2]) + 4;
            // insert or remove until we have expected size?
            // insert
            while appfooter.len() < expected_size as usize {
                appfooter.push(0x0u8);
            }
            // delete
            while appfooter.len() > expected_size as usize {
                appfooter.pop();
            }
            // (these might be useless)

            let footer_info = parse_tbf_footer(&appfooter).expect("Invalid footer!");
            footers.insert(footer_number, TbfFooter::new(footer_info.0, footer_info.1));
            footer_number += 1;
            footer_offset += footer_info.1 + 4;
        }

        // create an AppAttribute using the data we obtained
        return Some(AppAttributes::new(
            if let Some(addr) = header.get_fixed_address_flash() {
                if addr < settings.start_address as u32 {
                    // this rust app should not be here
                    panic!("This rust app starts at {addr:#x}, while the board's start_address is {:#x}", settings.start_address)
                }
                // turns out that fixed address is a loosely-used term, address has to be aligned down to a multiple of 1024 bytes
                align_down(addr as u64)
            } else {
                settings.start_address
            },
            total_size,
            0,
            header,
            footers,
            false,
        ));
    }
    None
}

// this return only the binary for a padding
fn create_padding(size: u32) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(&u16::to_le_bytes(2u16)); // tbf version 2
    buf.extend_from_slice(&u16::to_le_bytes(16u16)); // header size is 16
    buf.extend_from_slice(&u32::to_le_bytes(size)); // total_size is size
    let mut checksum = 0;
    for chunk in buf.chunks_exact(4) {
        let word = u32::from_le_bytes(chunk.try_into().unwrap());
        checksum ^= word;
    }
    buf.extend_from_slice(&u32::to_le_bytes(checksum));
    while buf.len() < size as usize {
        buf.push(0x0u8);
    }
    buf
}

// this function takes a rust app's fixed address and aligns it down to ALIGNMENT (1024 currently)
fn align_down(address: u64) -> u64 {
    address - address % ALIGNMENT
}

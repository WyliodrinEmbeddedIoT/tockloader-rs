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
use crate::CommandReshuffleApps;

#[async_trait]
impl CommandReshuffleApps for ProbeRSConnection {
    async fn reshuffle_apps(
        &mut self,
        settings: &BoardSettings,
        tab: Option<Tab>,
    ) -> Result<(), TockloaderError> {
        if !self.is_open() {
            return Err(InternalError::ConnectionNotOpen.into());
        }
        let session = self.session.as_mut().expect("Board must be open");
        let mut installed_apps: Vec<AppAttributes>;
        {
            let mut core = session.core(self.target_info.core)?;
            installed_apps =
                AppAttributes::read_apps_data_probe(&mut core, settings.start_address).unwrap();
        }
        let mut app_binaries: Vec<Vec<u8>> = Vec::new();
        if let Some(mut app) = reconstruct_app(&tab, settings) {
            //  log::info!("new app {app:?}");
            app.index = installed_apps.len() as u8;
            installed_apps.push(app.clone());
            //  log::info!("pushed??");
        }
        // panic!();
        // log::info!("new app {:?}", new_app.unwrap());
        // panic!();

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
        // for app in installed_apps.iter() {
        //     log::info!(
        //         "{}. {}, start: {:#x}, size: {}",
        //         app.index,
        //         app.tbf_header.get_package_name().unwrap_or(""),
        //         app.address,
        //         app.size
        //     )
        // }
        // panic!();
        let mut rust_apps: Vec<AppAttributes> = Vec::new();
        let mut c_apps: Vec<AppAttributes> = Vec::new();
        // log::info!("installed apps {:?}", installed_apps);
        // log::info!("{:#x}", settings.start_address);
        // panic!();
        for app in installed_apps.iter() {
            // remove this mut
            // app.size -= 100;
            match app.tbf_header.get_fixed_address_flash() {
                Some(_) => {
                    // app.address += 100;
                    // app.size += 1000;
                    // log::info!(
                    //     "app has fixed addr {:#x}, adding {} to rust_apps",
                    //     app.address,
                    //     app.tbf_header.get_package_name().unwrap_or("")
                    // );
                    rust_apps.push(app.clone());
                }
                None => {
                    // log::info!(
                    //     "mutable, adding {} to C apps",
                    //     app.tbf_header.get_package_name().unwrap_or("")
                    // );
                    // app.size = 99999;
                    c_apps.push(app.clone());
                }
            }
        }
        // c_apps.pop();
        // c_apps.pop();
        // c_apps.pop();
        // c_apps.pop();
        // c_apps[5].size = 549;
        // c_apps[7].size = 123;
        // rust_apps[0].address += 10000;
        // rust_apps[0].address += settings.start_address;
        // rust_apps[0].size -= 1000;
        // rust_apps.push(rust_apps[0].clone());
        // rust_apps[0].size += 1000;
        // rust_apps[0].address -= 10000;
        // rust_apps[0].address -= settings.start_address;
        // for app in rust_apps.iter() {
        //     log::info!(
        //         "rust app name {}, address {:#x}",
        //         app.tbf_header.get_package_name().unwrap_or(""),
        //         app.address
        //     );
        // }
        // log::info!("rust apps: {:?}", rust_apps);
        // log::info!("c apps: {:?}", c_apps);
        let mut permutations = (0..c_apps.len()).permutations(c_apps.len());
        // log::info!("found {} permutations", permutations.try_len().unwrap());

        let page_size: u32 = 512;
        let mut min_padding = usize::MAX;
        let mut saved_configuration: Vec<AppAttributes> = Vec::new();
        // log::info!("{:?}, {:?}", permutations.next(), permutations.next());
        for _ in 0..100_000 {
            // use just 1000 permutations, or else we'll be here for a while
            match permutations.next() {
                Some(order) => {
                    let mut total_padding: usize = 0;
                    // log::info!("permutation is {:?}", order);
                    let mut permutation_index: usize = 0;
                    let mut rust_index: usize = 0;
                    let mut reordered_apps: Vec<AppAttributes> = Vec::new();
                    loop {
                        // log::info!("");
                        // log::info!("");
                        // log::info!("");
                        // log::info!("");
                        // log::info!(
                        //     "perm index {}, order.len {}, rust_index {}, rust_len {}",
                        //     permutation_index,
                        //     order.len(),
                        //     rust_index,
                        //     rust_apps.len()
                        // );
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

                        // Is start_address a multiple of page size? I will try to insert a padding anyway
                        //
                        // how to insert an app:
                        // check if we need a padding to the left of the app
                        //      case 1 (RUST ONLY): there are no apps before, we pad starting from start_address until app start addr
                        //      case 2 (RUST ONLY): pad from last app end addr until current app start
                        //
                        //      C app: pad everytime until next page (if needed_padding > 0)
                        //              > for this, check for last installed app (or start addr, if it is the first)
                        //
                        // do we need padding after the last app? probably not
                        // maybe just insert an extra page / complete with 0xFF ultil next page

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
                            if !start_address.is_multiple_of(page_size as u64) {
                                page_size - start_address as u32 % page_size // c app needs to be inserted at a multiple of page_size
                            } else {
                                0
                            }
                        } else {
                            (rust_apps[rust_index].address - start_address) as u32
                            // rust app needs to be inserted at a fixed address, pad until there
                        };
                        if needed_padding > 0 {
                            // insert a padding
                            total_padding += needed_padding as usize;
                            reordered_apps.push(rust_apps[0].clone());
                            reordered_apps.last_mut().unwrap().address = start_address;
                            reordered_apps.last_mut().unwrap().size = needed_padding;
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
        // log::info!("min padding is {}", min_padding);
        // let mut index = 0;
        // for item in saved_configuration.iter() {
        //     if item.tbf_header.get_package_name().unwrap_or("") == "buttons_wait_for" && (item.size < 5000 || item.size > 10000) {
        //         log::info!("-----PADDING APP------, start:{:#x}, size {}, end {:#x}",
        //             item.address,
        //             item.size,
        //             item.address + item.size as u64
        //         );
        //     } else {
        //         log::info!(
        //             "{}. {}, start address {:#x}, size {}, rustapp {}, end address {:#x}",
        //             index,
        //             item.tbf_header.get_package_name().unwrap_or(""),
        //             item.address,
        //             item.size,
        //             match item.tbf_header.get_fixed_address_flash() {
        //                 Some(_) => {
        //                     true
        //                 }
        //                 None => {
        //                     false
        //                 }
        //             },
        //             item.address + item.size as u64,
        //         );
        //         index += 1;
        //     }
        // }

        let mut pkt: Vec<u8> = Vec::new();
        for item in saved_configuration.iter() {
            // i have to find a better way of determining if i used a padding or not, this is not okay
            if item.tbf_header.get_package_name().unwrap_or("") == "buttons_wait_for"
                && (item.size < 5000 || item.size > 10000)
            {
                // padding (debugging)
                // write padding binary
                let mut buf = create_padding(item.size);
                pkt.append(&mut buf);
            } else {
                pkt.append(&mut app_binaries[item.index as usize]);
            }
            // log::info!("write binary {:?}", app_binaries[index]);
        }
        // log::info!("pkt size is {}", pkt.len());
        let mut loader = session.target().flash_loader();

        loader.add_data(settings.start_address, &pkt)?;

        let mut options = DownloadOptions::default();
        options.keep_unwritten_bytes = true;

        // Finally, the data can be programmed
        // TODO(george-cosma): Can we move this outside the loop? Commit once?
        // (adi): yes, it worked
        loader.commit(session, options)?;
        // log::info!("finished");
        Ok(())
    }
}

fn reconstruct_app(tab: &Option<Tab>, settings: &BoardSettings) -> Option<AppAttributes> {
    if !tab.is_none() {
        let arch = settings
            .arch
            .as_ref()
            .ok_or(InternalError::MisconfiguredBoardSettings(
                "architechture".to_owned(),
            ))
            .unwrap();

        let binary = tab
            .as_ref()
            .unwrap()
            .extract_binary(arch)
            .expect("invalid arch");
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
        // log::info!("header slice {:?}", &binary);
        // panic!();
        let header =
            parse_tbf_header(&binary[0..header_len as usize], tbf_version).expect("invalid header");
        let binary_end_offset = header.get_binary_end();

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
                appfooter.push(0xFFu8);
            }
            // delete
            while appfooter.len() > expected_size as usize {
                appfooter.pop();
            }
            // log::info!("Byte1 {byte1}, byte2 {byte2}");
            // log::info!("footer {:?}",appfooter);
            // log::info!("footer size {}", appfooter.len());
            // log::info!("length from {:?}",
            //     appfooter.get(2..4).unwrap()
            // );
            let footer_info = parse_tbf_footer(&appfooter).expect("Invalid footer!");
            footers.insert(footer_number, TbfFooter::new(footer_info.0, footer_info.1));
            footer_number += 1;
            footer_offset += footer_info.1 + 4;
        }
        // log::info!("escaped loop?");
        return Some(AppAttributes::new(
            if let Some(addr) = header.get_fixed_address_flash() {
                addr as u64
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
    // log::info!("checksum?? {checksum}");
    buf.extend_from_slice(&u32::to_le_bytes(checksum));
    while buf.len() < size as usize {
        buf.push(0x0u8);
    }
    // log::info!("created padding {:?}", buf);
    // log::info!("pad size is {}", buf.len());
    // panic!();
    buf
}

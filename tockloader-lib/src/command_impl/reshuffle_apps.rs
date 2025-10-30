use itertools::Itertools;
use log::warn;

use crate::attributes::app_attributes::AppAttributes;
use crate::board_settings::BoardSettings;
use crate::tabs::tab::Tab;
use tbf_parser::parse::{parse_tbf_header, parse_tbf_header_lengths};

const ALIGNMENT: u64 = 1024;

#[derive(Clone)]
pub enum TockApp {
    Flexible(FlexibleApp),
    Fixed(FixedApp),
}

#[derive(Clone)]
pub struct FlexibleApp {
    idx: Option<usize>,
    size: u64,
}

#[derive(Clone)]
pub struct FixedApp {
    idx: Option<usize>,
    candidate_addresses: Vec<u64>,
    size: u64,
}

impl TockApp {
    pub fn replace_idx(&mut self, new_idx: usize) -> Option<usize> {
        match self {
            TockApp::Flexible(flexible_app) => flexible_app.idx.replace(new_idx),
            TockApp::Fixed(fixed_app) => fixed_app.idx.replace(new_idx),
        }
    }

    pub fn from_app_attributes(app_attributes: &AppAttributes) -> TockApp {
        if let Some(address) = app_attributes.tbf_header.get_fixed_address_flash() {
            TockApp::Fixed(FixedApp {
                idx: None,
                candidate_addresses: vec![address as u64], // (adi): change this when tbf selector gets merged
                size: app_attributes.tbf_header.total_size() as u64,
            })
        } else {
            TockApp::Flexible(FlexibleApp {
                idx: None,
                size: app_attributes.tbf_header.total_size() as u64,
            })
        }
    }

    /// This function returns an instance of TockApp
    pub fn from_tab(tab: &Tab, settings: &BoardSettings) -> Option<TockApp> {
        // extract the binary
        // this should be changed to accomodate candidate_addresses
        let binary = tab
            .extract_binary(settings.arch.clone().unwrap())
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

        if let Some(addr) = header.get_fixed_address_flash() {
            if addr < settings.start_address as u32 {
                // this rust app should not be here
                panic!(
                    "This rust app starts at {addr:#x}, while the board's start_address is {:#x}",
                    settings.start_address
                )
            }
            // turns out that fixed address is a loosely-used term, address has to be aligned down to a multiple of 1024 bytes
            let address = align_down(addr as u64);

            Some(TockApp::Fixed(FixedApp {
                idx: None,
                candidate_addresses: vec![address], // (adi): change this when tbf selector gets merged
                size: total_size as u64,
            }))
        } else {
            Some(TockApp::Flexible(FlexibleApp {
                idx: None,
                size: total_size as u64,
            }))
        }
    }
}

impl FixedApp {
    fn as_index(&self, install_address: u64) -> Index {
        Index {
            idx: self.idx,
            address: install_address,
            size: self.size,
        }
    }
}

impl FlexibleApp {
    fn as_index(&self, install_address: u64) -> Index {
        Index {
            idx: self.idx,
            address: install_address,
            size: self.size,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Index {
    idx: Option<usize>,
    address: u64,
    size: u64,
}

pub fn reshuffle_apps(
    settings: &BoardSettings,
    mut installed_apps: Vec<TockApp>,
) -> Option<Vec<Index>> {
    // On the first pass, we must assign every app its original index, so we can
    // keep track of it.
    for (idx, app) in installed_apps.iter_mut().enumerate() {
        if app.replace_idx(idx).is_some() {
            warn!("Encountered existing index in TockApp at the start of reorder_apps.");
        }
    }

    let mut rust_apps: Vec<&mut FixedApp> = Vec::new();
    let mut c_apps: Vec<&mut FlexibleApp> = Vec::new();

    for app in &mut installed_apps {
        match app {
            TockApp::Flexible(flexible_app) => c_apps.push(flexible_app),
            TockApp::Fixed(fixed_app) => rust_apps.push(fixed_app),
        }
    }

    for app in &mut rust_apps {
        if app.candidate_addresses.is_empty() {
            warn!("Can not reorder apps since fixed application has no candidate addresses!");
            return None;
        }

        // TODO(eva-cosma): Remove this requirement

        // For now this algorithm only supports pre-chosen addresses for fixed apps.
        // We will keep only the first address around.
        if app.candidate_addresses.len() > 1 {
            let first = app.candidate_addresses[0];
            app.candidate_addresses.clear();
            app.candidate_addresses.push(first);
        }
    }

    // this is necessary. If a rust app is already installed, for example: at 0x48000
    // and we want to install another one at 0x40000, reorder them first
    rust_apps.sort_by_key(|app| app.candidate_addresses[0]);

    // make permutations only for the c apps, as their order can be changed
    let mut permutations = (0..c_apps.len()).permutations(c_apps.len());

    let mut min_padding = usize::MAX;
    let mut saved_configuration: Vec<Index> = Vec::new();

    for _ in 0..100_000 {
        // use just 100k permutations, or else we'll be here for a while
        if let Some(order) = permutations.next() {
            let mut total_padding: usize = 0;
            let mut permutation_index: usize = 0;
            let mut rust_index: usize = 0;
            let mut reordered_apps: Vec<Index> = Vec::new();

            loop {
                let insert_c: bool; // every iteration will insert an app, or break if there are none left

                // start either where the last app ends, or at start address if there are no apps
                let address = reordered_apps
                    .last()
                    .map_or(settings.start_address, |app| app.address + app.size);

                if order.get(permutation_index).is_some() {
                    // we have a C app
                    if rust_apps.get(rust_index).is_some() {
                        // we also have a rust app, insert C app only if it fits
                        insert_c = c_apps[order[permutation_index]].size
                            <= rust_apps[rust_index].candidate_addresses[0] - address;
                    } else {
                        // we have only a C app, insert it accordingly
                        insert_c = true;
                    }
                } else {
                    // we don't have a c app
                    if rust_apps.get(rust_index).is_some() {
                        // we have a rust app, insert it
                        insert_c = false;
                    } else {
                        // we don't have any app, break?
                        break;
                    }
                }

                let mut start_address = reordered_apps
                    .last()
                    .map_or(settings.start_address, |app| app.address + app.size);

                let needed_padding = if insert_c {
                    if !start_address.is_multiple_of(settings.page_size) {
                        // c app needs to be inserted at a multiple of page_size
                        settings.page_size - start_address % settings.page_size
                    } else {
                        0
                    }
                } else {
                    if rust_apps[rust_index].candidate_addresses[0] < start_address {
                        // the program wants to insert a rust app where another rust app already exists
                        warn!(
                            "Can't insert the rust app, space is already occupied by another rust app"
                        );
                        return None;
                    }
                    // rust app needs to be inserted at a fixed address, pad until there
                    rust_apps[rust_index].candidate_addresses[0] - start_address
                };

                if needed_padding > 0 {
                    // insert a padding
                    total_padding += needed_padding as usize;
                    reordered_apps.push(Index {
                        idx: None,
                        address: start_address,
                        size: needed_padding,
                    });

                    start_address += needed_padding as u64;
                }

                if insert_c {
                    // insert the c app, also change its address
                    let c_app = c_apps[order[permutation_index]].as_index(start_address);
                    if c_app.idx.is_none() {
                        panic!("C app has no index assigned!");
                    }

                    reordered_apps.push(c_app);
                    permutation_index += 1;
                } else {
                    // insert the rust app, don't change its address because it is fixed
                    let rust_app = rust_apps[rust_index]
                        .as_index(rust_apps[rust_index].candidate_addresses[0]);
                    if rust_app.idx.is_none() {
                        panic!("Rust app has no index assigned!");
                    }

                    reordered_apps.push(rust_app);
                    rust_index += 1;
                }
            }

            // find the configuration that uses the minimum padding
            if total_padding < min_padding {
                min_padding = total_padding;
                saved_configuration = reordered_apps.clone();
                if min_padding == 0 {
                    break;
                }
            }
        } else {
            break;
        }
    }

    Some(saved_configuration)
}

/// This function returns the binary for a padding
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

/// This function takes a rust app's fixed address and aligns it down to ALIGNMENT (1024 currently)
fn align_down(address: u64) -> u64 {
    address - address % ALIGNMENT
}

/// This function creates the full binary that will be written
pub fn create_pkt(configuration: Vec<Index>, mut app_binaries: Vec<Vec<u8>>) -> Vec<u8> {
    let mut pkt: Vec<u8> = Vec::new();
    for item in configuration.iter() {
        if item.idx.is_none() {
            // write padding binary
            let mut buf = create_padding(item.size as u32);
            pkt.append(&mut buf);
        } else {
            pkt.append(&mut app_binaries[item.idx.unwrap()]);
        }
    }
    pkt
}

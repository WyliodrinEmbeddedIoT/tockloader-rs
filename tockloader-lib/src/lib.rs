// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

pub mod attributes;
mod board_settings;
mod errors;
pub mod probe_session;
pub mod tabs;

use attributes::app_attributes::AppAttributes;
use attributes::get_app_attributes::{get_apps_data, get_start_address};
use attributes::get_board_attributes::{get_appaddr, get_board_attributes, get_bootloader_version};
use attributes::get_kernel_attributes::{get_kernel_attributes, get_kernel_version};
use attributes::hardware_attributes::HardwareAttributes;

use tabs::tab::Tab;

use errors::TockloaderError;
use probe_rs::probe::DebugProbeInfo;
use probe_session::ProbeSession;

pub async fn list_probe(
    choice: DebugProbeInfo,
    board: &str,
    chip: &str,
    core_index: &usize,
) -> Vec<AppAttributes> {
    let mut probe_session = ProbeSession::new(choice, board, chip);
    let mut core = probe_session.get_core(*core_index);

    get_apps_data(&mut core)
}

pub async fn info_probe(
    choice: DebugProbeInfo,
    board: &str,
    chip: &str,
    core_index: &usize,
) -> (HardwareAttributes, Vec<AppAttributes>) {
    let mut probe_session = ProbeSession::new(choice, board, chip);

    let mut core = probe_session.get_core(*core_index);

    let mut attributes: HardwareAttributes = HardwareAttributes::new();

    get_board_attributes(&mut core, &mut attributes);

    get_bootloader_version(&mut core, &mut attributes);

    get_kernel_attributes(&mut core, &mut attributes);

    let apps_details = get_apps_data(&mut core);

    (attributes, apps_details)
}

pub async fn install_app(
    choice: DebugProbeInfo,
    board: &String,
    chip: &str,
    core_index: &usize,
    tab_file: Tab,
) -> Result<(), TockloaderError> {
    // Open port and configure it
    let mut probe_session = ProbeSession::new(choice, board, chip);
    let mut core = probe_session.get_core(*core_index);
    let address: u64 = get_appaddr(&mut core).expect("Could not find app address.");

    // Jump through the linked list of apps to check the address to install the app
    let _start_address = get_start_address(&mut core, address);

    // Verify if the specified app is compatible with board
    match tab_file.is_compatible_with_board(board) {
        Ok(value) => {
            if value {
                println!("Specified tab is compatible with board.");
            } else {
                println!("Specified tab is not compatible with board.");
            }
        }
        Err(e) => println!("Something went wrong: {:?}", e),
    }

    // Verify if the specified app is compatible with kernel version
    match tab_file.is_compatible_with_kernel_verison(get_kernel_version(&mut core) as f32) {
        Ok(value) => {
            if value {
                println!("Specified tab is compatible with your kernel version.");
            } else {
                println!("Specified tab is not compatible with your kernel version.");
            }
        }
        Err(e) => println!("Something went wrong: {:?}", e),
    }

    let mut attr = HardwareAttributes::new();
    get_board_attributes(&mut core, &mut attr);
    tab_file.extract_app(attr.arch);
    Ok(())
}

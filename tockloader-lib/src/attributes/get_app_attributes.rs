// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use byteorder::{ByteOrder, LittleEndian};
use probe_rs::{Core, MemoryInterface};
use tbf_parser::parse::parse_tbf_header_lengths;

use crate::attributes::get_kernel_attributes::bytes_to_string;

use super::{app_attributes::AppAttributes, get_board_attributes::get_appaddr};

pub(crate) fn get_apps_data(board_core: &mut Core) -> Vec<AppAttributes> {
    let mut address: u64 = get_appaddr(board_core).expect("Could not find app address.");
    let mut apps_counter = 0;
    let mut apps_details: Vec<AppAttributes> = vec![];

    loop {
        let mut details: AppAttributes = AppAttributes::new();

        details.address = Some(address);

        let mut app_header_data = vec![0u8; 4];

        let _ = board_core.read(address, &mut app_header_data);

        let tbf_version = LittleEndian::read_u16(&app_header_data[0..2]);

        let header_size = LittleEndian::read_u16(&app_header_data[2..4]);

        let mut app_header_data = vec![0u8; header_size.into()];

        let _ = board_core.read(address, &mut app_header_data);

        let total_size = LittleEndian::read_u32(&app_header_data[4..8]);

        let flag = LittleEndian::read_u32(&app_header_data[8..12]);

        let flag_enable = flag & 1;

        let mut enable: bool = false;

        if flag_enable == 1 {
            enable = true;
        }

        let flag_sticky = (flag >> 1) & 1;

        let mut sticky: bool = false;

        if flag_sticky == 1 {
            sticky = true;
        }

        let checksum = LittleEndian::read_u32(&app_header_data[12..16]);

        let tvl_main_type: u16 = LittleEndian::read_u16(&app_header_data[16..18]);

        let init_fn_offset: u32 = LittleEndian::read_u32(&app_header_data[20..24]);

        let protected_size: u32 = LittleEndian::read_u32(&app_header_data[24..28]);

        let minimum_ram_size: u32 = LittleEndian::read_u32(&app_header_data[28..32]);

        let tvl_program_type: u16 = LittleEndian::read_u16(&app_header_data[32..34]);

        if tvl_program_type != 9 {
            break;
        }

        let binary_end_offset: u32 = LittleEndian::read_u32(&app_header_data[48..52]);

        let app_version: u32 = LittleEndian::read_u32(&app_header_data[52..56]);

        let tvl_package_type = LittleEndian::read_u16(&app_header_data[56..58]);

        let package_length = LittleEndian::read_u16(&app_header_data[58..60]);

        let package_name = bytes_to_string(&app_header_data[60..(60 + package_length as usize)]);

        let tvl_kernel_version_type = LittleEndian::read_u16(
            &app_header_data[(header_size as usize - 6)..(header_size as usize - 4)],
        );

        let kernel_major = LittleEndian::read_u16(
            &app_header_data[(header_size as usize - 4)..(header_size as usize - 2)],
        );

        let kernel_minor = LittleEndian::read_u16(
            &app_header_data[(header_size as usize - 2)..header_size as usize],
        );

        let kernel_version = (kernel_major, kernel_minor);

        let footer_size = total_size - binary_end_offset;

        details.tbf_version = Some(tbf_version);
        details.header_size = Some(header_size);
        details.total_size = Some(total_size);
        details.checksum = Some(checksum);
        details.flag_enabled = Some(enable);
        details.flag_sticky = Some(sticky);
        details.tvl_main_type = Some(tvl_main_type);
        details.init_fn_offset = Some(init_fn_offset);
        details.protected_size = Some(protected_size);
        details.minumum_ram_size = Some(minimum_ram_size);
        details.tvl_program_type = Some(tvl_program_type);
        details.binary_end_offset = Some(binary_end_offset);
        details.app_version = Some(app_version);
        details.tvl_package_type = Some(tvl_package_type);
        details.package_name = Some(package_name);
        details.tvl_kernel_version_type = Some(tvl_kernel_version_type);
        details.kernel_major = Some(kernel_major);
        details.kernel_minor = Some(kernel_minor);
        details.kernel_version = Some(kernel_version);
        details.footer_size = Some(footer_size);

        apps_details.insert(apps_counter, details);
        apps_counter += 1;
        address += total_size as u64;
    }
    apps_details
}

// TODO(MicuAna): add error handling
pub(crate) fn get_start_address(board_core: &mut Core, mut address: u64) -> Option<u64> {
    loop {
        // Read a block of 200 8-bit words
        let mut buff = vec![0u8; 200];
        match board_core.read(address, &mut buff) {
            Ok(_) => {}
            Err(e) => {
                println!("Error reading memory: {:?}", e);
                break;
            }
        }
        let (_ver, _header_len, whole_len) =
            match parse_tbf_header_lengths(&buff[0..8].try_into().unwrap()) {
                Ok((ver, header_len, whole_len)) if header_len != 0 => (ver, header_len, whole_len),
                _ => break, // No more apps
            };

        address += whole_len as u64;
    }
    Some(address)
}

// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use probe_rs::{Core, MemoryInterface};

use tbf_parser::{
    self,
    parse::{parse_tbf_footer, parse_tbf_header, parse_tbf_header_lengths},
    types::{TbfFooterV2Credentials, TbfHeader},
};

// Structure used to package an app's header and footer data.
#[derive(Debug)]
pub struct AppAttributes {
    pub tbf_header: TbfHeader,
    pub tbf_footers: Vec<TbfFooter>,
}

// Structure used to package a footers credential data and the size of the footer.
#[derive(Debug)]
pub struct TbfFooter {
    pub credentials: TbfFooterV2Credentials,
    pub size: u32,
}

impl TbfFooter {
    fn new(credentials: TbfFooterV2Credentials, size: u32) -> TbfFooter {
        TbfFooter { credentials, size }
    }
}

impl AppAttributes {
    pub(crate) fn new(header_data: TbfHeader, footers_data: Vec<TbfFooter>) -> AppAttributes {
        AppAttributes {
            tbf_header: header_data,
            tbf_footers: footers_data,
        }
    }

    // Function used to retrive header and footer data
    // from all the applications flashed on the board.
    // Starting from the 0x40000 address,
    // we read the first 8 bytes to determine,
    // using the parse_tbf_header_lengths function, the following:
    // - tbf-version
    // - header_size
    // - total_size
    // Afterward, with the newfound header_size, 
    // we read the the rest of the header information
    // using the parse_tbf_header function.
    // Then, by saving the end of the binary of the app, 
    // which is also the start of the footer of the app, 
    // we calculate the total size of the footer
    // and using calculated total footer size, 
    // the initial offset of the binary end and
    // the offset from the previous footer, which should be 
    // the inital binary offset + the previous footer's size (footer_info.1) 
    // + 4 bytes that contained information regarding the type of the footer and it's length,
    // we can decuce the exact size of the footer that we need to read from the 
    // memory address found by adding the application address and the offset from the previous footer.
    // After that we package it under the TbfFooter structure.
    // And add all the gathered data into a AppAttributes structure
    // that then is placed into the vector of application information called apps_details.

    pub(crate) fn read_apps_data(board_core: &mut Core, addr: u64) -> Vec<AppAttributes> {
        let mut appaddr: u64 = addr;
        let mut apps_counter = 0;
        let mut apps_details: Vec<AppAttributes> = vec![];

        loop {
            let mut appdata = vec![0u8; 8];

            // Do not ignore the result of the read operation
            board_core.read(appaddr, &mut appdata).unwrap();

            let tbf_version: u16;
            let header_size: u16;
            let total_size: u32;

            match parse_tbf_header_lengths(&appdata.try_into().unwrap()) {
                Ok(data) => {
                    tbf_version = data.0;
                    header_size = data.1;
                    total_size = data.2;
                }
                _ => break,
            };

            let mut header_data = vec![0u8; header_size as usize];

            // Do not ignore the result of the read operation
            board_core.read(appaddr, &mut header_data).unwrap();

            let header: TbfHeader = parse_tbf_header(&header_data, tbf_version)
                .unwrap_or_else(|e| panic!("Error found while getting tbf header data: {:?}", e));

            let binary_end_offset = header.get_binary_end();

            let mut footers: Vec<TbfFooter> = vec![];
            let total_footers_size = total_size - binary_end_offset;
            let mut footer_offset = binary_end_offset;
            let mut footer_number = 0;

            loop {
                let mut appfooter = vec![
                    0u8;
                    (total_footers_size - (footer_offset - binary_end_offset))
                        .try_into()
                        .unwrap()
                ];

                // Do not ignore the result of the read operation
                board_core
                    .read(appaddr + footer_offset as u64, &mut appfooter)
                    .unwrap();

                let footer_info: (TbfFooterV2Credentials, u32) = parse_tbf_footer(&appfooter)
                    .unwrap_or_else(|e| panic!("Paniced while obtaining footer data: {:?}", e));

                footers.insert(footer_number, TbfFooter::new(footer_info.0, footer_info.1));

                footer_number += 1;
                footer_offset += footer_info.1 + 4;

                if footer_offset == total_size {
                    break;
                }
            }

            let details: AppAttributes = AppAttributes::new(header, footers);

            apps_details.insert(apps_counter, details);
            apps_counter += 1;
            appaddr += total_size as u64;
        }
        apps_details
    }
}

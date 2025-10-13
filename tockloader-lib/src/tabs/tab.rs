// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use crate::errors::{TabError, TockloaderError};
use crate::tabs::metadata::Metadata;
use std::fs::File;
use std::io::Read;
use tar::Archive;

struct TbfFile {
    pub filename: String,
    pub data: Vec<u8>,
}

pub struct Tab {
    metadata: Metadata,
    tbf_files: Vec<TbfFile>,
}

impl Tab {
    pub fn open(path: String) -> Result<Self, TockloaderError> {
        let mut metadata = None;
        let mut tbf_files = Vec::new();
        let tab_file = File::open(path).map_err(TabError::IO)?;
        let mut archive = Archive::new(tab_file);

        for archive_entry in archive.entries().map_err(TabError::IO)? {
            let mut archive_file = archive_entry.map_err(TabError::IO)?;

            let path = archive_file.path().map_err(TabError::IO)?;
            let file_name = match path.file_name().and_then(|name| name.to_str()) {
                Some(name) => name.to_owned(),
                None => continue,
            };

            if file_name == "metadata.toml" {
                let mut buf = String::new();
                archive_file
                    .read_to_string(&mut buf)
                    .map_err(TabError::IO)?;
                metadata = Some(Metadata::new(buf)?);
            } else if file_name.ends_with(".tbf") {
                let mut data = Vec::new();

                archive_file.read_to_end(&mut data).map_err(TabError::IO)?;
                // log::info!("read filename {:?}", file_name);

                // log::info!("data? {:?}", data);
                tbf_files.push(TbfFile {
                    filename: file_name.to_string(),
                    data,
                });
            }
        }
        // panic!();

        match metadata {
            Some(metadata) => Ok(Tab {
                metadata,
                tbf_files,
            }),
            None => Err(TabError::MissingMetadata.into()),
        }
    }

    pub fn is_compatible_with_kernel_verison(&self, _kernel_version: u32) -> bool {
        // Kernel version seems to not be working properly on the microbit bootloader. It is always
        // "1" despite the actual version.
        // return self.metadata.minimum_tock_kernel_version.major <= kernel_version;
        true
    }

    pub fn is_compatible_with_board(&self, board: &String) -> bool {
        if let Some(boards) = &self.metadata.only_for_boards {
            boards.contains(board)
        } else {
            true
        }
    }

    // maybe change the parameter into board settings?
    pub fn extract_binary(&self, arch: &str) -> Result<Vec<u8>, TockloaderError> {
        for file in &self.tbf_files {
            if file.filename.starts_with(arch) {
                // make an inquire that shows only relevant configurations?
                // flash >= start_addr
                // ram >= ???
                // we need a ram parameter in board_settings
                // if file.filename.starts_with("cortex-m4.0x00040000.0x20008000") { // here i set it manually for testing
                // TODO(adi): this needs a better implementation for rust apps, the tbf is not selected correctly
                // should we select the tab manually? with inquire?
                return Ok(file.data.clone());
            }
        }

        Err(TabError::MissingBinary(arch.to_owned()).into())
    }
}

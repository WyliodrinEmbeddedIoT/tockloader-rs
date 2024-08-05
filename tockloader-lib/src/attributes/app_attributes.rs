// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

#[derive(Debug)]
pub struct AppAttributes {
    pub tbf_version: Option<u16>,
    pub header_size: Option<u16>,
    pub total_size: Option<u32>,
    pub checksum: Option<u32>,
    pub flag_enabled: Option<bool>,
    pub flag_sticky: Option<bool>,
    pub tvl_main_type: Option<u16>,
    pub init_fn_offset: Option<u32>,
    pub protected_size: Option<u32>,
    pub minumum_ram_size: Option<u32>,
    pub tvl_program_type: Option<u16>,
    pub binary_end_offset: Option<u32>,
    pub app_version: Option<u32>,
    pub tvl_package_type: Option<u16>,
    pub package_name: Option<String>,
    pub tvl_kernel_version_type: Option<u16>,
    pub kernel_major: Option<u16>,
    pub kernel_minor: Option<u16>,
    pub kernel_version: Option<(u16, u16)>,
    pub footer_size: Option<u32>,
    pub address: Option<u64>,
}

impl AppAttributes {
    pub(crate) fn new() -> AppAttributes {
        AppAttributes {
            package_name: None,
            flag_enabled: None,
            header_size: None,
            total_size: None,
            kernel_version: None,
            minumum_ram_size: None,
            tbf_version: None,
            checksum: None,
            flag_sticky: None,
            tvl_main_type: None,
            init_fn_offset: None,
            protected_size: None,
            tvl_program_type: None,
            binary_end_offset: None,
            app_version: None,
            tvl_package_type: None,
            tvl_kernel_version_type: None,
            kernel_major: None,
            kernel_minor: None,
            footer_size: None,
            address: None,
        }
    }
}

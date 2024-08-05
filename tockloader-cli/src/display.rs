// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use tockloader_lib::attributes::{
    app_attributes::AppAttributes, hardware_attributes::HardwareAttributes,
};

pub async fn print_list(app_details: &mut [AppAttributes], info: bool) {
    match info {
        false => {
            for (i, temp_data) in app_details.iter().enumerate() {
                println!("\n\x1b[0m\x1b[1;35m ┏━━━━━━━━━━━━━━━━┓");
                println!(
                    "\x1b[0m\x1b[1;31m ┃ \x1b[0m\x1b[1;32m App_{:<9?} \x1b[0m\x1b[1;31m┃",
                    i
                );
                println!("\x1b[0m\x1b[1;33m ┗━━━━━━━━━━━━━━━━┛");
                println!(
                    "\n \x1b[1;32m Name:                {}",
                    temp_data.package_name.clone().unwrap()
                );

                println!(
                    " \x1b[1;32m Version:             {}",
                    temp_data.app_version.unwrap()
                );

                println!(
                    " \x1b[1;32m Enabled:             {}",
                    temp_data.flag_enabled.unwrap()
                );

                println!(
                    " \x1b[1;32m Stricky:             {}",
                    temp_data.flag_sticky.unwrap()
                );

                println!(
                    " \x1b[1;32m Total_Size:          {}\n\n",
                    temp_data.total_size.unwrap()
                );
            }
        }

        true => {
            for (i, temp_data) in app_details.iter().enumerate() {
                println!("\n\x1b[0m\x1b[1;35m ┏━━━━━━━━━━━━━━━━┓");
                println!(
                    "\x1b[0m\x1b[1;31m ┃ \x1b[0m\x1b[1;32m App_{:<9?} \x1b[0m\x1b[1;31m┃",
                    i
                );
                println!("\x1b[0m\x1b[1;33m ┗━━━━━━━━━━━━━━━━┛");
                println!(
                    "\n \x1b[1;32m Name:                {}",
                    temp_data.package_name.clone().unwrap()
                );

                println!(
                    " \x1b[1;32m Version:             {}",
                    temp_data.app_version.unwrap()
                );

                println!(
                    " \x1b[1;32m Enabled:             {}",
                    temp_data.flag_enabled.unwrap()
                );

                println!(
                    " \x1b[1;32m Stricky:             {}",
                    temp_data.flag_sticky.unwrap()
                );

                println!(
                    " \x1b[1;32m Total_Size:          {}",
                    temp_data.total_size.unwrap()
                );

                println!(
                    " \x1b[1;32m Address in Flash:  {:<10}",
                    temp_data.address.unwrap()
                );

                println!(
                    " \x1b[1;32m    TBF version:    {:<10}",
                    temp_data.tbf_version.unwrap()
                );

                println!(
                    " \x1b[1;32m    header_size:    {:<10}",
                    temp_data.header_size.unwrap()
                );

                println!(
                    " \x1b[1;32m    total_size:     {:<10}",
                    temp_data.total_size.unwrap()
                );

                println!(
                    " \x1b[1;32m    checksum:       {:<10}",
                    temp_data.checksum.unwrap()
                );

                println!(" \x1b[1;32m    flags:");
                println!(
                    " \x1b[1;32m        enabled:        {:<10}",
                    temp_data.flag_enabled.unwrap()
                );

                println!(
                    " \x1b[1;32m        sticky:         {:<10}",
                    temp_data.flag_sticky.unwrap()
                );

                println!(
                    " \x1b[1;32m    TVL: Main ({})",
                    temp_data.tvl_main_type.unwrap()
                );

                println!(
                    " \x1b[1;32m        init_fn_offset:             {:<10}",
                    temp_data.init_fn_offset.unwrap()
                );

                println!(
                    " \x1b[1;32m        protected_size:             {:<10}",
                    temp_data.protected_size.unwrap()
                );

                println!(
                    " \x1b[1;32m        minimum_ram_size:           {:<10}",
                    temp_data.minumum_ram_size.unwrap()
                );

                println!(
                    " \x1b[1;32m    TVL: Program ({})",
                    temp_data.tvl_program_type.unwrap()
                );

                println!(
                    " \x1b[1;32m        init_fn_offset:             {:<10}",
                    temp_data.init_fn_offset.unwrap()
                );

                println!(
                    " \x1b[1;32m        protected_size:             {:<10}",
                    temp_data.protected_size.unwrap()
                );

                println!(
                    " \x1b[1;32m        minimum_ram_size:           {:<10}",
                    temp_data.minumum_ram_size.unwrap()
                );

                println!(
                    " \x1b[1;32m        binary_end_offset:          {:<10}",
                    temp_data.binary_end_offset.unwrap()
                );

                println!(
                    " \x1b[1;32m        app_version:                {:<10}",
                    temp_data.app_version.unwrap()
                );

                println!(
                    " \x1b[1;32m    TVL: Package Name ({})",
                    temp_data.tvl_package_type.unwrap()
                );

                println!(
                    " \x1b[1;32m        package_name:               {:<10}",
                    temp_data.package_name.clone().unwrap()
                );

                println!(
                    " \x1b[1;32m    TVL: Kernel Version ({})",
                    temp_data.tvl_kernel_version_type.unwrap()
                );

                println!(
                    " \x1b[1;32m        kernel_major:               {:<10}",
                    temp_data.kernel_major.unwrap()
                );

                println!(
                    " \x1b[1;32m        kernel_minor:               {:<10}",
                    temp_data.kernel_minor.unwrap()
                );

                println!(
                    " \x1b[1;32m        kernel version:             {}.{}",
                    temp_data.kernel_version.unwrap().0,
                    temp_data.kernel_version.unwrap().1
                );

                println!("\n \x1b[1;32m    Footer");

                println!(
                    " \x1b[1;32m        footer_size:            {:<10}",
                    temp_data.footer_size.unwrap()
                );

                println!(" \x1b[1;32m    Footer TVL: Credentials");

                println!(
                    " \x1b[1;32m        Length:                 {:<10}\n\n",
                    temp_data.footer_size.unwrap() - 8
                );
            }
        }
    }
}

pub async fn print_info(attributes: &mut HardwareAttributes) {
    println!("\n\nx1b[1;32m Kernel Attributes");
    println!(
        "\x1b[1;32m     Sentinel:               {:<10}",
        attributes.sentinel.clone().unwrap()
    );
    println!(
        "\x1b[1;32m     Version:                {:<10}",
        attributes.kernel_version.unwrap()
    );
    println!("\x1b[1;32m KATLV: APP Memory");
    println!(
        "\x1b[1;32m     app_memory_start:       {:<10}",
        attributes.app_mem_start.unwrap()
    );
    println!(
        "\x1b[1;32m     app_memory_len:         {:<10}",
        attributes.app_mem_len.unwrap()
    );
    println!("\x1b[1;32m KATLV: Kernel Binary");
    println!(
        "\x1b[1;32m     kernel_binary_start:    {:<10}",
        attributes.kernel_bin_start.unwrap()
    );
    println!(
        "\x1b[1;32m     kernel_binary_len:      {:<10}\n\n",
        attributes.kernel_bin_len.unwrap()
    );
}

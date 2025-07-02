// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use tockloader_lib::attributes::app_attributes::AppAttributes;
use tockloader_lib::attributes::system_attributes::SystemAttributes;

// ANSI escape codes for colors
const RESET: &str = "\x1b[0m";
const BOLD_MAGENTA: &str = "\x1b[1;35m";
const BOLD_RED: &str = "\x1b[1;31m";
const BOLD_GREEN: &str = "\x1b[1;32m";
const BOLD_YELLOW: &str = "\x1b[1;33m";


pub async fn print_list(app_details: &[AppAttributes]) {
    for (i, details) in app_details.iter().enumerate() {
        println!("\n{}{}{}", RESET, BOLD_MAGENTA, " ┏━━━━━━━━━━━━━━━━┓");
        println!(
            "{}{}{} ┃ {}{}{:<9?} {}{}{}┃",
            RESET, BOLD_RED, RESET, BOLD_GREEN, "App_", i, RESET, BOLD_RED, RESET
        );
        println!("{}{}{}", RESET, BOLD_YELLOW, " ┗━━━━━━━━━━━━━━━━┛");
        
        println!(
            "\n {} Name:             {}{}", 
            BOLD_GREEN,
            details.tbf_header.get_package_name().unwrap(),
            RESET
        );

        println!(
            " {} Version:          {}{}", 
            BOLD_GREEN,
            details.tbf_header.get_binary_version(),
            RESET
        );

        println!(
            " {} Enabled:          {}{}", 
            BOLD_GREEN,
            details.tbf_header.enabled(),
            RESET
        );

        println!(
            " {} Sticky:           {}{}", 
            BOLD_GREEN,
            details.tbf_header.sticky(),
            RESET
        );

        println!(
            " {} Total_Size:       {}{}\n\n", 
            BOLD_GREEN,
            details.tbf_header.total_size(),
            RESET
        );
    }
}


pub async fn print_info(app_details: &mut [AppAttributes], system_details: &mut SystemAttributes) {
    for (i, details) in app_details.iter().enumerate() {
        println!("\n{}{}{}", RESET, BOLD_MAGENTA, " ┏━━━━━━━━━━━━━━━━┓");
        println!(
            "{}{}{} ┃ {}{}{:<9?} {}{}{}┃",
            RESET, BOLD_RED, RESET, BOLD_GREEN, "App_", i, RESET, BOLD_RED, RESET
        );
        println!("{}{}{}", RESET, BOLD_YELLOW, " ┗━━━━━━━━━━━━━━━━┛");

        println!(
            "\n {} Name:             {}{}", 
            BOLD_GREEN,
            details.tbf_header.get_package_name().unwrap(),
            RESET
        );

        println!(
            " {} Version:          {}{}", 
            BOLD_GREEN,
            details.tbf_header.get_binary_version(),
            RESET
        );

        println!(
            " {} Enabled:          {}{}", 
            BOLD_GREEN,
            details.tbf_header.enabled(),
            RESET
        );

        println!(
            " {} Sticky:           {}{}", 
            BOLD_GREEN,
            details.tbf_header.sticky(),
            RESET
        );

        println!(
            " {} Total_Size:       {}{}", 
            BOLD_GREEN,
            details.tbf_header.total_size(),
            RESET
        );

        println!(
            " {} Address in Flash: {}{}",
            BOLD_GREEN,
            system_details.appaddr.unwrap(),
            RESET
        );

        println!(
            " {}    TBF version:   {}{}",
            BOLD_GREEN,
            details.tbf_header.get_binary_version(),
            RESET
        );

        println!(
            " {}    header_size:   {}{}",
            BOLD_GREEN,
            details.tbf_header.header_size(),
            RESET
        );

        println!(
            " {}    total_size:    {}{}",
            BOLD_GREEN,
            details.tbf_header.total_size(),
            RESET
        );

        println!(
            " {}    checksum:      {}{}", 
            BOLD_GREEN,
            details.tbf_header.checksum(),
            RESET
        );

        println!(" {}    flags:{}", BOLD_GREEN, RESET);  
        println!(
            " {}        enabled:       {}{}",  
            BOLD_GREEN,
            details.tbf_header.enabled(),
            RESET
        );

        println!(
            " {}        sticky:        {}{}",  
            BOLD_GREEN,
            details.tbf_header.sticky(),
            RESET
        );

        println!(" {}    TVL: Main (1){}", BOLD_GREEN, RESET);  
        println!(
            " {}        init_fn_offset:        {}{}",  
            BOLD_GREEN,
            details.tbf_header.get_init_function_offset(),
            RESET
        );

        println!(
            " {}        protected_size:        {}{}",  
            BOLD_GREEN,
            details.tbf_header.get_protected_size(),
            RESET
        );

        println!(
            " {}        minimum_ram_size:      {}{}",  
            BOLD_GREEN,
            details.tbf_header.get_minimum_app_ram_size(),
            RESET
        );

        println!(" {}    TVL: Program (9){}", BOLD_GREEN, RESET);  
        println!(
            " {}        init_fn_offset:        {}{}",  
            BOLD_GREEN,
            details.tbf_header.get_init_function_offset(),
            RESET
        );

        println!(
            " {}        protected_size:        {}{}",  
            BOLD_GREEN,
            details.tbf_header.get_protected_size(),
            RESET
        );

        println!(
            " {}        minimum_ram_size:      {}{}",  
            BOLD_GREEN,
            details.tbf_header.get_minimum_app_ram_size(),
            RESET
        );

        println!(
            " {}        binary_end_offset:     {}{}",  
            BOLD_GREEN,
            details.tbf_header.get_binary_end(),
            RESET
        );

        println!(
            " {}        app_version:           {}{}",  
            BOLD_GREEN,
            details.tbf_header.get_binary_version(),
            RESET
        );

        println!(" {}    TVL: Package Name (3){}", BOLD_GREEN, RESET);  
        println!(
            " {}        package_name:          {}{}",  
            BOLD_GREEN,
            details.tbf_header.get_package_name().unwrap(),
            RESET
        );

        println!(" {}    TVL: Kernel Version (8){}", BOLD_GREEN, RESET);  
        println!(
            " {}        kernel_major:          {}{}",  
            BOLD_GREEN,
            details.tbf_header.get_kernel_version().unwrap().0,
            RESET
        );

        println!(
            " {}        kernel_minor:          {}{}",  
            BOLD_GREEN,
            details.tbf_header.get_kernel_version().unwrap().1,
            RESET
        );

        println!("\n {}    Footer{}", BOLD_GREEN, RESET);  

        let mut total_footer_size: u32 = 0;

        // Usage of +4 is a result of the structure of the Tock Binary Format (https://book.tockos.org/doc/tock_binary_format)
        // Because we need the real size of the footer including the type and length.
        for footer_details in details.tbf_footers.iter() {
            total_footer_size += footer_details.size + 4;
        }

        println!(
            " {}            footer_size:       {}{}",  
            BOLD_GREEN,
            total_footer_size,
            RESET
        );

        for (j, footer_details) in details.tbf_footers.iter().enumerate() {
            println!(" {}    Footer [{}] TVL: Credentials{}", BOLD_GREEN, j, RESET);  

            println!(
                " {}        Type:                  {}{}",  
                BOLD_GREEN,
                footer_details.credentials.get_type(),
                RESET
            );

            // Usage of -4 is a result of the structure of the Tock Binary Format (https://book.tockos.org/doc/tock_binary_format)
            // Because we only need the size of the credentials without the type and length bytes.
            println!(
                " {}        Length:                {}{}",  
                BOLD_GREEN,
                footer_details.size - 4,
                RESET
            );
        }
    }

    println!("\n\n{} Kernel Attributes{}", BOLD_GREEN, RESET);  
    println!(
        "{}    Sentinel:          {:<10}{}",  
        BOLD_GREEN,
        system_details.sentinel.clone().unwrap(),
        RESET
    );
    println!(
        "{}    Version:           {:<10}{}",  
        BOLD_GREEN,
        system_details.kernel_version.unwrap(),
        RESET
    );
    println!("{} KATLV: APP Memory{}", BOLD_GREEN, RESET);  
    println!(
        "{}    app_memory_start:  {:<10}{}",  
        BOLD_GREEN,
        system_details.app_mem_start.unwrap(),
        RESET
    );
    println!(
        "{}    app_memory_len:    {:<10}{}",  
        BOLD_GREEN,
        system_details.app_mem_len.unwrap(),
        RESET
    );
    println!("{} KATLV: Kernel Binary{}", BOLD_GREEN, RESET);  
    println!(
        "{}    kernel_binary_start: {:<10}{}",  
        BOLD_GREEN,
        system_details.kernel_bin_start.unwrap(),
        RESET
    );
    println!(
        "{}    kernel_binary_len:   {:<10}{}\n\n",  
        BOLD_GREEN,
        system_details.kernel_bin_len.unwrap(),
        RESET
    );
}
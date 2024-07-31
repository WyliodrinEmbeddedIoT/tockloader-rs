// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

mod board_settings;
mod cli;
mod errors;
mod interfaces;


use std::fs::File;
use std::io::Read;

use board_settings::BoardSettings;
use clap::ArgMatches;
use cli::make_cli;
use errors::TockloaderError;

use inquire::Select;
use probe_rs::probe::list::Lister;
use probe_rs::{MemoryInterface, Permissions};

use tar::Archive;
use tbf_parser::parse::*;
use tbf_parser::types::*;
use tock_process_console;
use tokio::time::{sleep, Duration};
use tokio_serial::{FlowControl, Parity, SerialPort, SerialStream, StopBits};
use utf8_decode::Decoder;


#[tokio::main]
async fn main() -> Result<(), TockloaderError> {
    let result = run().await;
    if let Err(e) = &result {
        eprintln!("\n{}", e);
    }
    result
}

async fn run() -> Result<(), TockloaderError> {
    let matches = make_cli().get_matches();

    if matches.get_flag("debug") {
        println!("Debug mode enabled");
    }

    match matches.subcommand() {
        Some(("listen", _sub_matches)) => {
            match tock_process_console::run().await {
                Ok(()) => {}
                Err(_) => {
                    print!("cli bricked!")
                }
            };
        }
        Some(("list", sub_matches)) => {
            list_probes(sub_matches).await?;
        }

        Some(("install", sub_matches)) => {
            install_apps(sub_matches).await?;
        }
        // If only the "--debug" flag is set, then this branch is executed
        // Or, more likely at this stage, a subcommand hasn't been implemented yet.
        _ => {
            println!("Could not run the provided subcommand.");
            _ = make_cli().print_help();
        }
    }

    Ok(())
}

async fn list_probes(sub_matches: &ArgMatches) -> Result<(), TockloaderError> {
    let lister = Lister::new();
    let probes = lister.list_all();
    println!("Probes: {:?}\n", probes);

    let ans = Select::new("Which probe do you want to use?", probes).prompt();

    match ans {
        Ok(choice) => {
            let probe = choice.open().unwrap();

            let chip = sub_matches.get_one::<String>("chip").unwrap();
            let board = sub_matches.get_one::<String>("board").unwrap();

            let board_settings = BoardSettings::new(board.clone(), chip.clone());

            let mut session = probe
                .attach(board_settings.chip, Permissions::default())
                .unwrap();

            println!("Session target: {:?}\n", session.target());
            println!("Session interfaces: {:?}\n", session.architecture());
            println!("Session core: {:?}\n", session.list_cores());

            let core_index = sub_matches.get_one::<usize>("core").unwrap();

            let mut core = session.core(*core_index).unwrap();

            let mut address = board_settings.start_address;

            // Jump through the linked list of apps
            loop {
                // Read a block of 200 8-bit words
                let mut buff = vec![0u8; 200];
                match core.read(address, &mut buff) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Error reading memory: {:?}", e);
                        break;
                    }
                }

                let (ver, header_len, whole_len) =
                    match parse_tbf_header_lengths(&buff[0..8].try_into().unwrap()) {
                        Ok((ver, header_len, whole_len)) if header_len != 0 => {
                            println!("Version: {:?}\n", ver);
                            println!("Header length: {:?}\n", header_len);
                            println!("Whole length: {:?}\n", whole_len);
                            (ver, header_len, whole_len)
                        }
                        _ => break, // No more apps
                    };

                let header = parse_tbf_header(&buff[0..header_len as usize], ver);
                match header {
                    Ok(header) => {
                        println!("Enabled: {:?}\n", header.enabled());
                        println!(
                            "Minimum App Ram Size: {:?}\n",
                            header.get_minimum_app_ram_size()
                        );
                        println!(
                            "Init function offset: {:?}\n",
                            header.get_init_function_offset()
                        );
                        println!("Protected size: {:?}\n", header.get_protected_size());
                        println!(
                            "Package name: {:?}\n",
                            header.get_package_name().unwrap_or_default()
                        );
                        println!(
                            "Kernel version: {:?}\n",
                            header.get_kernel_version().unwrap_or_default()
                        );
                    }
                    // TODO(MicuAna): refactor when reworking errors
                    Err(TbfParseError::ChecksumMismatch(
                        provided_checksum,
                        calculated_checksum,
                    )) => {
                        println!(
                            "Checksum mismatch: provided = {}, calculated = {}",
                            provided_checksum, calculated_checksum
                        );
                        break;
                    }
                    Err(e) => {
                        println!("Failed to parse TBF header: {:?}", e);
                        break;
                    }
                }
                address += whole_len as u64;
            }
        }
        Err(err) => println!("{}", err),
    }

    Ok(())
}
async fn install_apps(sub_matches: &ArgMatches) -> Result<(), TockloaderError> {
    // Enter bootloader mode for microbit
    sleep(Duration::from_millis(500)).await;
    let ports = tokio_serial::available_ports()?;
    println!("Found {} port(s)\n", ports.len());
    // Use the first port
    let port = ports[0].clone();
    println!("Using port {} for the bootloader\n", port.port_name);
    // Open the port and configure it
    let builder = tokio_serial::new(port.port_name, 115200);
    match SerialStream::open(&builder) {
        Ok(mut port) => {
            println!("Serial port opened successfully!\n");
            port.set_parity(Parity::None).unwrap();
            port.set_stop_bits(StopBits::One).unwrap();
            port.set_flow_control(FlowControl::None).unwrap();
            port.set_timeout(Duration::from_millis(500)).unwrap();
            port.write_request_to_send(false).unwrap();
            port.write_data_terminal_ready(false).unwrap();
        }
        Err(e) => {
            eprintln!("Failed to open serial port: {}\n", e);
        }
    }

    let chip = sub_matches.get_one::<String>("chip").unwrap();
    let board = sub_matches.get_one::<String>("board").unwrap();
    let board_settings = BoardSettings::new(board.clone(), chip.clone());
    let tab_path = sub_matches.get_one::<String>("tab").unwrap();

    // This is temporary
    let kernel_version = sub_matches.get_one::<String>("kernver").unwrap();

    let mut archive = Archive::new(File::open(tab_path).unwrap());
    for entry in archive.entries().unwrap().into_iter() {
        match entry {
            Ok(mut entry) => {
                if let Ok(path) = entry.path() {
                    if let Some(file_name) = path.file_name() {
                        if file_name == "metadata.toml" {
                            let mut buf = String::new();
                            entry.read_to_string(&mut buf).unwrap();
                            let replaced = buf.replace("\"", "");
                            let parts = replaced.split("\n");
                            let collection = parts.collect::<Vec<&str>>();

                            for item in collection {
                                if item.contains("only-for-boards") {
                                    let columns = item.split("=");
                                    let elem = columns.collect::<Vec<&str>>();
                                    let all_boards = elem[1].split(", ");
                                    let boards = all_boards.collect::<Vec<&str>>();
                                    for bd in boards {
                                        if bd == board {
                                            println!("App is compatible with board!");
                                            break;
                                        }
                                    }
                                    println! {"App is not compatible with board!"};
                                    break;
                                }
                                if item.contains("minimum-tock-kernel-version") {
                                    let columns = item.split("=");
                                    let elem = columns.collect::<Vec<&str>>();
                                    let kernver = elem[1];
                                    if kernver == kernel_version {
                                        println!("App is compatible with this kernel version!");
                                    } else {
                                        println! {"App is compatible with this kernel version!"};
                                    }
                                    break;
                                }
                            }
                        }
                    } else {
                        eprintln!("Failed to get path");
                    }
                }
            }
            Err(e) => eprintln!("Failed to get entry: {}", e),
        }
    }

    // Identify the address to write the new app(s)
    let lister = Lister::new();
    let probes = lister.list_all();
    println!("Probes: {:?}\n", probes);

    let ans = Select::new("Which probe do you want to use?", probes).prompt();

    match ans {
        Ok(choice) => {
            let probe = choice.open().unwrap();

            let mut session = probe
                .attach(board_settings.chip, Permissions::default())
                .unwrap();

            println!("Session target: {:?}\n", session.target());
            println!("Session interfaces: {:?}\n", session.architecture());
            println!("Session core: {:?}\n", session.list_cores());

            let core_index = sub_matches.get_one::<usize>("core").unwrap();

            let mut core = session.core(*core_index).unwrap();

            let mut address = board_settings.start_address;

            // Jump through the linked list of apps
            loop {
                // Read a block of 200 8-bit words
                let mut buff = vec![0u8; 200];
                match core.read(address, &mut buff) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Error reading memory: {:?}", e);
                        break;
                    }
                }
                let (_ver, _header_len, whole_len) =
                    match parse_tbf_header_lengths(&buff[0..8].try_into().unwrap()) {
                        Ok((ver, header_len, whole_len)) if header_len != 0 => {
                            (ver, header_len, whole_len)
                        }
                        _ => break, // No more apps
                    };

                address += whole_len as u64;
            }

            let mut bytes = vec![0u8; 1024];
            match core.read(0x600, &mut bytes) {
                Ok(_) => {
                    let mut i: u16 = 0;
                    while i < 1024 {
                        let decoder =
                            Decoder::new(bytes[i as usize..(i + 8) as usize].iter().cloned());
                        let mut key = String::new();
                        for c in decoder {
                            key.push(c?);
                        }
                        println!("{}", key);
                        let vlen = bytes[(i + 8) as usize];
                        let index: u16 = vlen as u16 + 9;
                        let decoder = Decoder::new(
                            bytes[(i + 9) as usize..(i + index) as usize]
                                .iter()
                                .cloned(),
                        );
                        let mut value = String::new();
                        for c in decoder {
                            value.push(c?);
                        }
                        println!("{}", value);
                        i = i + 64;
                    }
                }
                Err(e) => {
                    println!("Error reading memory: {:?}", e);
                }
            }
        }
        Err(err) => println!("{}", err),
    }

    Ok(())
}


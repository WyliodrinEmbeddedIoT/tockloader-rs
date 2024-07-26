// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

mod cli;
mod errors;
mod interfaces;


use std::io::{stdin, Read};

use interfaces::serial::{self, serial_data_get, serial_pick};
use cli::make_cli;
use errors::TockloaderError;
use tock_process_console;
use tokio_serial::SerialPortType;
use tokio_util::io::poll_read_buf;

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
            let _ = match tock_process_console::run().await {
                Ok(()) => {}
                Err(_) => {
                    print!("cli bricked!")
                }
            };
        }

        Some(("info", _sub_matches)) => {
            let mut vec_boards: Vec<String> = vec![];
            let mut board_ports: Vec<String> = vec![];

            (vec_boards, board_ports) = serial_data_get().await;

            serial_pick(vec_boards).await;
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

// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

mod cli;
mod display;
mod errors;
mod serial;

use cli::make_cli;
use display::{print_info, print_list};
use errors::TockloaderError;
use serial::select_probe;
use tockloader_lib::{info_probe, install_app, list_probe, tabs::tab::Tab};

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
            let probe = select_probe();
            match probe {
                Ok(probe) => {
                    let mut apps_details = list_probe(
                        probe,
                        sub_matches.get_one::<String>("board").unwrap(),
                        sub_matches.get_one::<String>("chip").unwrap(),
                        sub_matches.get_one::<usize>("core").unwrap(),
                    )
                    .await;
                    print_list(&mut apps_details, false).await;
                }
                Err(err) => println!("{}", err),
            }
        }
        Some(("install", sub_matches)) => {
            let probe = select_probe();
            let tab_file = Tab::new(sub_matches.get_one::<String>("tab").unwrap().to_string());
            match probe {
                Ok(probe) => {
                    install_app(
                        probe,
                        sub_matches.get_one::<String>("board").unwrap(),
                        sub_matches.get_one::<String>("chip").unwrap(),
                        sub_matches.get_one::<usize>("core").unwrap(),
                        tab_file,
                    )
                    .await
                    .unwrap();
                }
                Err(err) => println!("{}", err),
            }
        }
        Some(("info", sub_matches)) => {
            let probe = select_probe();
            match probe {
                Ok(probe) => {
                    let mut attributes = info_probe(
                        probe,
                        sub_matches.get_one::<String>("board").unwrap(),
                        sub_matches.get_one::<String>("chip").unwrap(),
                        sub_matches.get_one::<usize>("core").unwrap(),
                    )
                    .await;
                    print_list(&mut attributes.1, true).await;
                    print_info(&mut attributes.0).await;
                }
                Err(err) => println!("{}", err),
            }
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

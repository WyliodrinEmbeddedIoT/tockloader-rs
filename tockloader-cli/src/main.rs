// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

mod cli;
mod display;
mod known_boards;

use anyhow::{Context, Result};
use clap::ArgMatches;
use cli::make_cli;
use display::print_list;
use known_boards::KnownBoardNames;
use tockloader_lib::{
    board_settings::BoardSettings,
    connection::{Connection, ConnectionInfo, ProbeTargetInfo, SerialTargetInfo},
    known_boards::KnownBoard,
    list_debug_probes, list_serial_ports,
    tabs::tab::Tab,
};

fn get_serial_target_info(user_options: &ArgMatches) -> SerialTargetInfo {
    let board = get_known_board(user_options);
    if let Some(board) = board {
        return board.serial_target_info();
    }

    let mut result = SerialTargetInfo::default();

    if let Some(baud_rate) = user_options.get_one::<u32>("baud-rate") {
        result.baud_rate = *baud_rate;
    }

    result
}

fn get_probe_target_info(user_options: &ArgMatches) -> ProbeTargetInfo {
    let board = get_known_board(user_options);
    if let Some(board) = board {
        return board.probe_target_info();
    }

    let chip = user_options
        .get_one::<String>("chip")
        .expect("Expected validation to catch missing chip")
        .clone();

    let mut result = ProbeTargetInfo::default(chip);

    if let Some(core) = user_options.get_one::<usize>("core") {
        result.core = *core;
    }

    result
}

fn get_board_settings(user_options: &ArgMatches) -> BoardSettings {
    let board = get_known_board(user_options);
    if let Some(board) = board {
        return board.get_settings();
    }

    let mut result = BoardSettings::default();

    if let Some(strat_address_str) = user_options.get_one::<String>("app-address") {
        todo!()
    }

    return result;
}

fn using_serial(user_options: &ArgMatches) -> bool {
    *user_options.get_one::<bool>("serial").unwrap_or(&false)
}

fn get_known_board(user_options: &ArgMatches) -> Option<Box<dyn KnownBoard>> {
    user_options.get_one::<String>("board").map(|board| {
        match KnownBoardNames::from_str(&board).expect("validation to ensure valid board") {
            KnownBoardNames::NucleoF4 => {
                Box::new(tockloader_lib::known_boards::NucleoF4) as Box<dyn KnownBoard>
            }
            KnownBoardNames::MicrobitV2 => {
                Box::new(tockloader_lib::known_boards::MicrobitV2) as Box<dyn KnownBoard>
            }
        }
    })
}

fn open_connection(user_options: &ArgMatches) -> Result<Connection> {
    if using_serial(&user_options) {
        let path = if let Some(path) = user_options.get_one::<String>("port") {
            path.clone()
        } else {
            let serial_ports = list_serial_ports().context("Failed to list serial ports.")?;
            let port_names: Vec<_> = serial_ports.iter().map(|p| p.port_name.clone()).collect();
            let ans = inquire::Select::new("Which serial port do you want to use?", port_names)
                .prompt()
                .context("No device is connected.")?;
            ans
        };

        Connection::open(ConnectionInfo::SerialInfo(
            path,
            get_serial_target_info(&user_options),
        ))
        .context("Failed to open serial connection.")
    } else {
        let ans =
            inquire::Select::new("Which debug probe do you want to use?", list_debug_probes())
                .prompt()
                .context("No debug probe is connected.")?;

        Connection::open(ConnectionInfo::ProbeInfo(
            ans,
            get_probe_target_info(&user_options),
        ))
        .context("Failed to open probe connection.")
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut cmd = cli::make_cli();
    let matches = cmd.get_matches_mut();

    match matches.subcommand() {
        Some(("listen", sub_matches)) => {
            cli::validate(&mut cmd, &sub_matches);
            tock_process_console::run()
                .await
                .context("Failed to run console.")?;
        }
        Some(("list", sub_matches)) => {
            cli::validate(&mut cmd, &sub_matches);

            let mut conn = open_connection(&sub_matches)?;
            let settings = get_board_settings(&sub_matches);

            let app_details = tockloader_lib::list(&mut conn, &settings)
                .await
                .context("Failed to list apps.")?;

            display::print_list(&app_details).await;
        }
        Some(("info", sub_matches)) => {
            cli::validate(&mut cmd, &sub_matches);
            let mut conn = open_connection(&sub_matches)?;
            let settings = get_board_settings(&sub_matches);

            let mut attributes = tockloader_lib::info(&mut conn, &settings)
                .await
                .context("Failed to get data from the board.")?;

            display::print_info(&mut attributes.apps, &mut attributes.system).await;
        }
        Some(("install", sub_matches)) => {
            cli::validate(&mut cmd, &sub_matches);
            let tab_file = Tab::open(sub_matches.get_one::<String>("tab").unwrap().to_string())
                .context("Failed to use provided tab file.")?;

            let mut conn = open_connection(&sub_matches)?;
            let settings = get_board_settings(&sub_matches);

            tockloader_lib::install_app(&mut conn, &settings, tab_file)
                .await
                .context("Failed to install app.")?;
        }
        _ => {
            println!("Could not run the provided subcommand.");
            _ = make_cli().print_help();
        }
    }
    Ok(())
}

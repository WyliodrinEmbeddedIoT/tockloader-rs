// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

pub mod board_interface;
pub mod virtual_terminal;

use std::{error, io::stdin};

use clap::ArgMatches;
use tokio_serial::{SerialPortType, SerialStream};

use crate::errors::TockloaderError;

pub struct SerialInterface {
    port: String,
    baud_rate: u32,
    stream: Option<SerialStream>,
}

impl SerialInterface {
    pub fn new(args: &ArgMatches) -> Result<Self, TockloaderError> {
        // If the user has specified a port, we want to try to use it.
        // Otherwise, we let tokio-serial enumarate all ports and
        // if multiple ports are present, we let the user decide which.
        let port = if let Some(user_port) = args.get_one::<String>("port") {
            user_port.clone()
        } else {
            let available_ports = tokio_serial::available_ports()?;

            if available_ports.is_empty() {
                return Err(TockloaderError::NoPortAvailable);
            } else if available_ports.len() == 1 {
                clean_port_path(available_ports[0].port_name.clone())
            } else {
                // available_ports.len() > 1
                // todo!("Make user choose out of multiple available ports")
                for n in 0..available_ports.len() {
                    println!("port [{}]: {:?},\n", n, available_ports[n]);
                }

                clean_port_path(available_ports[15].port_name.clone())
            }
        };

        let baud_rate = if let Some(baud_rate) = args.get_one::<u32>("baud-rate") {
            *baud_rate
        } else {
            unreachable!("'--baud-rate' should have a default value.")
        };

        Ok(Self {
            port,
            baud_rate,
            stream: None,
        })
    }
}

// When listing available ports, tokio_serial list unix ports like so:
//     /sys/class/tty/ttyACM0
//     /sys/class/tty/<port>
// For some users, tokio_serial fails to open ports using this path scheme.
// This function replaces it with the normal '/dev/<port>' scheme.
// Windows COM ports should not be affected.
fn clean_port_path(port: String) -> String {
    if port.contains("/sys/class/tty/") {
        port.replace("/sys/class/tty/", "/dev/")
    } else {
        port
    }
}

pub async fn serial_data_get() -> (Vec<String>, Vec<String>) {
    let available_ports = match tokio_serial::available_ports() {
        Ok(ports) => ports,
        Err(error) => panic!("Error while searching for ports: {}", error),
    };

    let mut vec_boards: Vec<String> = vec![];
    let mut board_ports: Vec<String> = vec![];
    for (port_index, port) in available_ports.iter().enumerate() {
        let product = match &port.port_type {
            SerialPortType::UsbPort(usb) => usb.product.clone(),
            SerialPortType::PciPort => Some("PciPort".to_string()),
            SerialPortType::BluetoothPort => Some("BluetoothPort".to_string()),
            SerialPortType::Unknown => Some("Unknown".to_string()),
        };

        if let SerialPortType::UsbPort(_) = port.port_type {
            let temp_serial = format! {"Port[{port_index}](Name:{:#?}, Type:{}), \n", port.port_name, product.unwrap_or("Unknown".to_string())};
            vec_boards.push(temp_serial.clone().into());
            board_ports.push(port.port_name.clone());
        }
    }

    (vec_boards, board_ports)
}

pub async fn serial_pick(boards: Vec<String>, boards_ports: Vec<String>) -> Result<String, String>{
    for port in boards.iter() {
        print!("{}", port)
    }

    let mut port = String::new();
    stdin().read_line(&mut port).unwrap();

    port.remove(port.len() - 1);
    let port_number = port.parse::<usize>();

    if port_number.is_ok() {
        //TODO(NegrilaRares) PORT PROCESSING
        print!("{}", boards[port_number.clone().unwrap()]);
        Ok(boards_ports[port_number.unwrap()].clone())

    } else {
        println!("Invalid port inputed.");
        Err("Invalid port inputed.".to_string())
    }

}

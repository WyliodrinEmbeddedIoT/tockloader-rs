#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

use probe_rs::probe::list::Lister;
use serde::{Deserialize, Serialize};
use tockloader_lib::board_settings::BoardSettings;
use tockloader_lib::connection::SerialConnection;
use tockloader_lib::connection::SerialTargetInfo;
use tockloader_lib::connection::{Connection, ProbeRSConnection, ProbeTargetInfo};
use tockloader_lib::CommandInfo;
use tokio_serial::available_ports;
use tokio_serial::FlowControl;
use tokio_serial::Parity;
use tokio_serial::SerialPortType;
use tokio_serial::StopBits;

#[derive(Debug, Serialize, Deserialize)]
pub struct DebugProbeSummary {
    pub identifier: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub serial_number: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerialPortSummary {
    pub port_name: String,
    pub usb_vid: Option<u16>,
    pub usb_pid: Option<u16>,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub serial_number: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectedDevices {
    pub debug_probes: Vec<DebugProbeSummary>,

    pub serial_ports: Vec<SerialPortSummary>,
}

#[tauri::command]
async fn list_all_devices() -> Result<ConnectedDevices, String> {
    // 1. List Debug Probes
    let probes = Lister::new().list_all();
    let debug_probe_summaries: Vec<DebugProbeSummary> = probes
        .into_iter()
        .map(|p| DebugProbeSummary {
            identifier: p.identifier,
            vendor_id: p.vendor_id,
            product_id: p.product_id,
            serial_number: p.serial_number,
        })
        .collect();

    // 2. List Serial Ports
    let serial_ports = match available_ports() {
        Ok(ports) => ports,
        Err(e) => {
            eprintln!("Error listing serial ports: {e:?}");
            return Err(format!("Failed to list serial ports: {e}"));
        }
    };

    let serial_port_summaries: Vec<SerialPortSummary> = serial_ports
        .into_iter()
        .map(|p| {
            let mut usb_vid = None;
            let mut usb_pid = None;
            let mut manufacturer = None;
            let mut product = None;
            let mut serial_number = None;

            // Extract USB-specific fields if the port is a USB port
            if let SerialPortType::UsbPort(usb_info) = p.port_type {
                usb_vid = Some(usb_info.vid);
                usb_pid = Some(usb_info.pid);
                manufacturer = usb_info.manufacturer;
                product = usb_info.product;
                serial_number = usb_info.serial_number;
            }

            SerialPortSummary {
                port_name: p.port_name,
                usb_vid,
                usb_pid,
                manufacturer,
                product,
                serial_number,
            }
        })
        .collect();

    // Combine results into the ConnectedDevices struct
    Ok(ConnectedDevices {
        debug_probes: debug_probe_summaries,
        serial_ports: serial_port_summaries,
    })
}

#[tauri::command]
async fn connect_to_probe(
    probe_identifier: String,
    chip: String,
    core: usize,
) -> Result<String, String> {
    let probes = Lister::new().list_all();
    let debug_probe_info = probes
        .into_iter()
        .find(|p| p.identifier == probe_identifier)
        .ok_or_else(|| format!("Probe with identifier '{probe_identifier}' not found."))?;

    let target_info = ProbeTargetInfo { chip, core };
    let mut connection = ProbeRSConnection::new(debug_probe_info, target_info);

    match connection.open().await {
        Ok(_) => {
            let info_result = connection.info(&BoardSettings::default()).await;
            println!("Result of connection.info(): {info_result:?}");
            Ok(format!(
                "Successfully connected to probe '{}' with chip '{}' and core {}.",
                probe_identifier, connection.target_info.chip, connection.target_info.core
            ))
        }
        Err(e) => {
            eprintln!("Error connecting to probe: {e:?}");
            Err(format!("Failed to connect to probe: {e}"))
        }
    }
}
//new code for serial port connection
#[tauri::command]
#[allow(clippy::too_many_arguments)]
async fn connect_to_serial(
    port_name: String,
    baud_rate: u32,
    parity_str: String,
    stop_bits_str: String,
    flow_control_str: String,
    timeout_ms: u64,
    request_to_send: bool,
    data_terminal_ready: bool,
) -> Result<String, String> {
    //added now
    println!("Attempting to connect to serial port: {port_name}");
    println!("Settings: Baud={baud_rate}, Parity={parity_str}, StopBits={stop_bits_str}, Flow={flow_control_str}, Timeout={timeout_ms}ms, RTS={request_to_send}, DTR={data_terminal_ready}",
    );

    // Parse string inputs into tokio_serial enums
    let parity = match parity_str.as_str() {
        "None" => Parity::None,
        "Odd" => Parity::Odd,
        "Even" => Parity::Even,
        _ => return Err(format!("Invalid parity: {parity_str}")),
    };

    let stop_bits = match stop_bits_str.as_str() {
        "One" => StopBits::One,
        "Two" => StopBits::Two,
        _ => return Err(format!("Invalid stop bits: {stop_bits_str}")),
    };

    let flow_control = match flow_control_str.as_str() {
        "None" => FlowControl::None,
        "Software" => FlowControl::Software,
        "Hardware" => FlowControl::Hardware,
        _ => return Err(format!("Invalid flow control: {flow_control_str}")),
    };

    let baud_rate_val = baud_rate;
    let serial_target_info = SerialTargetInfo {
        baud_rate: baud_rate_val,
        parity,
        stop_bits,
        flow_control,
        timeout: Duration::from_millis(timeout_ms),
        request_to_send,
        data_terminal_ready,
    };

    let mut connection = SerialConnection::new(port_name.clone(), serial_target_info);
    // Attempt to open the serial connection
    match connection.open().await {
        Ok(_) => {
            let info_result = connection.info(&BoardSettings::default()).await;
            println!("Result of connection.info(): {info_result:?}");
            println!("Successfully opened serial port.");
            Ok(format!("Successfully connected to serial port '{port_name}' at {baud_rate_val} baud with custom settings."))
        }
        Err(e) => {
            eprintln!("Error connecting to serial port: {e:?}");
            Err(format!("Failed to connect to serial port: {e}"))
        }
    }
}
// The main function for your Tauri application
fn main() {
    tauri::Builder::default()
        // Register the new command so it can be called from the frontend
        .invoke_handler(tauri::generate_handler![
            list_all_devices,
            connect_to_probe,
            connect_to_serial
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

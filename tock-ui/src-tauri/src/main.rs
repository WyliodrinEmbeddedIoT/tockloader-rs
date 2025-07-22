#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use probe_rs::probe::list::Lister;
use serde::{Deserialize, Serialize};
use tokio_serial::available_ports;
use tokio_serial::SerialPortType;
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

// The main function for your Tauri application
fn main() {
    tauri::Builder::default()
        // Register the new command so it can be called from the frontend
        .invoke_handler(tauri::generate_handler![list_all_devices])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

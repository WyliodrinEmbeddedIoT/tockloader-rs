use anyhow::Context;
use async_trait::async_trait;
use probe_rs::flashing::DownloadOptions;
use probe_rs::MemoryInterface;
use tbf_parser::parse::{parse_tbf_header, parse_tbf_header_lengths};

use crate::board_settings::BoardSettings;
use crate::connection::{Connection, ProbeRSConnection};
use crate::errors::{InternalError, TockError, TockloaderError};
use crate::CommandUninstall;

#[async_trait]
impl CommandUninstall for ProbeRSConnection {
    async fn uninstall_app(
        &mut self,
        settings: &BoardSettings,
        app_name: Option<&str>,
    ) -> Result<(), TockloaderError> {
        if !self.is_open() {
            return Err(InternalError::ConnectionNotOpen.into());
        }
        let session = self.session.as_mut().expect("Board must be open");
        loop {
            let mut installed_apps: Vec<AppData> = Vec::new();
            let mut index: u8 = 1;
            let mut appaddr: u64 = settings.start_address;

            // make "Delete all" a part of this vector
            installed_apps.push(AppData {
                name: "Delete all".to_string(),
                address: settings.start_address,
                index: 0,
                size: 0,
            });

            loop {
                let mut board_core = session.core(self.target_info.core)?;
                // Read the first 8 bytes, which is the length of a TLV header.
                let mut appdata = vec![0u8; 8];

                board_core.read(appaddr, &mut appdata)?;
                let tbf_version: u16;
                let header_size: u16;
                let app_size: u32;

                // The first 8 bytes of the application data contain the TBF header
                // lengths and version.
                //
                // Note on expect: `read` always fills up the entire buffer, which
                // was previously declared as 8 bytes.
                match parse_tbf_header_lengths(
                    &appdata[0..8]
                        .try_into()
                        .expect("Buffer length must be at least 8 bytes long."),
                ) {
                    Ok(data) => {
                        tbf_version = data.0;
                        header_size = data.1;
                        app_size = data.2;
                    }
                    _ => break,
                };

                // Read the rest of the header
                let mut header_data = vec![0u8; header_size.into()];
                board_core.read(appaddr, &mut header_data)?;

                let header = parse_tbf_header(&header_data, tbf_version)
                    .map_err(TockError::InvalidAppTbfHeader)?;
                let pname = header.get_package_name().unwrap_or("").to_owned();
                installed_apps.push(AppData {
                    address: appaddr,
                    name: pname,
                    size: app_size,
                    index,
                });
                index += 1;
                appaddr += app_size as u64;
            }
            if installed_apps.len() == 1 {
                if app_name.is_none() {
                    return Err(TockloaderError::Tock(TockError::MissingAttribute(
                        "No apps installed".to_string(),
                    )));
                } else {
                    return Err(TockloaderError::Tock(TockError::MissingAttribute(
                        "Requested app is not instaleld".to_string(),
                    )));
                }
            }
            let mut app: &AppData;
            match app_name {
                Some(app_name) => {
                    app = match installed_apps.iter().find(|iter| iter.name == app_name) {
                        Some(app) => app,
                        None => break,
                    }
                }
                None => loop {
                    app = inquire::Select::new(
                        "Which app do you want to uninstall?",
                        installed_apps.iter().clone().collect(),
                    )
                    .prompt()
                    .context("No apps installed")
                    .unwrap();

                    if inquire::Select::new(
                        format!("You chose {app}",).as_str(),
                        ["Cancel", "Confirm"].to_vec(),
                    )
                    .prompt()
                    .unwrap()
                        == "Confirm"
                    {
                        break;
                    }
                },
            }
            let address: u64; // here we'll write the remaining apps
            let buf_size: usize = if app.index > 0 {
                // buf_size is the size of all apps that are to the right of our target app
                address = app.address; // put remaining apps where target app starts
                installed_apps.iter().fold(0, |total_size, aux_app| {
                    if aux_app.index > app.index {
                        total_size + aux_app.size as usize // increase if app is to the right
                    } else {
                        total_size
                    }
                })
            } else {
                // Delete all case
                address = settings.start_address; // put a 0x0 byte here, all apps will become invalid
                installed_apps.iter().fold(0, |total_size, aux_app| {
                    total_size + aux_app.size as usize // increase if app is to the right
                })
            };
            let mut buffer = vec![0u8; buf_size];
            if buf_size > 0 && app.index > 0 {
                // copy apps only if we didn't choose "Delete all"
                // we have apps to the right
                let mut board_core = session.core(self.target_info.core)?;
                board_core.read(installed_apps[app.index as usize + 1].address, &mut buffer)?;
            }

            buffer.extend([0x0; 512]); // add an extra page of 0x0
            let mut loader = session.target().flash_loader();

            loader.add_data(address, &buffer)?;

            let mut options = DownloadOptions::default();
            options.keep_unwritten_bytes = true;

            loader.commit(session, options)?;

            if app_name.is_none() {
                // exit if we don't have a name set, else continue removing
                break;
            }
        }
        Ok(())
    }
}

struct AppData {
    name: String,
    address: u64,
    size: u32,
    index: u8,
}

impl std::fmt::Display for AppData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.name == "Delete all" {
            write!(f, "{}", self.name)
        } else {
            write!(
                f,
                "{}. {} - start: {:#x}, size: {}",
                self.index, self.name, self.address, self.size
            )
        }
    }
}

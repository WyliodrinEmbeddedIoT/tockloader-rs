use anyhow::Context;
use async_trait::async_trait;
use probe_rs::flashing::DownloadOptions;
use probe_rs::MemoryInterface;
use tbf_parser::parse::{parse_tbf_header, parse_tbf_header_lengths};

use crate::board_settings::BoardSettings;
use crate::connection::{Connection, ProbeRSConnection};
use crate::errors::{InternalError, TockError, TockloaderError};
use crate::CommandDisableApp;

const ENABLED_OFFSET: u64 = 8;
const CHECKSUM_OFFSET: u64 = 12;

#[async_trait]
impl CommandDisableApp for ProbeRSConnection {
    async fn disable_app(
        &mut self,
        settings: &BoardSettings,
        app_name: Option<&str>,
    ) -> Result<(), TockloaderError> {
        if !self.is_open() {
            return Err(InternalError::ConnectionNotOpen.into());
        }
        let session = self.session.as_mut().expect("Board must be open");
        'outer: loop {
            // this loop is labeled so we can exit early if the app is already disabled
            let mut installed_apps: Vec<AppData> = Vec::new();
            let mut index: u8 = 1;
            let mut appaddr: u64 = settings.start_address;

            // make "Disable all" a part of this vector
            installed_apps.push(AppData {
                name: "Disable all".to_string(),
                address: settings.start_address,
                index: 0,
                size: 0,
                checksum: 0,
                enabled: true,
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
                    checksum: u32::from_ne_bytes(
                        header_data[CHECKSUM_OFFSET as usize..CHECKSUM_OFFSET as usize + 4]
                            .try_into()
                            .unwrap(),
                    ),
                    index,
                    enabled: header_data[ENABLED_OFFSET as usize] == 1,
                });
                // log::info!("found checksum {:?}", (installed_apps[index as usize].checksum).to_ne_bytes());
                // log::info!("changing checksum will result in {:?}", (installed_apps[index as usize].checksum - 1).to_ne_bytes());
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
                    app = match installed_apps
                        .iter()
                        .find(|iter| iter.name == app_name && iter.enabled)
                    {
                        Some(app) => app,
                        None => break,
                    }
                }
                None => loop {
                    app = inquire::Select::new(
                        "Which app do you want to disable?",
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
                        if !app.enabled {
                            println!("App is already disabled!");
                            break 'outer; // exit from the big loop, we don't have to do anything else
                        }
                        break;
                    }
                },
            }
            let mut loader = session.target().flash_loader();
            if app.index == 0 {
                // ALL
                // log::info!("ALL");
                for app_iter in installed_apps[1..].iter() {
                    // log::info!("checking app {}", app_iter.name);
                    if app_iter.enabled {
                        // log::info!("entered here????");
                        loader.add_data(app_iter.address + ENABLED_OFFSET, &[0x0])?;
                        loader.add_data(
                            app_iter.address + CHECKSUM_OFFSET,
                            &(app_iter.checksum - 1).to_ne_bytes(),
                        )?; // the checksum must be increased or we'll invalidate the header
                    }
                }
            } else {
                // only one
                loader.add_data(app.address + ENABLED_OFFSET, &[0x0])?;
                loader.add_data(
                    app.address + CHECKSUM_OFFSET,
                    &(app.checksum - 1).to_ne_bytes(),
                )?; // the checksum must be increased or we'll invalidate the header
            }
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
    checksum: u32,
    index: u8,
    enabled: bool,
}

impl std::fmt::Display for AppData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.name == "Disable all" {
            write!(f, "{}", self.name)
        } else {
            write!(
                f,
                "{}. {} - start: {:#x}, size: {}, enabled: {}",
                self.index, self.name, self.address, self.size, self.enabled
            )
        }
    }
}

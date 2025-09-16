use async_trait::async_trait;
use probe_rs::flashing::DownloadOptions;
use probe_rs::MemoryInterface;
use tbf_parser::parse::{parse_tbf_header, parse_tbf_header_lengths};

use crate::attributes::app_attributes::AppAttributes;
use crate::board_settings::BoardSettings;
use crate::connection::{Connection, ProbeRSConnection};
use crate::errors::{InternalError, TockloaderError};
use crate::CommandUninstall;

#[async_trait]
impl CommandUninstall for ProbeRSConnection {
    async fn uninstall_app(
        &mut self,
        settings: &BoardSettings,
        installed_apps: &Vec<AppAttributes>,
        app: &AppAttributes,
    ) -> Result<(), TockloaderError> {
        if !self.is_open() {
            return Err(InternalError::ConnectionNotOpen.into());
        }
        let session = self.session.as_mut().expect("Board must be open");

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
        Ok(())
    }
}

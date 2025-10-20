use async_trait::async_trait;
use probe_rs::{flashing::DownloadOptions, MemoryInterface};

use crate::{
    attributes::{app_attributes::AppAttributes, system_attributes::SystemAttributes},
    board_settings::BoardSettings,
    connection::{Connection, ProbeRSConnection},
    errors::{InternalError, TockloaderError},
    IOCommands,
};

#[async_trait]
impl IOCommands for ProbeRSConnection {
    async fn read(&mut self, address: u64, size: usize) -> Result<Vec<u8>, TockloaderError> {
        if !self.is_open() {
            return Err(InternalError::ConnectionNotOpen.into());
        }
        let session = self.session.as_mut().expect("Board must be open");

        let mut core = session.core(self.target_info.core)?;
        let mut appdata = vec![0u8; size];
        core.read(address, &mut appdata)?;
        Ok(appdata)
    }

    async fn write(&mut self, address: u64, pkt: Vec<u8>) -> Result<(), TockloaderError> {
        if !self.is_open() {
            return Err(InternalError::ConnectionNotOpen.into());
        }
        let session = self.session.as_mut().expect("Board must be open");
        let mut loader = session.target().flash_loader();

        loader.add_data(address, &pkt)?;

        let mut options = DownloadOptions::default();
        options.keep_unwritten_bytes = true;

        loader.commit(session, options)?;
        Ok(())
    }

    async fn list_apps(
        &mut self,
        settings: &BoardSettings,
    ) -> Result<Vec<AppAttributes>, TockloaderError> {
        if !self.is_open() {
            return Err(InternalError::ConnectionNotOpen.into());
        }
        let session = self.session.as_mut().expect("Board must be open");
        let mut core = session.core(self.target_info.core)?;

        AppAttributes::read_apps_data_probe(&mut core, settings.start_address)
    }

    async fn read_system_attributes(&mut self) -> Result<SystemAttributes, TockloaderError> {
        if !self.is_open() {
            return Err(InternalError::ConnectionNotOpen.into());
        }
        let session = self.session.as_mut().expect("Board must be open");

        let mut core = session.core(self.target_info.core)?;

        // TODO(george-cosma): extract these informations without bootloader
        let system_attributes = SystemAttributes::read_system_attributes_probe(&mut core)?;
        Ok(system_attributes)
    }
}

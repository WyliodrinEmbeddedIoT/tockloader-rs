use async_trait::async_trait;
use probe_rs::flashing::DownloadOptions;

use crate::board_settings::BoardSettings;
use crate::connection::{Connection, ProbeRSConnection};
use crate::errors::{InternalError, TockloaderError};
use crate::CommandEraseApps;

#[async_trait]
impl CommandEraseApps for ProbeRSConnection {
    async fn erase_apps(&mut self, settings: &BoardSettings) -> Result<(), TockloaderError> {
        if !self.is_open() {
            return Err(InternalError::ConnectionNotOpen.into());
        }
        let session = self.session.as_mut().expect("Board must be open");

        let mut loader = session.target().flash_loader();

        let address = settings.start_address;
        // A single 0x0 byte is enough to invalidate the tbf header and make it all programs
        // unreadable to tockloader. This does mean app information will still exist on the board,
        // but they will be overwritten when the space is needed.
        loader.add_data((address as u32).into(), &[0x0])?;

        let mut options = DownloadOptions::default();
        options.keep_unwritten_bytes = true;

        // Finally, the data can be programmed
        loader.commit(session, options)?;
        Ok(())
    }
}

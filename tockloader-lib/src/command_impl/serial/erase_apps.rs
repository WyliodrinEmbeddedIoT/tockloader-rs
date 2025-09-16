use async_trait::async_trait;

use crate::board_settings::BoardSettings;
use crate::bootloader_serial::{
    issue_command, ping_bootloader_and_wait_for_response, Command, Response,
};
use crate::connection::{Connection, SerialConnection};
use crate::errors::{InternalError, TockloaderError};
use crate::CommandEraseApps;

#[async_trait]
impl CommandEraseApps for SerialConnection {
    async fn erase_apps(&mut self, settings: &BoardSettings) -> Result<(), TockloaderError> {
        if !self.is_open() {
            return Err(InternalError::ConnectionNotOpen.into());
        }
        let stream = self.stream.as_mut().expect("Board must be open");

        ping_bootloader_and_wait_for_response(stream).await?;

        let pkt = (settings.start_address as u32).to_le_bytes().to_vec();
        let (_, _) = issue_command(stream, Command::ErasePage, pkt, true, 0, Response::OK).await?;
        Ok(())
    }
}

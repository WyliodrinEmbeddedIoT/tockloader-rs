use async_trait::async_trait;

use crate::attributes::app_attributes::AppAttributes;
use crate::board_settings::BoardSettings;
use crate::bootloader_serial::ping_bootloader_and_wait_for_response;
use crate::connection::{Connection, SerialConnection};
use crate::errors::{InternalError, TockloaderError};
use crate::CommandList;

#[async_trait]
impl CommandList for SerialConnection {
    async fn list(
        &mut self,
        settings: &BoardSettings,
    ) -> Result<Vec<AppAttributes>, TockloaderError> {
        if !self.is_open() {
            return Err(InternalError::ConnectionNotOpen.into());
        }
        let stream = self.stream.as_mut().expect("Board must be open");

        ping_bootloader_and_wait_for_response(stream).await?;

        AppAttributes::read_apps_data_serial(stream, settings.start_address).await
    }
}

use async_trait::async_trait;

use crate::board_settings::BoardSettings;
use crate::connection::TockloaderConnection;
use crate::errors::TockloaderError;
use crate::{CommandEraseApps, IO};

#[async_trait]
impl CommandEraseApps for TockloaderConnection {
    async fn erase_apps(&mut self, settings: &BoardSettings) -> Result<(), TockloaderError> {
        self.write(settings.start_address, vec![0u8], settings)
            .await
    }
}

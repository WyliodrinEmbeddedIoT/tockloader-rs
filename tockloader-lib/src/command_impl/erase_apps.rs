use async_trait::async_trait;

use crate::board_settings::BoardSettings;
use crate::connection::TockloaderConnection;
use crate::errors::TockloaderError;
use crate::{CommandEraseApps, IOCommands};

#[async_trait]
impl CommandEraseApps for TockloaderConnection {
    async fn erase_apps(&mut self, settings: &BoardSettings) -> Result<(), TockloaderError> {
        self.write(settings.start_address, [0x0].to_vec()).await
    }
}

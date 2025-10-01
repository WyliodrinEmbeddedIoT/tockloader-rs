use crate::board_settings::BoardSettings;
use crate::connection::TockloaderConnection;
use crate::errors::TockloaderError;

pub async fn reorder_apps(
    connection: &mut TockloaderConnection,
    settings: &BoardSettings,
    applications: &[()],
) -> Result<(), TockloaderError> {
    todo!();
}

use async_trait::async_trait;

use crate::attributes::app_attributes::AppAttributes;
use crate::attributes::general_attributes::GeneralAttributes;
use crate::board_settings::BoardSettings;
use crate::connection::TockloaderConnection;
use crate::errors::TockloaderError;
use crate::tabs::tab::Tab;
use crate::{
    CommandEraseApps, CommandInfo, CommandInstall, CommandList, ReadWrite, UtilityReshuffleApps,
};

#[async_trait]
impl ReadWrite for TockloaderConnection {
    async fn read(&mut self, address: u64, data: &mut [u8]) -> Result<(), TockloaderError> {
        match self {
            TockloaderConnection::ProbeRS(conn) => conn.read(address, data).await,
            TockloaderConnection::Serial(conn) => conn.read(address, data).await,
        }
    }

    async fn write(&mut self, address: u64, data: &[u8]) -> Result<(), TockloaderError> {
        match self {
            TockloaderConnection::ProbeRS(conn) => conn.write(address, data).await,
            TockloaderConnection::Serial(conn) => conn.write(address, data).await,
        }
    }
}

#[async_trait]
impl CommandList for TockloaderConnection {
    async fn list(
        &mut self,
        settings: &BoardSettings,
    ) -> Result<Vec<AppAttributes>, TockloaderError> {
        match self {
            TockloaderConnection::ProbeRS(conn) => conn.list(settings).await,
            TockloaderConnection::Serial(conn) => conn.list(settings).await,
        }
    }
}

#[async_trait]
impl CommandInfo for TockloaderConnection {
    async fn info(
        &mut self,
        settings: &BoardSettings,
    ) -> Result<GeneralAttributes, TockloaderError> {
        match self {
            TockloaderConnection::ProbeRS(conn) => conn.info(settings).await,
            TockloaderConnection::Serial(conn) => conn.info(settings).await,
        }
    }
}

#[async_trait]
impl CommandInstall for TockloaderConnection {
    async fn install_app(
        &mut self,
        settings: &BoardSettings,
        tab_file: Tab,
    ) -> Result<(), TockloaderError> {
        match self {
            TockloaderConnection::ProbeRS(conn) => conn.install_app(settings, tab_file).await,
            TockloaderConnection::Serial(conn) => conn.install_app(settings, tab_file).await,
        }
    }
}

#[async_trait]
impl CommandEraseApps for TockloaderConnection {
    async fn erase_apps(&mut self, settings: &BoardSettings) -> Result<(), TockloaderError> {
        match self {
            TockloaderConnection::ProbeRS(conn) => conn.erase_apps(settings).await,
            TockloaderConnection::Serial(conn) => conn.erase_apps(settings).await,
        }
    }
}

#[async_trait]
impl UtilityReshuffleApps for TockloaderConnection {
    async fn reshuffle_apps(
        &mut self,
        settings: &BoardSettings,
        tab: Option<Tab>,
    ) -> Result<(), TockloaderError> {
        match self {
            TockloaderConnection::ProbeRS(conn) => conn.reshuffle_apps(settings, tab).await,
            TockloaderConnection::Serial(_conn) => todo!(),
        }
    }
}

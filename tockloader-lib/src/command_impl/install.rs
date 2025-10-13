use async_trait::async_trait;

use crate::attributes::app_attributes::AppAttributes;
use crate::board_settings::BoardSettings;
use crate::command_impl::reshuffle_apps::{create_pkt, reconstruct_app, reshuffle_apps};
use crate::connection::TockloaderConnection;
use crate::errors::TockloaderError;
use crate::tabs::tab::Tab;
use crate::{CommandInstall, CommandList, IOCommands};

#[async_trait]
impl CommandInstall for TockloaderConnection {
    async fn install_app(
        &mut self,
        settings: &BoardSettings,
        tab: Tab,
    ) -> Result<(), TockloaderError> {
        // get the already installed apps
        let mut installed_apps: Vec<AppAttributes> = self.list(settings).await.unwrap();

        // reconstruct the new app
        if let Some(mut app) = reconstruct_app(Some(&tab), settings) {
            app.index = installed_apps.len() as u8;
            installed_apps.push(app.clone());
        }

        // obtain the binaries in a vector
        let mut app_binaries: Vec<Vec<u8>> = Vec::new();

        for app in installed_apps.iter() {
            match app.installed {
                true => {
                    app_binaries.push(app.clone().read(self).await.unwrap());
                }
                false => {
                    // TODO(adi): change this when TBF Filtering will get merged
                    app_binaries.push(
                        tab.extract_binary(settings.arch.as_ref().unwrap().as_str())
                            .unwrap(),
                    );
                }
            }
        }
        let configuration = reshuffle_apps(settings, installed_apps).unwrap();

        // create the pkt, this contains all the binaries in a vec
        let pkt = create_pkt(configuration, app_binaries);

        // write the pkt
        let _ = self.write(settings.start_address, pkt).await;
        Ok(())
    }
}

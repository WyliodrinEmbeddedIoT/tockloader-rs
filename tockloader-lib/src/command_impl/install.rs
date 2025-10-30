use async_trait::async_trait;

use crate::attributes::app_attributes::AppAttributes;
use crate::board_settings::BoardSettings;
use crate::command_impl::reshuffle_apps::{create_pkt, reshuffle_apps, TockApp};
use crate::connection::TockloaderConnection;
use crate::errors::TockloaderError;
use crate::tabs::tab::Tab;
use crate::{CommandInstall, CommandList, IO};

#[async_trait]
impl CommandInstall for TockloaderConnection {
    async fn install_app(
        &mut self,
        settings: &BoardSettings,
        tab: Tab,
    ) -> Result<(), TockloaderError> {
        let app_attributes_list: Vec<AppAttributes> = self.list(settings).await.unwrap();
        let mut tock_app_list = app_attributes_list
            .iter()
            .map(|app| TockApp::from_app_attributes(app, settings))
            .collect::<Vec<TockApp>>();

        // obtain the binaries in a vector
        let mut app_binaries: Vec<Vec<u8>> = Vec::new();

        for app in tock_app_list.iter() {
            app_binaries.push(app.clone().read_binary(self).await.unwrap())
        }

        let mut app = TockApp::from_tab(&tab, settings).unwrap();

        app.replace_idx(tock_app_list.len());
        tock_app_list.push(app.clone());

        app_binaries.push(
            tab.extract_binary(settings.arch.as_ref().unwrap().as_str())
                .unwrap(),
        );

        let configuration = reshuffle_apps(settings, tock_app_list).unwrap();

        // create the pkt, this contains all the binaries in a vec
        let pkt = create_pkt(configuration, app_binaries);

        // write the pkt
        let _ = self.write(settings.start_address, pkt, settings).await;
        Ok(())
    }
}

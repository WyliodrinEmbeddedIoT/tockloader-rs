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
            .map(TockApp::from_app_attributes)
            .collect::<Vec<TockApp>>();

        // obtain the binaries in a vector
        let mut app_binaries: Vec<Vec<u8>> = Vec::new();

        let mut address = settings.start_address;
        for app in app_attributes_list.iter() {
            app_binaries.push(
                self.read(address, app.tbf_header.total_size() as usize)
                    .await
                    .unwrap(),
            );
            address += app.tbf_header.total_size() as u64;
        }

        let mut app = TockApp::from_tab(&tab, settings).unwrap();

        app.replace_idx(tock_app_list.len());
        tock_app_list.push(app.clone());

        app_binaries.push(tab.extract_binary(settings.arch.clone().unwrap()).unwrap());

        let configuration = reshuffle_apps(settings, tock_app_list).unwrap();

        // create the pkt, this contains all the binaries in a vec
        let pkt = create_pkt(configuration, app_binaries);

        // write the pkt
        let _ = self.write(settings.start_address, pkt, settings).await;
        Ok(())
    }
}

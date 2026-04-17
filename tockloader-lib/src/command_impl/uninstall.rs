use async_trait::async_trait;

use crate::{
    attributes::app_attributes::AppAttributes,
    command_impl::reshuffle_apps::{create_pkt, reshuffle_apps, TockApp},
    connection::{Connection, TockloaderConnection},
    errors::{InternalError, TockloaderError},
    CommandEraseApps, CommandList, CommandUninstall, IO,
};

#[async_trait]
impl CommandUninstall for TockloaderConnection {
    async fn uninstall_app(
        &mut self,
        app_name: Option<String>,
        app_index: Option<usize>,
    ) -> Result<(), TockloaderError> {
        let settings = self.get_settings();

        let mut app_attributes_list: Vec<AppAttributes> = self.list().await?;

        // Remove all apps with given name
        if let Some(name) = app_name {
            let _ = app_attributes_list
                .retain(|app| app.tbf_header.get_package_name().unwrap_or("") != name);
        } else if let Some(index) = app_index {
            // Delete all apps, call erase
            if index == 0 {
                self.erase_apps().await?;
                return Ok(());
            }
            // Remove the selected index
            app_attributes_list.remove(index - 1);
        } else {
            panic!("Called uninstall with wrong parameters!");
        }

        let tock_app_list = app_attributes_list
            .iter()
            .map(TockApp::from_app_attributes)
            .collect::<Vec<TockApp>>();

        // obtain the binaries in a vector
        let mut app_binaries: Vec<Vec<u8>> = Vec::new();

        for app in app_attributes_list.iter() {
            let address = app.address;
            app_binaries.push(
                self.read(address, app.tbf_header.total_size() as usize)
                    .await
                    .unwrap(),
            );
        }

        let configuration =
            reshuffle_apps(&settings, tock_app_list).ok_or(TockloaderError::Internal(
                InternalError::MisconfiguredBoardSettings("Can't fit new app".to_string()),
            ))?;

        // create the pkt, this contains all the binaries in a vec
        let mut pkt = create_pkt(configuration, app_binaries, None, &settings);

        pkt.append(&mut [0u8; 512].to_vec());

        log::debug!("pkt len {}", pkt.len());
        // write the pkt
        let _ = self.write(settings.start_address, &pkt).await?;
        Ok(())
    }
}

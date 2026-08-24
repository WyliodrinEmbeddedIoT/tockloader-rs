use regression_test::RegTest;
use regression_test_macros::regtest;
use std::env;
use tockloader_lib::{
    connection::{Connection, ProbeRSConnection, TockloaderConnection},
    known_boards::{KnownBoard, Nrf52840dk},
    list_debug_probes, CommandList,
};

#[regtest]
#[tokio::test]
#[ignore = "requires a connected nRF52840 debug probe"]
pub async fn test_list(mut rt: RegTest) {
    let debug_probes = list_debug_probes();

    dbg!(&debug_probes);
    let selected_probe = if let Ok(selector) = env::var("HILLTOP_PROBE_SELECTOR") {
        let mut parts = selector.splitn(3, ':');
        let vendor_id = parts
            .next()
            .and_then(|value| u16::from_str_radix(value.trim_start_matches("0x"), 16).ok());
        let product_id = parts
            .next()
            .and_then(|value| u16::from_str_radix(value.trim_start_matches("0x"), 16).ok());
        let serial_number = parts.next();

        debug_probes
            .iter()
            .find(|probe| {
                vendor_id == Some(probe.vendor_id)
                    && product_id == Some(probe.product_id)
                    && serial_number == probe.serial_number.as_deref()
            })
            .unwrap_or_else(|| panic!("No debug probe matches selector '{selector}'"))
    } else {
        assert_eq!(
            debug_probes.len(),
            1,
            "Expected exactly one debug probe, or set HILLTOP_PROBE_SELECTOR; found {}.",
            debug_probes.len()
        );
        &debug_probes[0]
    };

    let board = Nrf52840dk;

    let mut conn: TockloaderConnection = ProbeRSConnection::new(
        selected_probe.clone(),
        board.probe_target_info(),
        board.get_settings(),
    )
    .into();

    conn.open().await.expect("Failed to open connection.");

    let apps = conn.list().await.expect("Failed to list apps.");

    assert!(
        apps.len() == 1,
        "Expected exactly one app to be installed, but found {}.",
        apps.len()
    );

    assert_eq!(
        apps[0].tbf_header.get_package_name(),
        Some("c_hello"),
        "Expected the installed app to be 'c_hello', but found '{:?}'.",
        apps[0].tbf_header.get_package_name()
    );

    rt.regtest_dbg(apps);

    conn.close().await.expect("Failed to close connection.");
}

use abi_stable::std_types::{ROption, RString, RVec};
use anyrun_plugin::*;
use nmrs::{DeviceState, Network, NetworkManager, WifiDevice};

struct NMDetails {
    networks: Vec<Network>,
    devices: Vec<WifiDevice>,
}

#[init]
fn init(config_dir: RString) -> NMDetails {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        let nm = NetworkManager::new()
            .await
            .expect("network manager not found");
        let networks = nm
            .list_networks(None)
            .await
            .expect("network manager not found");
        let devices = nm
            .list_wifi_devices()
            .await
            .expect("network manager not found");

        NMDetails { networks, devices }
    })
}

#[info]
fn info() -> PluginInfo {
    PluginInfo {
        name: "Network Manager".into(),
        icon: "nm-device-wireless-symbolic".into(),
    }
}

#[get_matches]
fn get_matches(input: RString, nm: &NMDetails) -> RVec<Match> {
    let mut matches: RVec<Match> = RVec::new();

    let input = if let Some(input) = input.strip_prefix(":nm") {
        input.trim()
    } else {
        return RVec::new();
    };

    matches.extend(
        nm.networks
            .iter()
            .filter(|network| network.ssid.contains(&input.to_string()))
            .map(|network| Match {
                title: network.ssid.clone().into(),
                icon: ROption::RNone,
                use_pango: false,
                description: ROption::RSome(
                    format!(
                        "{} network",
                        if network.is_active {
                            "disconnect from"
                        } else {
                            "connect to"
                        }
                    )
                    .into(),
                ),
                id: ROption::RSome(0),
            }),
    );
    matches.extend(
        nm.devices
            .iter()
            .filter(|device| device.interface.contains(&input.to_string()))
            .map(|device| Match {
                title: device.interface.clone().into(),
                icon: ROption::RNone,
                use_pango: false,
                description: ROption::RSome(
                        match device.state {
                            DeviceState::Unmanaged => "NOT MANAGED",
                            DeviceState::Unavailable => "toggle device on",
                            DeviceState::Disconnected => "toggle device off",
                            DeviceState::Prepare => "toggle device off",
                            DeviceState::Activated => "toggle device off",
                            DeviceState::IpConfig => "CONFIG REQUIRED",
                            DeviceState::NeedAuth => "CONFIG REQUIRED",
                            DeviceState::IpCheck => "device busy",
                            DeviceState::Deactivating => "device busy",
                            _ => ""
                        }
                    .into(),
                ),
                id: ROption::RSome(1),
            }),
    );

    matches
}

#[handler]
fn handler(selection: Match) -> HandleResult {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        let nm = NetworkManager::new()
            .await
            .expect("network manager not found");

        match selection.id {
            ROption::RSome(0) => {
                if nm
                    .is_connected(&selection.title)
                    .await
                    .expect("Failed to identify connection")
                {
                    nm.disconnect(Some(&selection.title))
                        .await
                        .expect("Unable to disconnect");
                } else {
                    nm.connect(&selection.title, None, nmrs::WifiSecurity::Open)
                        .await
                        .expect("Unsaved Connection");
                }
            }
            ROption::RSome(1) => {
                let device = nm
                    .wifi_device_by_interface(&selection.title)
                    .await
                    .expect("Interface Not Found");

                match device.state {
                    DeviceState::Unmanaged => {},
                    DeviceState::Unavailable => {nm.set_wireless_enabled(true).await.expect("Unable to toggle interface");},
                    DeviceState::Disconnected => {nm.set_wireless_enabled(false).await.expect("Unable to toggle interface");},
                    DeviceState::Prepare => {nm.set_wireless_enabled(false).await.expect("Unable to toggle interface");},
                    DeviceState::Activated => {nm.set_wireless_enabled(false).await.expect("Unable to toggle interface");},
                    DeviceState::IpConfig => {},
                    DeviceState::NeedAuth => {},
                    DeviceState::IpCheck => {},
                    DeviceState::Deactivating => {},
                    _ => {}
                };
            }
            ROption::RNone => {}
            ROption::RSome(2_u64..=u64::MAX) => {}
        }
    });
    // Handle the selected match and return how anyrun should proceed
    HandleResult::Close
}

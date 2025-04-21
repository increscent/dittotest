use clap::Parser;
use dittolive_ditto::prelude::*;
use dittolive_ditto::store::dql::QueryResult;
use serde_json::json;
use std::io::Write;
use std::str::FromStr;
use std::sync::Arc;
use tokio::io;
use tokio::io::AsyncBufReadExt;

mod constants;

#[derive(Parser, Debug)]
#[clap(
    author = "Ditto - Various",
    version = "1.0",
    about = "DittoTest Application"
)]
struct Args {
    #[clap(long, help = "Enable debug logging")]
    debug: bool,

    #[clap(long, help = "Enable or disable cloud sync")]
    cloud_sync: bool,

    #[clap(long, help = "Use shared key authentication")]
    shared_key: bool,

    #[clap(long, help = "Use offline playground authentication")]
    offline_playground: bool,

    #[clap(long, value_name = "URL", help = "Set a custom authentication URL")]
    custom_auth_url: Option<String>,

    #[clap(long, help = "Enable P2P BLE communication")]
    p2p_ble_enabled: bool,

    #[clap(long, help = "Enable P2P LAN communication")]
    p2p_lan_enabled: bool,

    #[clap(long, help = "TCP Port to Listen")]
    tcp_listen_port: Option<u16>,

    #[clap(long, help = "TCP Port to Connect")]
    tcp_connect_port: Option<u16>,

    #[clap(long, help = "No stdin (for running in background)")]
    no_stdin: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app_id = AppId::from_str(constants::APP_ID)?;
    let auth_token = String::from(constants::PLAYGROUND_AUTH_TOKEN);

    let args = Args::parse();

    let logging_level = if args.debug {
        LogLevel::Debug
    } else {
        LogLevel::Info
    };
    let cloud_sync = args.cloud_sync;
    let custom_auth_url = args.custom_auth_url.as_deref();
    let p2p_ble_enabled = args.p2p_ble_enabled;
    let p2p_lan_enabled = args.p2p_lan_enabled;

    // Initialize Ditto
    let mut ditto = Ditto::builder()
        .with_root(Arc::new(PersistentRoot::from_current_exe()?))
        .with_minimum_log_level(logging_level);

    if args.shared_key {
        ditto = ditto.with_identity(|ditto_root| {
            identity::SharedKey::new(
                ditto_root,
                app_id,
                constants::SHARED_KEY
                    .expect("SHARED_KEY constant is required when using the --shared-key option"),
            )
        })?;
    } else if args.offline_playground {
        ditto = ditto
            .with_identity(|ditto_root| identity::OfflinePlayground::new(ditto_root, app_id))?;
    } else {
        ditto = ditto.with_identity(|ditto_root| {
            identity::OnlinePlayground::new(
                ditto_root,
                app_id,
                auth_token,
                cloud_sync,
                custom_auth_url,
            )
        })?;
    }

    let ditto = ditto
        .with_transport_config(|_identity| -> TransportConfig {
            let mut transport_config = TransportConfig::new();
            transport_config.peer_to_peer.bluetooth_le.enabled = p2p_ble_enabled;
            transport_config.peer_to_peer.lan.enabled = p2p_lan_enabled;
            if let Some(port) = args.tcp_listen_port {
                transport_config.listen.tcp.enabled = true;
                transport_config.listen.tcp.interface_ip = "127.0.0.1".to_string();
                transport_config.listen.tcp.port = port;
            }
            if let Some(port) = args.tcp_connect_port {
                transport_config
                    .connect
                    .tcp_servers
                    .insert(format!("127.0.0.1:{port}"));
            }
            println!("{transport_config:?}");
            transport_config
        })?
        .build()?;

    constants::DITTO_LICENSE.map(|license| {
        ditto
            .set_offline_only_license_token(license)
            .expect("Valid offline ditto license required if using --shared-key option")
    });

    ditto.disable_sync_with_v3().unwrap();

    // Start syncing with peers
    ditto.start_sync()?;

    let _observer =
        ditto
            .store()
            .register_observer("SELECT * from wats", None, move |result: QueryResult| {
                println!("count: {}", result.item_count());
            });

    let _subscription = ditto
        .sync()
        .register_subscription("SELECT * FROM wats", None);

    if args.no_stdin {
        loop {}
    }

    let stdin = io::stdin();
    let reader = io::BufReader::new(stdin);
    let mut lines = reader.lines();

    let mut connect_to: Option<String> = None;
    if let Some(port) = args.tcp_connect_port {
        connect_to = Some(format!("127.0.0.1:{port}"));
    }

    let mut i = 0;

    print!("Press enter to increment:");
    std::io::stdout().flush().unwrap();
    while let Some(_) = lines.next_line().await? {
        ditto
            .store()
            .execute(
                "INSERT INTO wats DOCUMENTS (:newWat)",
                Some(
                    json!({
                        "newWat": {
                            "color": "blue"
                        }
                    })
                    .into(),
                ),
            )
            .await?;

        print!("Press enter to increment:");
        std::io::stdout().flush().unwrap();

        i += 1;
        if i % 5 == 0 {
            if i % 10 == 0 {
                ditto.update_transport_config(|config| {
                    config.connect.tcp_servers.clear();
                });
            } else {
                if let Some(ref host) = connect_to {
                    ditto.update_transport_config(|config| {
                        config.connect.tcp_servers.insert(host.to_string());
                    });
                }
            }
        }
    }

    Ok(())
}

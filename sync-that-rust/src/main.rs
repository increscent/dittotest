use clap::Parser;
use dittolive_ditto::experimental::bus::Reliability;
use dittolive_ditto::experimental::peer_pubkey::PeerPubkey;
use dittolive_ditto::prelude::*;
use dittolive_ditto::store::dql::QueryResult;
use serde_json::json;
use std::fs::{File, read_to_string};
use std::io::Write;
use std::str::FromStr;
use std::sync::Arc;
use tokio::io;
use tokio::io::AsyncBufReadExt;
use tokio::time::{sleep, Duration};

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

    #[clap(long, help = "Use Ditto bus")]
    ditto_bus: bool,

    #[clap(long, help = "Write peer key to file")]
    save_peer_key: Option<String>,

    #[clap(long, help = "Read bus peer key to file")]
    read_bus_peer_key: Option<String>,

    #[clap(long, help = "Peer key of peer to connect to on Ditto bus")]
    bus_peer_key: Option<String>,
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

    if let Some(path) = args.save_peer_key {
        let peer_key_string = ditto.presence().graph().local_peer.peer_key_string;
        println!("Saving peer key: {peer_key_string} ({path})");
        let mut output = File::create(path)?;
        write!(output, "{peer_key_string}")?;
    }

    let mut target_peer_key_string = args.bus_peer_key;

    if let Some(path) = args.read_bus_peer_key {
        target_peer_key_string = Some(read_to_string(path)?);
    }

    if args.ditto_bus {
        if let Some(peer_key_string) = target_peer_key_string {
            // Client
            let peer = PeerPubkey::from_str(&peer_key_string).unwrap();

            sleep(Duration::from_secs(5)).await;
            let stream = ditto
                .bus()
                .connect(peer.clone(), "wat")
                .reliability(Reliability::Reliable)
                .on_receive_factory(tokio::sync::mpsc::unbounded_channel)
                .finish_async()
                .await;
            println!("{stream:?}");
            tokio::spawn(async move {
                if let Ok(mut stream) = stream {
                    loop {
                        let _ = stream.tx().message("hi").try_send().unwrap();
                        let x = stream.recv().await.expect("stream is open");
                        println!("{x:?}");
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            });
        } else {
            // Server
            let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
            let handle = ditto
                .bus()
                .bind_topic("wat")
                .reliability(Reliability::Reliable)
                .on_receive_factory(tokio::sync::mpsc::unbounded_channel)
                .finish((sender, receiver));

            let mut stream = handle.unwrap();
            if let Some(mut stream) = stream.recv().await {
                let remote = stream.peer_pubkey();
                println!("New stream opened by remote peer {remote:?}");
                loop {
                    let _ = stream.tx().message("hello").try_send().unwrap();
                    let x = stream.recv().await.expect("stream is open");
                    println!("{x:?}");
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
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

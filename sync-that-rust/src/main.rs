
use dittolive_ditto::prelude::*;
use dittolive_ditto::store::dql::QueryResult;
use serde_json::json;
use std::io::Write;
use std::str::FromStr;
use std::sync::Arc;
use tokio::io::AsyncBufReadExt;
use tokio::io;

mod constants;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app_id = AppId::from_str(constants::APP_ID)?;
    let auth_token = String::from(constants::PLAYGROUND_AUTH_TOKEN);
    let cloud_sync = true;
    let custom_auth_url = None;

    // Initialize Ditto
    let ditto = Ditto::builder()
        .with_root(Arc::new(PersistentRoot::from_current_exe()?))
        .with_identity(|ditto_root| {
            identity::OnlinePlayground::new(
                ditto_root,
                app_id,
                auth_token,
                cloud_sync,
                custom_auth_url,
            )
        })?
        .build()?;

    // Start syncing with peers
    ditto.start_sync()?;

    let _observer = ditto.store().register_observer(
        "SELECT * from wats",
        None,
        move |result: QueryResult| {
            println!("count: {}", result.item_count());
        });

    let _subscription =
        ditto.sync().register_subscription("SELECT * FROM wats", None);

    let stdin = io::stdin();
    let reader = io::BufReader::new(stdin);
    let mut lines = reader.lines();

    print!("Press enter to increment:");
    std::io::stdout().flush().unwrap();
    while let Some(_) = lines.next_line().await? {
        ditto.store().execute(
            "INSERT INTO wats DOCUMENTS (:newWat)",
            Some(json!({
                "newWat": {
                    "color": "blue"
                }
            }).into()),
        ).await?;

        print!("Press enter to increment:");
        std::io::stdout().flush().unwrap();
    }

    Ok(())
}

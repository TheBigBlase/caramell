extern crate utils;

use rumqttc::v5::{AsyncClient, MqttOptions};
use std::time::Duration;
mod server_utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = utils::load_toml("caramell-server");
    let params = cfg.params.unwrap();
    let blockchain_p = cfg.blockchain.unwrap();

    let mut mqttoptions = MqttOptions::new(
        params.clone().id,
        params.clone().broker_ip,
        params.broker_port,
    );

    //remember unread msg
    mqttoptions
        .set_keep_alive(Duration::from_secs(60))
        .set_clean_start(true);

    let (client, eventloop) = AsyncClient::new(mqttoptions, 10);

    server_utils::init_broker_srvlist(client, params.clone(), blockchain_p)
        .await?;

    let mem_client = utils::MemcacheClient::new(
        String::from(params.cache_ip.unwrap()),
        params.cache_port.unwrap(),
        vec!["".to_string()],
    );

    // run forever
    let _handle = server_utils::serve_trust(eventloop, mem_client).await;
    Ok(())
}

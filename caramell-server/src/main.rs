extern crate utils;

use rumqttc::v5::{mqttbytes::QoS, AsyncClient, MqttOptions};
use std::time::Duration;
mod server_utils;

#[tokio::main]
async fn main() {
    let params = utils::load_toml("caramell-server").params.unwrap();
    let mut mqttoptions = MqttOptions::new(params.id, params.broker_ip, params.broker_port, );

    //remember unread msg
    mqttoptions
        .set_keep_alive(Duration::from_secs(60))
        .set_clean_start(true);

    let (client, eventloop) = AsyncClient::new(mqttoptions, 10);

    client
        .publish(
            format!("srvList/{}:{}", 1, params.cache_port.unwrap()),
            QoS::AtLeastOnce,
            true,
            params.cache_name.unwrap(),
        )
        .await
        .unwrap();

    let mem_client = utils::MemcacheClient::new(
        String::from(params.cache_ip.unwrap()),
        params.cache_port.unwrap(),
        vec!["".to_string()],
    );

    // run forever
    let _handle = server_utils::serve_trust(eventloop, mem_client).await;
}

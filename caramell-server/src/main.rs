extern crate utils;

use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let params = utils::load_toml("caramell-server").params.unwrap();
    let mut mqttoptions = MqttOptions::new(params.id, params.broker_ip, params.broker_port);
    //remember unread msg
    mqttoptions
        .set_keep_alive(Duration::from_secs(60))
        .set_clean_session(false);

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client.subscribe("msg/srv", QoS::AtLeastOnce).await.unwrap();

    let mem_client = utils::MemcacheClient::new(
        String::from(params.cache_ip.unwrap()),
        params.cache_port.unwrap(),
        vec!["".to_string()],
    );

    while let Ok(notification) = eventloop.poll().await {
        let res = utils::check_publish(notification, mem_client.clone());
        if res.is_err() {
            panic!("MemcacheError: {}", res.err().unwrap());
        }
    }
}

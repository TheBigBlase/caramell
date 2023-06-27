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
        .set_clean_session(true);

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    client
        .publish(
            format!(
                "srvList/{}:{}",
                1,
                params.cache_port.unwrap()
            ),
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

    while let Ok(notification) = eventloop.poll().await {
        let res = utils::check_publish(notification, mem_client.clone());
        match res {
            Err(utils::ErrorBrokerMemcached::MemcacheError(e)) => panic!("MemcacheError: {:?}", e),
            Ok(string) => {
                println!("{}", string)
            }
            _ => {}
        }
    }
}

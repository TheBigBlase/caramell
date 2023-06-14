extern crate utils;

use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::time::Duration;
use tokio::{task, time};


#[tokio::main]
async fn main() {
    let params: utils::Params = utils::load_toml("caramell-client").params.unwrap();

    let mut mqttoptions = MqttOptions::new(params.id.clone(), params.broker_ip.clone(), params.broker_port);
    mqttoptions.set_keep_alive(Duration::from_secs(10));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    task::spawn(async move {
        let mut i = 1;
        loop {
            if client
                .publish("msg/srv", QoS::AtLeastOnce, true, format!("MEM;key{};val{};0", i, i))
                .await
                .is_err() {
                    panic!("client error");
                };
            time::sleep(Duration::from_secs(1)).await;
            i+=1;
        }
    });

    while let Ok(notification) = eventloop.poll().await {
        println!("Received = {:?}", notification);
    }
}

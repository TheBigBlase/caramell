extern crate utils;

use rumqttc::v5::{Client, MqttOptions};
use utils::Broker;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let params: utils::Params = utils::load_toml("caramell-client").params.unwrap();

    let mut mqttoptions = MqttOptions::new(
        params.id.clone(),
        params.broker_ip.clone(),
        params.broker_port,
    );
    mqttoptions.set_keep_alive(Duration::from_secs(10));

    let (client, eventloop) = Client::new(mqttoptions, 10);

    let broker_list = utils::get_list_cacher_from_broker(client, eventloop)?;
    println!("broker list: {:?}", broker_list);

    let broker:&Broker = broker_list.first().unwrap();


    Ok(())
}

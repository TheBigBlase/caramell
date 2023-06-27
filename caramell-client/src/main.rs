extern crate utils;

use rumqttc::v5::{MqttOptions, Client};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let params: utils::Params = utils::load_toml("caramell-client").params.unwrap();

    let mut mqttoptions = MqttOptions::new(
        params.id.clone(),
        params.broker_ip.clone(),
        params.broker_port
    );
    mqttoptions.set_keep_alive(Duration::from_secs(10));

    let (client, eventloop) = Client::new(mqttoptions, 10);

    println!("broker list: {:?}", utils::get_list_cacher_from_broker(client, eventloop)?);


    Ok(())
 }

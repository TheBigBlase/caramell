use std::time::Duration;
//TODO use utils::blockchain for all ethers stuff


use rumqttc::v5::{MqttOptions, AsyncClient};
use utils::{Broker, Config};

pub async fn broker_list(cfg: Config) -> Result<Vec<Broker>, Box<dyn std::error::Error>> {
    let params: utils::Params = cfg.params.unwrap();

    let mut mqttoptions = MqttOptions::new(
        params.id.clone(),
        params.broker_ip.clone(),
        params.broker_port,
    );
    // useless, keeping it just in case :)
    mqttoptions.set_keep_alive(Duration::from_secs(60));

    let (client, eventloop) = AsyncClient::new(mqttoptions, 10);

    utils::get_list_cacher_from_broker(&client, eventloop).await
}


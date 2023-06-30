extern crate utils;

use rumqttc::v5::{Client, MqttOptions};
use std::time::Duration;
use tokio::runtime::Runtime;
use utils::blockchain;
use utils::Broker;
#[macro_use] extern crate rocket;

// TODO make a rest api :)

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    rocket::build().mount("/", routes![index]);

    let cfg = utils::load_toml("caramell-client");
    let params: utils::Params = cfg.params.unwrap();
    let rpc_url = cfg.blockchain.expect("Need blockchain block in cfg").rpc_url_http;

    let mut mqttoptions = MqttOptions::new(
        params.id.clone(),
        params.broker_ip.clone(),
        params.broker_port,
    );
    // useless, keeping it just in case :)
    mqttoptions.set_keep_alive(Duration::from_secs(60));

    let (client, eventloop) = Client::new(mqttoptions, 10);

    let broker_list = utils::get_list_cacher_from_broker(client, eventloop)?;
    println!("broker list: {:?}", broker_list);

    let broker: &Broker = broker_list.first().unwrap();

    let address = broker.address;

    let config = utils::load_toml("caramell-client");

    // "await" in a sync func:
    let rt = Runtime::new().unwrap();
    let promise_wallet = blockchain::create_wallet(config.clone());
    let wallet = rt.block_on(promise_wallet)?;

    let promise_contract = blockchain::get_client_contract_addr(config.clone(), Some(address), wallet.clone());
    let client_contract_addr = rt.block_on(promise_contract)?;
    
    println!("contract address: {:?}", client_contract_addr);

    let client_contract = blockchain::create_client(address, wallet, rpc_url.as_str())?;

    Ok(())
}

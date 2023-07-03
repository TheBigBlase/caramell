extern crate utils;

mod client_utils;
use rumqttc::v5::{Client, MqttOptions};
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::time::Duration;
use tokio::runtime::Runtime;
use utils::blockchain;
use utils::Broker;
use rocket::Config;
#[macro_use] extern crate rocket;

// TODO make a rest api :)

#[get("/brokerList")]
async fn broker_list() -> String {
    let crml_cfg = utils::load_toml("caramell-server");//TODO dont re read at every call
    let lst = client_utils::broker_list(crml_cfg).await.ok();
    serde_json::to_string(&lst.unwrap()).unwrap()
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
async fn rocket() -> _ {
    let mut rocket_cfg = Config::default();
    rocket_cfg.port = 8989;
    rocket_cfg.address =  "0.0.0.0".parse().unwrap(); //listens from everywhere

    let crml_cfg = utils::load_toml("caramell-client");
    client_utils::broker_list(crml_cfg).await.unwrap();
    rocket::build()
        .mount("/", routes![index])
        .mount("/", routes![broker_list])
        .configure(rocket_cfg)
}

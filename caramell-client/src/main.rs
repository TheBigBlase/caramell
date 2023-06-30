extern crate utils;

mod client_utils;
use rumqttc::v5::{Client, MqttOptions};
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
    let lst = client_utils::broker_list(crml_cfg).ok();
    let mut res = String::from("");
    let iter = lst.unwrap().into_iter();
    for k in iter {
        res.push_str(format!("{:?}", k).as_str());
    }

    res
}


#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    let mut rocket_cfg = Config::default();
    rocket_cfg.port = 8989;

    let crml_cfg = utils::load_toml("caramell-server");
    client_utils::broker_list(crml_cfg).unwrap();
    rocket::build()
        .mount("/", routes![index])
        .mount("/", routes![broker_list])
        .configure(rocket_cfg)
}

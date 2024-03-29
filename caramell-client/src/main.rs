extern crate utils;

mod client_utils;

use rumqttc::v5::AsyncClient;
use std::sync::{Arc, Mutex};
use tokio::sync::watch::Receiver;
use tokio::sync::watch::channel;

use rocket::fairing::{Fairing, Info, Kind};

use rocket::figment::Figment;
use rocket::http::Header;
use rocket::serde::json::Json;
use rocket::{Config, Response};
#[macro_use]
extern crate rocket;
use rocket::State;
use utils::blockchain::{create_wallet, init_contract};
use utils::contracts::client_contract::Data;
use utils::{Broker, H160};

#[get("/brokerList")]
async fn broker_list(
    state: &State<Arc<Mutex<Vec<Broker>>>>,
) -> Json<Vec<Broker>> {
    //let state_inner = state.inner().clone();
    //let cfg = state_inner.config.clone();
    //let cli = state_inner.client.clone();
    let vec = state.inner().lock().unwrap();
    Json(vec.clone())
}

// TODO read summuary and read data
#[get("/brk/<ip>/<port>/get/<name>")]
async fn read_server(
    cfg: &State<utils::Config>,
    client_mqtt: &State<AsyncClient>,
    brk_lst: &State<Arc<Mutex<Vec<Broker>>>>,
    rx: &State<Receiver<String>>,
    ip: String,
    port: String,
    name: String,
) -> Json<String> {
    let cfg = cfg.inner().clone();

    let mut brk: Broker = Broker::default();
    brk.ip = ip;
    brk.port = port.parse().unwrap();

    let slf_addr = format!(
        "{}:{}",
        cfg.clone().params.unwrap().self_ip.unwrap(),
        cfg.clone().params.unwrap().self_port.unwrap()
        );

    // TODO save file instead of printing it :)
    let res = client_utils::read_data(
        name.as_str(),
        brk,
        client_mqtt.inner().clone(),
        &slf_addr.to_string(),
        rx.inner().clone(),
    )
    .await;

    Json(res.unwrap_or(String::new()))
}

// TODO read summuary and read data
#[get("/ctrct/<ctrct>/get_summary/<name>")]
async fn get(
    cfg: &State<utils::Config>,
    brk_lst: &State<Arc<Mutex<Vec<Broker>>>>,
    ctrct: String,
    name: String,
) -> Json<Vec<Data>> {
    let cfg = cfg.inner().clone();
    let mut brk: Broker = Broker::default();
    let contract = ctrct.parse::<H160>().unwrap();

    {
        let brk_lst = brk_lst.lock().unwrap();
        let brk_lst = brk_lst.clone();
        let res = brk_lst.binary_search_by(|b| b.address.cmp(&contract));
        match res {
            Ok(idx) => brk = brk_lst.get(idx).unwrap().clone(),
            Err(_) => (),
        };
    }

    let client = init_contract(cfg, brk).await.unwrap();

    let res;

    if name == "all".to_string() {
        res = client_utils::retrieve_all_data_location(Arc::new(client))
            .await
            .ok()
    } else {
        let tmp =
            client_utils::retrieve_data_location(Arc::new(client), name).await;
        let mut v = Vec::new();
        v.push(tmp.unwrap());
        res = Some(v);
    }
    Json(res.unwrap_or(vec![Data::default()]))
}

// same TODO as above
#[get("/ctrct/<ctrct>/set/<name>")]
async fn set(
    cfg: &State<utils::Config>,
    brk_lst: &State<Arc<Mutex<Vec<Broker>>>>,
    ctrct: String,
    name: String,
) -> Json<Data> {
    let cfg = cfg.inner().clone();
    let blck = cfg.clone().blockchain.unwrap();
    let brk: Broker;

    {
        let brk_lst = brk_lst.lock().unwrap();
        brk = brk_lst.first().unwrap().clone();
    }

    let client = init_contract(cfg, brk).await.unwrap();

    //let res = client_utils::set_data(client, name).await.ok();
    Json(Data::default())
}

// leaving that there for testing
#[get("/helloWorld")]
fn index() -> &'static str {
    "Hello, world!"
}

// ty kind stackoverflow stranger (migepatschen) :)
/// Catches all OPTION requests in order to get the CORS related Fairing triggered.
#[options("/<_..>")]
fn all_options() {
    /* Intentionally left empty */
}

pub struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Cross-Origin-Resource-Sharing Fairing",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(
        &self,
        _request: &'r rocket::Request<'_>,
        response: &mut Response<'r>,
    ) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, PATCH, PUT, DELETE, HEAD, OPTIONS, GET",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Credentials",
            "true",
        ));
    }
}

#[launch]
async fn rocket() -> _ {
    let mut rocket_cfg = Config::default();

    rocket_cfg.port = 8989;
    rocket_cfg.address = "0.0.0.0".parse().unwrap(); //listens from everywhere
                                                     //rocket_cfg.workers = 4; //?

    let figment = Figment::from(rocket_cfg);

    let crml_cfg = utils::load_toml("caramell-client");

    let (client, evtloop) = client_utils::init_eventloop(crml_cfg.clone())
        .await
        .unwrap();

    let vec: Vec<Broker> = Vec::new();
    let vec = Arc::new(Mutex::new(vec)).clone();
    let vec_clone = vec.clone();

    let slf_addr = format!(
        "{}:{}",
        crml_cfg.clone().params.unwrap().self_ip.unwrap(),
        crml_cfg.clone().params.unwrap().self_port.unwrap(),
    );

    let (tx, rx) = channel::<String>("".to_string());


    tokio::spawn(async move {
        let _handle = client_utils::handle_eventloop(
            evtloop,
            vec_clone,
            slf_addr.as_str(),
            tx,
        )
        .await
        .unwrap();
    });
    utils::subscribe_all(client.clone()).await.unwrap();

    rocket::custom(figment)
        .attach(Cors)
        .manage(crml_cfg)
        .manage(vec)
        .manage(rx)
        .manage(client)
        .mount("/", routes![index, broker_list, all_options, get, set])
}

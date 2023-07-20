extern crate utils;

mod client_utils;

use std::sync::{Arc, Mutex};

use rocket::fairing::{Fairing, Info, Kind};

use rocket::figment::Figment;
use rocket::http::Header;
use rocket::serde::json::Json;
use rocket::{Config, Response};
#[macro_use]
extern crate rocket;
use rocket::State;
use utils::blockchain::{init_contract, create_wallet};
use utils::contracts::client_contract::Data;
use utils::Broker;

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
#[get("/ctrct/<ctrct>/get_summary/<name>")]
async fn get(
    cfg: &State<utils::Config>,
    brk_lst: &State<Arc<Mutex<Vec<Broker>>>>,
    ctrct: String,
    name: String,
) -> Json<Vec<Data>> {
    let cfg = cfg.inner().clone();
    let blck = cfg.clone().blockchain.unwrap();
    let url = blck.rpc_url_ws.as_str();
    let brk: Broker;

    {
        let brk_lst = brk_lst.lock().unwrap();
        brk = brk_lst.first().unwrap().clone();
    }

    let client = init_contract(cfg, brk)
        .await
        .unwrap();

    let res;

    if name == "all".to_string() {
        res = client_utils::retrieve_all_data_location(Arc::new(client)).await.ok()
    } else {
        let tmp = client_utils::retrieve_data_location(Arc::new(client), name).await;
        let mut v = Vec::new();
        v.push(tmp.unwrap());
        res = Some(v);
    }
    Json(res.unwrap_or(vec![Data::default()]))
}

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

    let client = init_contract(cfg, brk)
        .await
        .unwrap();

    //let res = client_utils::set_data(client, name).await.ok();
    Json(Data::default())
}

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
    // TODO share vec between threads
    let vec: Vec<Broker> = Vec::new();
    let vec = Arc::new(Mutex::new(vec)).clone();
    let vec_clone = vec.clone();

    tokio::spawn(async move {
        let _handle = client_utils::handle_eventloop(evtloop, vec_clone)
            .await
            .unwrap();
    });
    utils::subscribe_all(client).await.unwrap();

    rocket::custom(figment)
        .attach(Cors)
        .manage(crml_cfg)
        .manage(vec)
        .mount("/", routes![index, broker_list, all_options, get, set])
}

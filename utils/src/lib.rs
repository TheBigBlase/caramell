use bytes::Bytes;
use memcache;
pub use primitive_types::H160;
use rumqttc::v5::mqttbytes::QoS;
use rumqttc::v5::{Client, ClientError, Connection, Event, Incoming};
use rumqttc::{self};
use serde::Deserialize;

use std::fs::File;
use std::io::Read;
use std::str::FromStr;
use std::time::Duration;

pub mod blockchain;
pub mod contracts;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub params: Option<Params>,
    pub blockchain: Option<Blockchain>,
}

#[derive(Deserialize, Clone)]
pub struct Params {
    pub broker_ip: String,
    pub broker_port: u16,
    pub cache_ip: Option<String>,
    pub cache_port: Option<u16>,
    pub cache_name: Option<String>,
    pub id: String,
    pub contract_addr: Option<H160>,
    pub pub_key: Option<H160>,
}

#[derive(Deserialize, Clone)]
pub struct Blockchain {
    pub rpc_url_http: String,
    pub rpc_url_ws: String,
    pub wallet_key: String,
    pub contract_addr: H160,
    pub contract_unowned_addr: Option<H160>,
}

#[derive(Debug)]
pub enum ErrorBrokerMemcached {
    NotPublished,
    Outgoing,
    NotInsertion,
    MemcacheError(memcache::MemcacheError),
    BadBroker,
    ClientError(ClientError),
}

#[derive(Debug)]
pub struct Broker {
    pub ip: String,
    pub port: u16,
    pub address: H160,
}

#[derive(Clone)]
pub struct MemcacheClient {
    client: memcache::Client,
}

impl MemcacheClient {
    /// Create memcache wrapper. Probably the rapper is useless TODO
    pub fn new(ip: String, port: u16, option: Vec<String>) -> MemcacheClient {
        let options = option.join("&");
        let url = format!("memcache://{}:{}?{}", ip, port, options);
        let client = memcache::connect(url).unwrap();
        MemcacheClient { client }
    }

    /// insert key / value into memcached
    pub fn insert_memcached(
        &self,
        key: String,
        value: String,
        exp: u32,
    ) -> Result<String, ErrorBrokerMemcached> {
        match self.client.set(key.as_str(), value.clone(), exp) {
            Ok(_) => Ok(key + String::from(":").as_str() + value.as_str()),
            Err(e) => Err(ErrorBrokerMemcached::MemcacheError(e)),
        }
    }
}

/// check if mqtt has any unread message and try to insert them.
/// returns inserted string if any, or ""
pub fn check_publish(
    msg: rumqttc::v5::Event,
    mem_client: MemcacheClient,
) -> Result<String, ErrorBrokerMemcached> {
    match msg {
        rumqttc::v5::Event::Outgoing(notif) => {
            println!("Outgoing {:?}", notif);
            Err(ErrorBrokerMemcached::Outgoing)
        }
        rumqttc::v5::Event::Incoming(notif) => match notif {
            rumqttc::v5::mqttbytes::v5::Packet::Publish(n) => {
                println!("{:?}", n.payload);
                check_mem(n.payload, mem_client)
            }
            _ => Err(ErrorBrokerMemcached::NotPublished),
        },
    }
}

/// insert msg in memcached if start w/ MEM
/// returns the inserted string, or an error
pub fn check_mem(msg: Bytes, mem_client: MemcacheClient) -> Result<String, ErrorBrokerMemcached> {
    if msg.starts_with("MEM".as_bytes()) {
        let tmp = String::from_utf8(msg.to_vec()).unwrap().to_string();
        let mut tmp = tmp.split(";");
        tmp.next();

        mem_client.insert_memcached(
            tmp.next().unwrap().to_string(),
            tmp.next().unwrap().to_string(),
            tmp.next().unwrap().to_string().parse().unwrap(),
        )
    } else {
        Err(ErrorBrokerMemcached::NotInsertion)
    }
}

/// Generic loading toml file. Returns generic config struct.
pub fn load_toml(path: &str) -> Config {
    let mut cargo_text = String::new();
    File::open(format!("{}/Cargo.toml", path.to_string()))
        .unwrap()
        .read_to_string(&mut cargo_text)
        .unwrap();
    let cfg: Config = toml::from_str(cargo_text.as_str()).unwrap();
    cfg
}

fn subscribe_all(client: Client) -> Result<(), ClientError> {
    client.subscribe("srvList/#", QoS::AtLeastOnce)
}

fn unsubscribe_all(client: Client) -> Result<(), ClientError> {
    client.unsubscribe("srvList/#")
}

/// extract broker info from a mqtt srvList msg
fn extract_broker(topic: Bytes, payload: Bytes) -> Result<Broker, Box<dyn std::error::Error>> {
    let topic = String::from_utf8(topic.to_vec())?;
    let mut it = topic.split(":");
    let ip = it.next().unwrap().try_into()?;
    let port = it.next().unwrap().parse::<u16>()?.try_into()?;
    let address = String::from_utf8(payload.to_vec())?.parse()?;

    Ok(Broker {
        ip,
        port,
        address
    })
}

/// get list of all cacher according to broker
pub fn get_list_cacher_from_broker(
    client: rumqttc::v5::Client,
    mut connection: Connection,
) -> Result<std::vec::Vec<Broker>, Box<dyn std::error::Error>> {
    subscribe_all(client.clone())?;

    let mut res = vec![];

    loop {
        let x = connection.recv_timeout(Duration::from_millis(1000));
        match x {
            Err(_) => break, // first error has to be timeout
            Ok(Ok(Event::Incoming(Incoming::Publish(p)))) => {
                let tmp = extract_broker(p.topic.clone(), p.payload.clone());

                match tmp {
                    Ok(b) => res.push(b),
                    Err(e) => println!("Could not retrive broker: {:?}, msg: {:?}", e, p),
                };
                //break at end of list. Mqtt sends list in alphabetical order.
            }
            Ok(Ok(_)) => (),
            Ok(Err(e)) => {
                //second error is con error
                println!("Error in get list cacher: {:?}", e);
            }
        }
    }

    Ok(res)
}

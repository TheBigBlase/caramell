use bytes::Bytes;
use memcache;
use rumqttc;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;

pub mod blockchain;
pub mod contracts;

#[derive(Deserialize)]
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
    pub id: String,
}

#[derive(Deserialize, Clone)]
pub struct Blockchain {
    pub rpc_url_http: String,
    pub rpc_url_ws: String,
    pub wallet_key: String,
    pub contract_addr: String
}

#[derive(Clone)]
pub struct MemcacheClient {
    ip: String,
    port: u16,
    option: Vec<String>,
    client: memcache::Client,
}

impl MemcacheClient {
    pub fn new(ip: String, port: u16, option: Vec<String>) -> MemcacheClient {
        let options = option.join("&");
        let url = format!("memcache://{}:{}?{}", ip, port, options);
        let client = memcache::connect(url).unwrap();
        MemcacheClient {
            ip,
            port,
            option,
            client,
        }
    }

    pub fn insert_memcached(
        &self,
        key: String,
        value: String,
        exp: u32,
    ) -> Result<(), memcache::MemcacheError> {
        self.client.set(key.as_str(), value, exp)
    }
}

pub fn check_publish(
    msg: rumqttc::Event,
    mem_client: MemcacheClient
) -> Result<(), memcache::MemcacheError> {
    match msg {
        rumqttc::Event::Outgoing(not) => {
            println!("Outgoing {:?}", not);
            Ok(())
        }
        rumqttc::Event::Incoming(not) => {
            match not {
                rumqttc::Packet::Publish(n) => {
                    println!("{:?}", n.payload);
                    check_mem(n.payload, mem_client)
                }
                _ => Ok(()),
            }
        }
    }
}

pub fn check_mem(msg: Bytes, mem_client: MemcacheClient) -> Result<(), memcache::MemcacheError> {
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
        Ok(())
    }
}

pub fn load_toml(path: &str) -> Config {
    let mut cargo_text = String::new();
    File::open(format!("{}/Cargo.toml", path.to_string()))
        .unwrap()
        .read_to_string(&mut cargo_text)
        .unwrap();
    let params: Config = toml::from_str(cargo_text.as_str()).unwrap();
    params
}


use bytes::Bytes;
use memcache;

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
    mem_client: MemcacheClient,
) -> Result<(), memcache::MemcacheError> {
    match msg {
        rumqttc::Event::Outgoing(not) => {
            println!("Outgoing {:?}", not);
            Ok(())
        }
        rumqttc::Event::Incoming(not) => {
            println!("in {:?}", not);
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

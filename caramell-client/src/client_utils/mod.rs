use rumqttc::v5::{Event, Incoming};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use utils::{blockchain, contracts::client_contract::Data};
//TODO use utils::blockchain for all ethers stuff

// used in main.rs to pass everything in one state obj

pub use rumqttc::v5::{AsyncClient, EventLoop, MqttOptions};
use utils::{Broker, Config};

async fn insert_vec_in_mutex(vec: Arc<Mutex<Vec<Broker>>>, brk: Broker) {
    vec.lock().unwrap().push(brk.clone());
}

pub async fn handle_eventloop(
    mut evtloop: EventLoop,
    vec: Arc<Mutex<Vec<Broker>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let msg = evtloop.poll().await;
        match msg {
            // TODO other events ?
            Ok(Event::Incoming(Incoming::Publish(p))) => {
                let topic = p.topic;
                if topic.starts_with("srvList".as_bytes()) {
                    let brk = utils::extract_broker(topic, p.payload.clone())
                        .unwrap();

                    insert_vec_in_mutex(vec.clone(), brk).await;
                }
            }
            Ok(_) => (),
            Err(e) => {
                println!("Disconnected: {:?}", e);
            }
        }
    }
}

pub async fn init_eventloop(
    cfg: Config,
) -> Result<(AsyncClient, EventLoop), Box<dyn std::error::Error>> {
    let params: utils::Params = cfg.params.unwrap();

    let mut mqttoptions = MqttOptions::new(
        params.id.clone(),
        params.broker_ip.clone(),
        params.broker_port,
    );
    // useless, keeping it just in case :)
    mqttoptions.set_keep_alive(Duration::from_secs(60));

    let (client, eventloop) = AsyncClient::new(mqttoptions, 10);
    Ok((client, eventloop))
}

/// DEPRECIATED keeping it if i need it after BUT DONT YOU DARE USE IT
pub async fn broker_list(
    cfg: Config,
    client: AsyncClient,
    evtloop: EventLoop,
) -> Result<Vec<Broker>, Box<dyn std::error::Error>> {
    utils::get_list_cacher_from_broker(&client, evtloop).await
}

/// returns data retrieved from client.
/// sends reqs through mqtt. Usefull for server side load balancing
/// Upon server reception, opens a ftp socket and sends file(s) back
/// through a ftp socket
pub async fn read_data(
    client_contract: blockchain::ClientContractAlias,
    data_name: String,
    client_mqtt: AsyncClient,
    evtloop: EventLoop,
) -> Result<Data, Box<dyn std::error::Error>> {
    let data = retrieve_data_location(client_contract, data_name).await?;

    // TODO read data from data.data as ip+addr / key value
    // ip/addr is gathered from mqtt, send reqs to mqtt if offline ?
    //      it does give a centralized point of failure / ease of use
    //      maybe each node should have 1 instance and everyone relays everyone
    //      dht of mqtt lmao

    let res = Data::default();
    Ok(res)
}

/// returns Data retrieved from blockchain
pub async fn retrieve_data_location(
    client: blockchain::ClientContractAlias,
    data_name: String,
) -> Result<Data, Box<dyn std::error::Error>> {
    let res = client.get_data(data_name).call().await?;

    Ok(res)
}

/// TODO :)
pub async fn set_data(
    client: blockchain::ClientContractAlias,
    data_name: String,
) -> Result<Data, Box<dyn std::error::Error>> {
    let res = Data::default();
    Ok(res)
}

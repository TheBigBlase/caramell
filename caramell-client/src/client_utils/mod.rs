use rumqttc::v5::{Event, Incoming};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use utils::{blockchain, contracts::client_contract::Data};

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

/// initialize the event loop and returns it with a client
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

/// returns data retrieved from client.
/// sends reqs through mqtt. Usefull for server side load balancing
/// Upon server reception, opens a ftp socket and sends file(s) back
/// through a ftp socket
///
/// Takes an async mqtt client that the server is connected to.
/// TODO
pub async fn read_data(
    client_contract: Arc<blockchain::ClientContractAlias>,
    data_name: String,
    client_mqtt: AsyncClient,
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

/// TODO :)
pub async fn set_data(
    client: Arc<blockchain::ClientContractAlias>,
    data_name: String,
) -> Result<Data, Box<dyn std::error::Error>> {
    let res = Data::default();
    Ok(res)
}

/// returns Data retrieved from blockchain
pub async fn retrieve_data_location(
    contract_client: Arc<blockchain::ClientContractAlias>,
    data_name: String,
) -> Result<Data, Box<dyn std::error::Error>> {
    let res = contract_client.get_data(data_name).call().await?;

    Ok(res)
}

/// retreive all data "pointers" from contract
pub async fn retrieve_all_data_location(
    client: Arc<blockchain::ClientContractAlias>,
) -> Result<Vec<Data>, Box<dyn std::error::Error>> {
    let res = client.get_all_data().call().await?;

    Ok(res)
}


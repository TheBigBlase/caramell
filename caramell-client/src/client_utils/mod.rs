use rumqttc::{v5::{
    mqttbytes::{v5::Publish, QoS},
    Event, Incoming,
}, RecvTimeoutError};

use tokio::time::{timeout, Timeout};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::{watch::Sender, watch::Receiver};
use utils::{blockchain, contracts::client_contract::Data};

pub use rumqttc::v5::{AsyncClient, EventLoop, MqttOptions};
use utils::{Broker, Config};

async fn insert_vec_in_mutex(vec: Arc<Mutex<Vec<Broker>>>, brk: Broker) {
    vec.lock().unwrap().push(brk.clone());
}

async fn handle_publish(
    p: Publish,
    vec: Arc<Mutex<Vec<Broker>>>,
    self_addr: &str,
) {
    let topic = p.topic;
    if topic.starts_with("srvList".as_bytes()) {
        let brk = utils::extract_broker(topic, p.payload.clone()).unwrap();

        insert_vec_in_mutex(vec.clone(), brk).await;
    } else if topic.starts_with(format!("srv/to/{}", self_addr).as_bytes()) {

        //let brk = utils::extract_broker(topic, );
    }
}

pub async fn handle_eventloop(
    mut evtloop: EventLoop,
    vec: Arc<Mutex<Vec<Broker>>>,
    self_addr: &str,
    tx: Sender<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // using a watch channel so we can clone rx, and we drop unread values
    // TODO prove that we are indeed self_addr
    loop {
        let msg = evtloop.poll().await;
        match msg {
            // TODO other events ?
            Ok(Event::Incoming(Incoming::Publish(p))) => {
                handle_publish(p.clone(), vec.clone(), self_addr);
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

/// send a read req to server. Returns the retrieved string.
/// Needs a mpsc channel to send it back, which is passed as arg.
/// sends reqs through mqtt. Usefull for server side load balancing
/// Upon server reception, opens a ftp socket and sends file(s) back
/// through a ftp socket
///
/// Takes an async mqtt client that the server is connected to.
/// TODO
pub async fn read_data(
    data_name: &str,
    brk: Broker,
    client_mqtt: AsyncClient,
    self_addr: &str,
    mut rx: Receiver<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    // TODO prove that src is us
    // maybe use a pgp signature w/ wallet key?

    client_mqtt
        .subscribe(format!("{}:{}", brk.ip, brk.port), QoS::ExactlyOnce)
        .await?;
    client_mqtt
        .publish(
            format!("{}:{}", brk.ip, brk.port),
            QoS::ExactlyOnce,
            true,
            format!("READ{};SRC{}", data_name, self_addr),
        )
        .await?;

    // TODO read data from data.data as ip+addr / key value
    // ip/addr is gathered from mqtt, send reqs to mqtt if offline ?
    //      it does give a centralized point of failure / ease of use
    //      maybe each node should have 1 instance and everyone relays everyone
    //      dht of mqtt lmao

    
    let res = rx
        .changed();

    utils::unsubscribe_all(client_mqtt).await?;

    match timeout(Duration::from_secs(2), res).await {
        Ok(_) => {
            Ok(rx.borrow().to_string())
        }
        Err(e) => {Err(Box::new(e))}
    }


    


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

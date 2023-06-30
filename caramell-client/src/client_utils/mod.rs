use std::time::Duration;
//TODO use utils::blockchain for all ethers stuff

use ethers::signers::{LocalWallet, Signer};
use ethers_middleware::SignerMiddleware;
use ethers_providers::{Provider, Ws};

use rumqttc::v5::{Client, MqttOptions};
use tokio::runtime::Runtime;
use utils::{blockchain, contracts::client_contract::clientContract, Broker, Config};

pub fn broker_list(cfg: Config) -> Result<Vec<Broker>, Box<dyn std::error::Error>> {
    let params: utils::Params = cfg.params.unwrap();

    let mut mqttoptions = MqttOptions::new(
        params.id.clone(),
        params.broker_ip.clone(),
        params.broker_port,
    );
    // useless, keeping it just in case :)
    mqttoptions.set_keep_alive(Duration::from_secs(60));

    let (client, eventloop) = Client::new(mqttoptions, 10);

    utils::get_list_cacher_from_broker(client, eventloop)
}

fn init_contract(
    cfg: utils::Config,
    brk_lst: Vec<Broker>,
) -> Result<clientContract<SignerMiddleware<Provider<Ws>, LocalWallet>>, Box<dyn std::error::Error>>
{
    let rpc_url = cfg.clone().blockchain.unwrap().rpc_url_ws;

    let broker: &Broker = brk_lst.first().unwrap();

    let address = broker.address;

    // "await" in a sync func:
    let rt = Runtime::new().unwrap();
    let promise_wallet = blockchain::create_wallet(cfg.clone());
    let wallet = rt.block_on(promise_wallet)?;

    let promise_contract =
        blockchain::get_client_contract_addr(cfg.clone(), Some(address), wallet.clone());
    let client_contract_addr = rt.block_on(promise_contract)?;

    println!("contract address: {:?}", client_contract_addr);

    let client_contract = blockchain::create_client(address, wallet, rpc_url.as_str())?;

    Ok(client_contract)
}

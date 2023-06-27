use ethers::providers::Ws;
use ethers::signers::{LocalWallet, Signer, Wallet};
use ethers::types::{H160, U256};
use ethers_middleware::core::k256::ecdsa::SigningKey;
use ethers_middleware::SignerMiddleware;
use ethers_providers::Provider;
use std::sync::Arc;

use tokio::task;

use rumqttc::v5::EventLoop;
use utils::blockchain;
use utils::contracts::client_factory::{clientFactory, ContractCreatedFilter};
use utils::MemcacheClient;

async fn serve_forever(mut eventloop: EventLoop, mem_client: MemcacheClient) {
    let _handle = task::spawn(async move {
        while let Ok(notification) = eventloop.poll().await {
            let res = utils::check_publish(notification, mem_client.clone());
            match res {
                Err(utils::ErrorBrokerMemcached::MemcacheError(e)) => {
                    panic!("MemcacheError: {:?}", e)
                }
                Ok(string) => {
                    println!("{}", string)
                }
                _ => {}
            }
        }
    }).await;

    panic!("exiting forever loop");
}

/// serve forever, checking for contract gas / validity after caching 1st data.
/// Begin transaction w/ next block
pub async fn serve_trust(
    eventloop: EventLoop,
    mem_client: MemcacheClient,
) -> Result<(), Box<dyn std::error::Error>> {
    //TODO check blck contracts
    let _serve_handle = serve_forever(eventloop, mem_client);

    let config = utils::load_toml("caramell-server");

    let wallet = blockchain::create_wallet(config.clone());
    let client_addr = blockchain::get_client_contract_addr(config).await?;
    println!("{:?}", client_addr);

    _serve_handle.await;

    Ok(())
}

use std::sync::Arc;

use crate::contracts::{
    client_contract::Data,
    client_factory::{clientFactory, ContractCreatedFilter},
};

use ethers::signers::{LocalWallet, Signer, Wallet};
use ethers_contract::EthLogDecode;
use ethers_middleware::core::k256::ecdsa::SigningKey;
use ethers_middleware::SignerMiddleware;
use ethers_providers::{Provider, StreamExt, Ws};
use primitive_types::{H160, U256};

/// get address of contract, listening to events. The returned contract is owned by the caller
pub async fn get_address_contract_from_event<M: 'static, D>(
    evt: ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, ContractCreatedFilter>,
    owner: H160,
) -> Result<H160, Box<dyn std::error::Error>>
where
    M: ethers_providers::Middleware,
    M::Provider: ethers_providers::PubsubClient,
    D: EthLogDecode,
{
    let mut contract_addr: H160 = H160::zero();

    let mut evt = evt.subscribe().await?;
    while let Ok(e) = evt.next().await.unwrap() {
        println!("BLCK {:?}", e);
        contract_addr = e.contract_address;
        if owner == e.owner {
            break;
        }
    }
    Ok(contract_addr)
}

/// Create data wrapper. Init data at 0, and time of block at the block time
/// on the blockchain side.
pub fn create_data(name: &str, time_to_store: U256) -> Data {
    Data {
        name: String::from(name),
        data: U256::zero(), // pointer location, set by contract
        time_to_store,
        time_created: U256::zero(), //set by contract
    }
}

/// simply create a wallet :)
/// useless to put in a lib, since its a """one liner"""
/// but jesus what a line that is
pub async fn create_wallet(
    config: crate::Config,
) -> Result<LocalWallet, Box<dyn std::error::Error + 'static>> {
    let wallet = config
        .blockchain
        .clone()
        .expect("blockchain config block not found in cargo.toml")
        .wallet_key
        .parse::<LocalWallet>()
        .expect("walletError"); //local node

    Ok(wallet)
}

async fn create_client_factory(
    config: crate::Config,
    wallet: LocalWallet,
) -> Result<
    clientFactory<SignerMiddleware<Provider<Ws>, Wallet<SigningKey>>>,
    Box<dyn std::error::Error>,
> {
    let rpc_url = config.blockchain.clone().unwrap().rpc_url_ws;
    let contract_addr: H160 = config.blockchain.clone().unwrap().contract_addr;

    let provider: Provider<Ws> = Provider::<Ws>::connect(rpc_url).await?;

    let middleware =
        SignerMiddleware::new(provider.clone(), wallet.clone().with_chain_id(1337 as u64));

    Ok(clientFactory::new(
        contract_addr.clone(),
        Arc::new(middleware.clone()),
    ))
}

/// get client's contract address w/ given wallet & factory address
pub async fn get_client_contract_addr(
    mut config: crate::Config,
    address: Option<H160>,
    wallet: Wallet<SigningKey>
) -> Result<H160, Box<dyn std::error::Error>> {

    if address.is_some() {
        config.blockchain.as_mut().unwrap().contract_addr = address.unwrap();
    }

    let factory = create_client_factory(config, wallet.clone()).await?;

    let mut client_addr = factory.get_client().call().await?;

    if client_addr.is_zero() {
        let evt = factory.events();

        client_addr = {
            factory.new_client().send().await?;
            get_address_contract_from_event::<
                SignerMiddleware<Provider<Ws>, Wallet<SigningKey>>,
                ContractCreatedFilter,
            >(evt, wallet.address())
        }
        .await?; // await BOTH, launch symultaneously (?)
    }

    Ok(client_addr)
}

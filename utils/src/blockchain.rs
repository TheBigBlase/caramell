use std::sync::Arc;

use crate::contracts::{
    client_contract::{clientContract, Data},
    client_factory::{clientFactory, ContractCreatedFilter},
};

use ethers::signers::{LocalWallet, Signer, Wallet};
use ethers_contract::EthLogDecode;
use ethers_middleware::core::k256::ecdsa::SigningKey;
use ethers_middleware::SignerMiddleware;
use ethers_providers::{Provider, StreamExt, Ws};
use primitive_types::{H160, U256};
use tokio::runtime::Runtime;

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

pub async fn create_middleware(
    url: &str,
    wallet: Wallet<SigningKey>,
) -> Result<SignerMiddleware<Provider<Ws>, LocalWallet>, Box<dyn std::error::Error>> {
    let provider = Provider::<Ws>::connect(url).await?;

    let mw = SignerMiddleware::new(provider.clone(), wallet.clone().with_chain_id(1337 as u64));

    Ok(mw)
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

    let mw = create_middleware(rpc_url.as_str(), wallet).await?;

    Ok(clientFactory::new(
        contract_addr.clone(),
        Arc::new(mw.clone()),
    ))
}

/// get client's contract address w/ given wallet & factory address
pub async fn get_client_contract_addr(
    mut config: crate::Config,
    address: Option<H160>,
    wallet: Wallet<SigningKey>,
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

pub fn create_client(
    address: H160,
    wallet: LocalWallet,
    url: &str,
) -> Result<
    clientContract<SignerMiddleware<ethers_providers::Provider<Ws>, LocalWallet>>,
    Box<dyn std::error::Error>,
> {
    let rt = Runtime::new().unwrap();
    let mw_promise = create_middleware(url, wallet);

    let mw = rt.block_on(mw_promise)?;

    let client = clientContract::new(address, Arc::new(mw));
    Ok(client)
}

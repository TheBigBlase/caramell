use crate::contracts::{client_factory::ContractCreatedFilter, client_contract::Data};
use ethers_contract::EthLogDecode;
use ethers_providers::StreamExt;
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
    let mut contract_addr: H160 = H160([0u8; 20]);

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
pub fn create_data(name: &str, time_to_store:U256) -> Data {
    Data {
        name: String::from(name), 
        data: U256::zero(),// pointer location, set by contract
        time_to_store, 
        time_created: U256::zero()//set by contract
    }
}

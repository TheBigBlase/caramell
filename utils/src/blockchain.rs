use crate::contracts::client_factory::ContractCreatedFilter;
use ethers_contract::EthLogDecode;
use ethers_providers::StreamExt;
use primitive_types::H160;

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

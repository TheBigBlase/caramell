use rumqttc::v5::mqttbytes::QoS;
use rumqttc::v5::AsyncClient;
use tokio::task;

use rumqttc::v5::EventLoop;
use utils::blockchain;

use utils::MemcacheClient;

/// sends cacher's address to the broker
pub async fn init_broker_srvlist(
    client: AsyncClient,
    params: utils::Params,
    blck_p: utils::Blockchain,
) -> Result<(), rumqttc::v5::ClientError> {

    // ToString only returns in display mode, and since our str is too long it 
    // also adds a "..." in the middle of our string :/
    // so we format it in debug mode TODO find another way
    let addr = format!("{:?}", blck_p.contract_addr);

    client
        .publish(
            format!("srvList/{}:{}", params.broker_ip, params.broker_port),
            QoS::AtLeastOnce,
            true,
            addr
        )
        .await
}

/// run broker interface forever
async fn broker_serve_forever(mut eventloop: EventLoop, mem_client: MemcacheClient) {
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
    })
    .await;

    panic!("exiting forever loop");
}

/// serve forever, checking for contract gas / validity after caching 1st data.
/// Begin transaction w/ next block
pub async fn serve_trust(
    eventloop: EventLoop,
    mem_client: MemcacheClient,
) -> Result<(), Box<dyn std::error::Error>> {
    let serve_handle = broker_serve_forever(eventloop, mem_client);

    let config = utils::load_toml("caramell-server");

    let wallet = blockchain::create_wallet(config.clone()).await?;

    let client_contract_addr = blockchain::get_client_contract_addr(config, None, wallet).await?;

    println!("contract address: {:?}", client_contract_addr);

    serve_handle.await;
    // TODO tx to pay every interval, for now each block
    // TODO also change contract :)

    Ok(())
}

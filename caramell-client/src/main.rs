use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::time::Duration;
use tokio::{task, time};

#[tokio::main]
async fn main() {
    let mut mqttoptions = MqttOptions::new("caramell-client", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    task::spawn(async move {
        let mut i = 1;
        loop {
            client
                .publish("msg/srv", QoS::AtLeastOnce, true, format!("MEM;YO{};val{};0", i, i))
                .await
                .unwrap();
            time::sleep(Duration::from_millis(5000)).await;
            i+=1;
        }
    });

    while let Ok(notification) = eventloop.poll().await {
        println!("Received = {:?}", notification);
    }
}

use ethers::{
    middleware::contract::MultiAbigen,
    prelude::{abigen, Abigen},
};

fn main() {
    let abi_factory = Abigen::new(
        "ClientFactory",
        "createBindings/contracts/clientFactory.json",
    )
    .unwrap()
    .add_derive("serde::Serialize")
    .unwrap()
    .add_derive("serde::Deserialize")
    .unwrap();

    let abi_client = Abigen::new(
        "ClientContract",
        "createBindings/contracts/clientContract.json",
    )
    .unwrap()
    .add_derive("serde::Serialize")
    .unwrap()
    .add_derive("serde::Deserialize")
    .unwrap();

    let files = vec![abi_factory, abi_client];

    MultiAbigen::from_abigens(files.into_iter())
        .build()
        .unwrap()
        .write_to_module("createBindings/src/contracts", false)
        .unwrap();
}

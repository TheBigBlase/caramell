use ethers::middleware::contract::MultiAbigen;

fn main() {
    MultiAbigen::from_json_files("createBindings/contracts/")
        .unwrap()
        .build()
        .unwrap()
        .write_to_module("createBindings/src/contracts", false)
        .unwrap();
}

use anyhow::Result;
use ethers::abi::{Abi, Tokenize};
use ethers::contract::Contract;
use ethers::prelude::artifacts::BytecodeObject;
use ethers::prelude::*;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use std::{convert::TryFrom, env};

abigen!(
    Deposit,
    "./abi/contracts/Deposit.sol/Deposit/abi.json",
    event_derives(serde::Deserialize, serde::Serialize);

    SimpleToken,
    "./abi/contracts/SimpleToken.sol/SimpleToken/abi.json",
    event_derives(serde::Deserialize, serde::Serialize);
);

async fn load_contract(path: &Path) -> Result<(Abi, BytecodeObject)> {
    let abi_path = path.join("abi.json");
    let bin_path = path.join("bin.txt");

    let abi = ethers::abi::Contract::load(match fs::File::open(&abi_path) {
        Ok(v) => v,
        Err(_) => panic!("Unable to open path {:?}", abi_path),
    })?;

    let bytecode_str = match fs::read_to_string(&bin_path) {
        Ok(v) => v,
        Err(_) => panic!("Unable to read from path {:?}", bin_path),
    };
    let trimmed = bytecode_str.trim().trim_start_matches("0x");
    let bytecode: BytecodeObject = serde_json::from_value(serde_json::json!(trimmed)).unwrap();

    Ok((abi, bytecode))
}

pub async fn deploy<M: 'static + Middleware, T: Tokenize>(
    client: Arc<M>,
    path: &Path,
    constructor_args: T,
) -> Result<Contract<M>> {
    let (abi, bytecode) = load_contract(path).await?;
    let factory = ContractFactory::new(abi.clone(), bytecode.into_bytes().unwrap(), client.clone());
    let contract = factory.deploy(constructor_args)?.send().await?;
    Ok(contract)
}

#[tokio::main]
async fn main() -> Result<()> {
    let rpc_url = match env::var("RPC_URL") {
        Ok(val) => val,
        Err(_) => "http://localhost:8545".to_string(),
    };

    let provider = Provider::<Http>::try_from(rpc_url.clone())
        .expect("could not instantiate HTTP Provider")
        .interval(Duration::from_millis(100u64));

    let accounts = provider.get_accounts().await.unwrap();

    let chain_id = provider.get_chainid().await.unwrap().as_u64();

    let deployer_wallet = LocalWallet::new(&mut rand::thread_rng()).with_chain_id(chain_id);
    let deployer = Arc::new(SignerMiddleware::new(
        provider.clone(),
        deployer_wallet.clone(),
    ));

    let alice = Arc::new(SignerMiddleware::new(
        provider.clone(),
        LocalWallet::new(&mut rand::thread_rng()).with_chain_id(chain_id),
    ));

    for address in [deployer_wallet.address(), alice.address()] {
        let tx = TransactionRequest::new()
            .to(address)
            .value(ethers::utils::parse_ether(U256::from(1))?)
            .from(accounts[0]);

        let tx = provider.send_transaction(tx, None).await?.await?;
        println!("Sent funding tx to {} {:?}", address, tx.unwrap().status);
    }

    let token = deploy(
        deployer.clone(),
        Path::new("./abi/contracts/SimpleToken.sol/SimpleToken"),
        (),
    )
    .await
    .unwrap();
    let token = SimpleToken::new(token.address(), deployer.clone());
    println!("deployed token");

    let deposit = deploy(
        deployer.clone(),
        Path::new("./abi/contracts/Deposit.sol/Deposit"),
        (),
    )
    .await
    .unwrap();
    let deposit = Deposit::new(deposit.address(), deployer.clone());
    println!("deployed deposit");

    let alice_token = SimpleToken::new(token.address(), alice.clone());
    let alice_deposit = Deposit::new(deposit.address(), alice.clone());

    let amount = U256::from(1000);

    token
        .transfer(alice.address(), amount)
        .send()
        .await?
        .await?;

    let tx = alice_token
        .approve(deposit.address(), amount)
        .send()
        .await?
        .await?;

    println!("approve tx status {:?}", tx.unwrap().status);

    let tx = alice_deposit
        .deposit(token.address(), amount)
        .send()
        .await?
        .await?;

    // try to avoid estimate gas?
    println!("deposit tx status {:?}", tx.unwrap().status);

    // Check the tokens have been transferred
    let balance_contract = token.balance_of(deposit.address()).call().await?;

    println!("balance contract {}", balance_contract);
    assert_eq!(balance_contract, amount);

    Ok(())
}

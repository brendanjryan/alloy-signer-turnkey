use alloy_primitives::U256;
use alloy_provider::ProviderBuilder;
use alloy_rpc_types_eth::TransactionRequest;
use alloy_signer::Signer;
use alloy_signer_turnkey::{TurnkeyP256ApiKey, TurnkeySigner};
use alloy_sol_types::{sol, SolCall};
use std::env;

// cargo run --example contract_call
sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    contract SimpleStorage {
        uint256 public storedValue;

        event ValueUpdated(uint256 newValue);

        constructor(uint256 initialValue) {
            storedValue = initialValue;
        }

        function setValue(uint256 value) public {
            storedValue = value;
            emit ValueUpdated(value);
        }

        function getValue() public view returns (uint256) {
            return storedValue;
        }

        function increment() public {
            storedValue += 1;
            emit ValueUpdated(storedValue);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let organization_id = env::var("TURNKEY_ORGANIZATION_ID")
        .expect("TURNKEY_ORGANIZATION_ID environment variable required");

    let api_private_key = env::var("TURNKEY_API_PRIVATE_KEY")
        .expect("TURNKEY_API_PRIVATE_KEY environment variable required");

    let ethereum_address = env::var("ETHEREUM_ADDRESS")
        .expect("ETHEREUM_ADDRESS environment variable required")
        .parse()
        .expect("Valid Ethereum address required");

    let contract_address = env::var("CONTRACT_ADDRESS")
        .unwrap_or_else(|_| "0x742d35Cc6635C0532925a3b8D9C3Ac6b95fc9b8E".to_string())
        .parse()
        .expect("Valid contract address required");

    let rpc_url = env::var("RPC_URL").unwrap_or_else(|_| "https://rpc.sepolia.org".to_string());

    println!("Setting up Turnkey signer...");

    let api_key = TurnkeyP256ApiKey::from_strings(api_private_key, None)?;
    let signer = TurnkeySigner::new(organization_id, ethereum_address, api_key)?;

    let signer = signer.with_chain_id(Some(11155111));

    println!("Signer address: {}", signer.address());
    println!("Chain ID: {:?}", signer.chain_id());

    println!("Connecting to RPC: {}", rpc_url);
    let _provider = ProviderBuilder::new().connect_http(rpc_url.parse()?);

    println!("Contract address: {}", contract_address);

    println!("=== Reading current value ===");

    let get_value_call = SimpleStorage::getValueCall {};
    let call_data = get_value_call.abi_encode();

    let _call_request = TransactionRequest::default()
        .to(contract_address)
        .input(call_data.into());

    println!(
        "View call prepared - getData: 0x{}",
        hex::encode(get_value_call.abi_encode())
    );

    println!("=== Setting new value ===");
    let new_value = U256::from(42);

    println!("Preparing transaction to set value to: {}", new_value);

    let call_data = SimpleStorage::setValueCall { value: new_value };
    println!("Call data: 0x{}", hex::encode(call_data.abi_encode()));

    let _tx_request = TransactionRequest::default()
        .to(contract_address)
        .input(call_data.abi_encode().into())
        .gas_limit(100_000u64)
        .gas_price(20_000_000_000u128);

    println!("Transaction details:");
    println!("To: {}", contract_address);
    println!("Gas limit: 100,000");
    println!("Gas price: 20 gwei");
    println!("Data: 0x{}", hex::encode(call_data.abi_encode()));

    println!("=== Signing transaction with Turnkey ===");

    use alloy_primitives::B256;
    let sample_hash = B256::from([
        0xa1, 0xb2, 0xc3, 0xd4, 0xe5, 0xf6, 0x07, 0x18, 0x29, 0x3a, 0x4b, 0x5c, 0x6d, 0x7e, 0x8f,
        0x90, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee,
        0xff, 0x00,
    ]);

    println!(
        "Signing sample transaction hash: 0x{}",
        hex::encode(sample_hash)
    );

    match signer.sign_hash(&sample_hash).await {
        Ok(signature) => {
            println!("Signature successful!");
            println!("Final Signature Components:");
            println!("r: 0x{}", hex::encode(signature.r().to_be_bytes::<32>()));
            println!("s: 0x{}", hex::encode(signature.s().to_be_bytes::<32>()));
            println!(
                "  v: {} (parity: {})",
                if signature.v() { 1 } else { 0 },
                signature.v()
            );

            // Verify the signature
            let recovered = signature.recover_address_from_prehash(&sample_hash)?;
            println!("Signature Verification:");
            println!("Original signer: {}", signer.address());
            println!("Recovered address: {}", recovered);

            if recovered == signer.address() {
                println!("Result: Signature verification successful!");
            } else {
                println!("Result: Signature verification failed!");
            }
        }
        Err(e) => {
            println!("Signing failed: {}", e);
        }
    }

    // Example 4: Sign a message to show the encoding process
    println!("=== Signing a Message ===");
    let message = b"Hello, Turnkey contract signing!";
    println!("Message to sign: {}", std::str::from_utf8(message)?);

    match signer.sign_message(message).await {
        Ok(signature) => {
            println!("Message signature successful!");

            // Verify the message signature
            let recovered = signature.recover_address_from_msg(message)?;
            println!("Message Signature Verification:");
            println!("Original signer: {}", signer.address());
            println!("Recovered address: {}", recovered);

            if recovered == signer.address() {
                println!("Result: Message signature verification successful!");
            } else {
                println!("Result: Message signature verification failed!");
            }
        }
        Err(e) => {
            println!("Message signing failed: {}", e);
        }
    }

    Ok(())
}

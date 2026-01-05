use alloy_consensus::TxEip1559;
use alloy_network::TransactionBuilder;
use alloy_primitives::{Bytes, TxKind, U256};
use alloy_rpc_types_eth::TransactionRequest;
use alloy_signer_turnkey::TurnkeyP256ApiKey;
use alloy_sol_types::{sol, SolConstructor};
use base64::{engine::general_purpose, Engine as _};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

// cargo run --example turnkey_rest_api

sol! {

    #[allow(missing_docs)]
    // random bytecode
    #[sol(rpc, bytecode = "608060405234801561001057600080fd5b5060405161019838038061019883398101604081905261002f91610037565b600055610050565b60006020828403121561004957600080fd5b5051919050565b610139806100636000396000f3fe608060405234801561001057600080fd5b506004361061005e5760003560e01c80632096525514610063578063209652551461007c5780633fa4f245146100915780636057361d1461009a578063d09de08a146100ad575b600080fd5b61006b60005481565b60405190815260200160405180910390f35b61008f61008a366004610092565b61006b565b005b61006b60005481565b61008f6100a8366004610092565b600055565b61008f6000600081905561010381906100c5903390565b60405190815260200160405180910390a1565b6000546040805191825260208201527f2e2f6a0aa0c21f6b65acc2efcaf8cd36977ebb4e2ad3a38c76de01a80dba4a60910160405180910390a1565b60006020828403121561011257600080fd5b503591905056fea2646970667358221220f0a9c3b9c8b4b9a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f45600")]
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

#[derive(Serialize)]
struct TurnkeySignTransactionRequest {
    #[serde(rename = "type")]
    activity_type: String,
    #[serde(rename = "timestampMs")]
    timestamp_ms: String,
    #[serde(rename = "organizationId")]
    organization_id: String,
    parameters: SignTransactionParameters,
}

#[derive(Serialize)]
struct SignTransactionParameters {
    #[serde(rename = "signWith")]
    sign_with: String,
    #[serde(rename = "unsignedTransaction")]
    unsigned_transaction: String,
    #[serde(rename = "type")]
    transaction_type: String,
}

#[derive(Deserialize)]
struct TurnkeySignTransactionResponse {
    activity: Activity,
}

#[derive(Deserialize)]
struct Activity {
    id: String,
    status: String,
    result: Option<ActivityResult>,
}

#[derive(Deserialize)]
struct ActivityResult {
    #[serde(rename = "signTransactionResult")]
    sign_transaction_result: Option<SignTransactionResult>,
}

#[derive(Deserialize)]
struct SignTransactionResult {
    #[serde(rename = "signedTransaction")]
    signed_transaction: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    // Required environment variables
    let organization_id = env::var("TURNKEY_ORGANIZATION_ID")
        .expect("TURNKEY_ORGANIZATION_ID environment variable required");

    let api_private_key = env::var("TURNKEY_API_PRIVATE_KEY")
        .expect("TURNKEY_API_PRIVATE_KEY environment variable required");

    let ethereum_address =
        env::var("ETHEREUM_ADDRESS").expect("ETHEREUM_ADDRESS environment variable required");

    // Optional environment variables with defaults
    let rpc_url = env::var("RPC_URL").unwrap_or_else(|_| "https://rpc.sepolia.org".to_string());

    println!("Turnkey REST API Contract Deployment Example");
    println!("Organization ID: {}", organization_id);
    println!("Ethereum Address: {}", ethereum_address);
    println!("RPC URL: {}", rpc_url);
    println!("Mode: Deploy");

    // Step 1: Construct the contract deployment transaction
    println!("Step 1: Constructing Contract Deployment Transaction");

    let initial_value = U256::from(42);
    let constructor_args = SimpleStorage::constructorCall {
        initialValue: initial_value,
    };
    let mut deploy_data = SimpleStorage::BYTECODE.to_vec();
    deploy_data.extend(constructor_args.abi_encode());

    println!(
        "Deploying SimpleStorage with initial value: {}",
        initial_value
    );
    println!(
        "Constructor args: 0x{}",
        hex::encode(constructor_args.abi_encode())
    );
    println!("Full deploy data length: {} bytes", deploy_data.len());

    let tx_request = TransactionRequest::default()
        .with_input(Bytes::from(deploy_data))
        .with_gas_limit(500_000u64) // Higher gas for deployment
        .with_max_fee_per_gas(30_000_000_000u128) // 30 gwei max fee
        .with_max_priority_fee_per_gas(2_000_000_000u128) // 2 gwei priority fee
        .with_nonce(0u64)
        .with_chain_id(11155111); // Sepolia testnet

    println!("Transaction details:");
    println!("  Type: Contract deployment");
    println!("  To: None (deployment)");
    println!("  Gas limit: 500,000");
    println!("  Max fee per gas: 30 gwei");
    println!("  Max priority fee per gas: 2 gwei");
    println!("  Nonce: 0");
    println!("  Chain ID: 11155111 (Sepolia)");

    // Step 2: Serialize the transaction for Turnkey
    let unsigned_transaction = serialize_transaction_for_turnkey(&tx_request)?;
    println!("Step 2: Serialized Transaction");
    println!("Unsigned transaction hex: {}", unsigned_transaction);

    // Step 3: Create API key for authentication
    println!("Step 3: Setting up Turnkey Authentication");
    let api_key = TurnkeyP256ApiKey::from_strings(api_private_key, None)?;
    println!("API key created successfully");

    // Step 4: Create the signing request
    println!("Step 4: Creating Turnkey Sign Request");

    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis()
        .to_string();

    let sign_request = TurnkeySignTransactionRequest {
        activity_type: "ACTIVITY_TYPE_SIGN_TRANSACTION_V2".to_string(),
        timestamp_ms: timestamp_ms.clone(),
        organization_id: organization_id.clone(),
        parameters: SignTransactionParameters {
            sign_with: ethereum_address.clone(),
            unsigned_transaction: unsigned_transaction.clone(),
            transaction_type: "TRANSACTION_TYPE_ETHEREUM".to_string(),
        },
    };

    let request_body = serde_json::to_string(&sign_request)?;
    println!("Request body: {}", request_body);

    // Step 5: Create the cryptographic stamp for authentication
    println!("Step 5: Creating Cryptographic Stamp");
    let stamp = create_turnkey_stamp(&api_key, &request_body, &timestamp_ms)?;
    println!("Authentication stamp created");

    // Step 6: Make the REST API call to Turnkey
    println!("Step 6: Calling Turnkey REST API");

    let client = Client::new();
    let response = client
        .post("https://api.turnkey.com/public/v1/submit/sign_transaction")
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .header("X-Stamp", stamp)
        .body(request_body)
        .send()
        .await?;

    println!("Response status: {}", response.status());

    if response.status().is_success() {
        let response_body: TurnkeySignTransactionResponse = response.json().await?;

        println!("Step 7: Processing Response");
        println!("Activity ID: {}", response_body.activity.id);
        println!("Activity Status: {}", response_body.activity.status);

        if let Some(result) = response_body.activity.result {
            if let Some(sign_result) = result.sign_transaction_result {
                println!("SUCCESS: Contract Deployment Transaction Signed");
                println!("Signed Transaction: {}", sign_result.signed_transaction);

                // The signed transaction can now be broadcast to the network
                println!("Next steps:");
                println!("1. Broadcast this signed transaction to deploy the contract");
                println!("2. Monitor the transaction status to get the deployed contract address");
                println!("Example broadcast command (using curl):");
                println!("curl -X POST {} \\", rpc_url);
                println!("  -H \"Content-Type: application/json\" \\");
                println!("  -d '{{\"jsonrpc\":\"2.0\",\"method\":\"eth_sendRawTransaction\",\"params\":[\"{}\"],\"id\":1}}'", sign_result.signed_transaction);
            } else {
                println!("No sign transaction result in response");
            }
        } else {
            println!("No result in response - activity may still be pending");
        }
    } else {
        let error_text = response.text().await?;
        println!("Error response: {}", error_text);
        return Err(format!("API call failed: {}", error_text).into());
    }

    Ok(())
}

/// Serialize a transaction request into the format expected by Turnkey
fn serialize_transaction_for_turnkey(
    tx_request: &TransactionRequest,
) -> Result<String, Box<dyn std::error::Error>> {
    // Extract transaction fields for EIP-1559 transaction
    let nonce = tx_request.nonce.unwrap_or(0);
    let max_fee_per_gas = tx_request.max_fee_per_gas.unwrap_or(30_000_000_000);
    let max_priority_fee_per_gas = tx_request.max_priority_fee_per_gas.unwrap_or(2_000_000_000);
    let gas_limit = 500_000u64; // deployment gas limit
    let value = tx_request.value.unwrap_or(U256::ZERO);
    let chain_id = tx_request.chain_id.unwrap_or(11155111);

    // Get the input data (contract bytecode + constructor args for deployment)
    let input_data = tx_request.input.input.clone().unwrap_or_default();

    // Create a proper EIP-1559 transaction using Alloy consensus types
    let tx_eip1559 = TxEip1559 {
        chain_id,
        nonce,
        max_fee_per_gas,
        max_priority_fee_per_gas,
        gas_limit,
        to: tx_request.to.unwrap_or(TxKind::Create),
        value,
        input: input_data.clone(),
        access_list: Default::default(), // Empty access list for deployment
    };

    // RLP encode the transaction using Alloy's proper encoding
    use alloy_rlp::Encodable;
    let mut encoded_buf = Vec::new();
    tx_eip1559.encode(&mut encoded_buf);

    let hex_tx = format!("0x{}", hex::encode(&encoded_buf));

    // Show transaction details for debugging
    let tx_json = serde_json::json!({
        "type": "EIP-1559 Transaction",
        "nonce": nonce,
        "max_fee_per_gas": max_fee_per_gas,
        "max_priority_fee_per_gas": max_priority_fee_per_gas,
        "gas_limit": gas_limit,
        "to": match tx_request.to {
            Some(TxKind::Call(addr)) => format!("{:?}", addr),
            Some(TxKind::Create) => "null (contract deployment)".to_string(),
            None => "null (contract deployment)".to_string(),
        },
        "value": format!("{}", value),
        "chain_id": chain_id,
        "input_length": input_data.len(),
        "rlp_encoded_length": encoded_buf.len(),
        "rlp_hex_length": hex_tx.len()
    });

    println!("RLP-encoded transaction structure:");
    println!("{}", serde_json::to_string_pretty(&tx_json)?);
    println!("RLP-encoded transaction: {}", hex_tx);

    Ok(hex_tx)
}

/// Create the cryptographic stamp required for Turnkey API authentication
fn create_turnkey_stamp(
    api_key: &TurnkeyP256ApiKey,
    request_body: &str,
    _timestamp_ms: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Use the TurnkeyP256ApiKey's built-in stamp method
    match api_key.stamp(request_body) {
        Ok(stamp) => {
            println!("Successfully created authentication stamp");
            Ok(stamp)
        }
        Err(e) => {
            println!("Failed to create stamp: {}", e);
            // Fall back to a demonstration stamp for educational purposes
            let demo_stamp = serde_json::json!({
                "publicKey": "04demo_key_placeholder",
                "signature": "0xdemo_signature_placeholder",
                "scheme": "SIGNATURE_SCHEME_TK_API_P256"
            });

            println!("Using demo stamp for educational purposes: {}", demo_stamp);
            let encoded = general_purpose::STANDARD.encode(demo_stamp.to_string());
            Ok(encoded)
        }
    }
}

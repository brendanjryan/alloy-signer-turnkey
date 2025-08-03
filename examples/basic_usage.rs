use alloy_signer_turnkey::{Signer, TurnkeyP256ApiKey, TurnkeySigner};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    let organization_id = env::var("TURNKEY_ORGANIZATION_ID")
        .expect("TURNKEY_ORGANIZATION_ID environment variable required");

    let api_private_key = env::var("TURNKEY_API_PRIVATE_KEY")
        .expect("TURNKEY_API_PRIVATE_KEY environment variable required");

    let address = env::var("ETHEREUM_ADDRESS")
        .expect("ETHEREUM_ADDRESS environment variable required")
        .parse()
        .expect("Valid Ethereum address required");

    let api_key = TurnkeyP256ApiKey::from_strings(api_private_key, None)?;

    println!("Creating Turnkey signer...");
    let signer = TurnkeySigner::new(organization_id, address, api_key)?;

    let signer = signer.with_chain_id(Some(1));

    println!("Signer address: {}", signer.address());
    println!("Chain ID: {:?}", signer.chain_id());

    let message = b"Hello, Turnkey!";
    println!("Signing message: {:?}", std::str::from_utf8(message)?);

    let signature = signer.sign_message(message).await?;
    println!("Signature: {signature}");

    let recovered_address = signature.recover_address_from_msg(message)?;
    println!("Recovered address: {recovered_address}");

    if recovered_address == signer.address() {
        println!("Signature verification successful!");
    } else {
        println!("Signature verification failed!");
    }

    Ok(())
}

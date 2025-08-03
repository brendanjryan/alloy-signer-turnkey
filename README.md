# alloy-signer-turnkey

A [Turnkey](https://www.turnkey.com/) based signer implementation for [Alloy](https://github.com/alloy-rs/alloy).

This crate provides a `TurnkeySigner` that implements Alloy's `Signer` trait, enabling you to use Turnkey-based signers within Alloy.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
alloy-signer-turnkey = "0.1.0"
```

## Usage

### Basic Setup

```rust
use alloy_signer_turnkey::{TurnkeySigner, TurnkeyP256ApiKey, Signer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create API key
    let api_key = TurnkeyP256ApiKey::from_strings(
        "your-api-private-key",
        None, // Optional: public key (will be derived if not provided)
    )?;
    
    // Create signer
    let signer = TurnkeySigner::new(
        "your-organization-id".to_string(),
        "your-private-key-id".to_string(),
        api_key,
    ).await?;

    // Ethereum mainnet 
    let signer = signer.with_chain_id(Some(1));
    
    println!("Signer address: {}", signer.address());
    
    Ok(())
}
```

### Message Signing

```rust
// Sign a message
let message = b"Hello, Turnkey!";
let signature = signer.sign_message(message).await?;

// Verify signature
let recovered = signature.recover_address_from_msg(message)?;
assert_eq!(recovered, signer.address());
```

### Hash Signing

```rust
use alloy_primitives::B256;

// Sign a hash directly
let hash = B256::from([0u8; 32]);
let signature = signer.sign_hash(&hash).await?;
```

### Integration with Alloy

```rust
use alloy::signers::Signer;
use alloy::providers::{Provider, ProviderBuilder};

// Use with Alloy providers
let provider = ProviderBuilder::new()
    .with_signer(signer)
    .on_http("https://eth-mainnet.alchemyapi.io/v2/your-api-key".parse()?);

// Now you can send transactions using Turnkey for signing
```

## Configuration

### Environment Variables

The simplest way to configure the signer is through environment variables:

```bash
export TURNKEY_ORGANIZATION_ID="your-organization-id"
export TURNKEY_PRIVATE_KEY_ID="your-private-key-id"  
export TURNKEY_API_PRIVATE_KEY="your-api-private-key"
```

### API Key Management

```rust
// From environment variables
let api_key = TurnkeyP256ApiKey::from_strings(
    std::env::var("TURNKEY_API_PRIVATE_KEY")?,
    None,
)?;

// From files
let api_key = TurnkeyP256ApiKey::from_files(
    "/path/to/private_key.pem",
    Some("/path/to/public_key.pem"), // Optional
)?;

// Generate new key (for testing)
let api_key = TurnkeyP256ApiKey::generate();
```

## Examples

Check out the [examples](examples/) directory for complete working examples:

- [`basic_usage.rs`](examples/basic_usage.rs) - Basic signing operations
- More examples coming soon!

Run an example:

```bash
cargo run --example basic_usage
```

## License

MIT

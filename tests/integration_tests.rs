use alloy_primitives::{Address, ChainId, B256};
use alloy_signer_turnkey::{Signer, TurnkeyP256ApiKey, TurnkeySigner};

const ADDRESS_STR: &str = "0xB00F0759DbeeF5E543Cc3E3B07A6442F5f3928a2";
const ORGANIZATION_ID: &str = "test-org";

fn address() -> Address {
    ADDRESS_STR.parse().unwrap()
}

#[tokio::test]
async fn test_signer_creation_success() {
    let api_key = TurnkeyP256ApiKey::generate();

    let result = TurnkeySigner::new(ORGANIZATION_ID.to_string(), address(), api_key);
    assert!(result.is_ok());

    let signer = result.unwrap();
    assert_eq!(signer.address(), address());
    assert_eq!(signer.chain_id(), None);
}

#[tokio::test]
async fn test_signer_with_chain_id() {
    let api_key = TurnkeyP256ApiKey::generate();

    let signer = TurnkeySigner::new(ORGANIZATION_ID.to_string(), address(), api_key)
        .unwrap()
        .with_chain_id(Some(1));

    assert_eq!(signer.address(), address());
    assert_eq!(signer.chain_id(), Some(1));
}

#[tokio::test]
async fn test_library_exports() {
    use alloy_signer_turnkey::*;

    let _api_key = TurnkeyP256ApiKey::generate();

    let _addr: Address = address();
    let _hash = B256::ZERO;
    let _chain: ChainId = 1;

    let error = TurnkeyError::Configuration("test".to_string());
    assert!(matches!(error, TurnkeyError::Configuration(_)));

    let _result: Result<String> = Ok("test".to_string());
}

#[test]
fn test_error_types() {
    use alloy_signer_turnkey::*;

    let config_error = TurnkeyError::Configuration("test config error".to_string());
    assert_eq!(
        config_error.to_string(),
        "Configuration error: test config error"
    );

    let api_error = TurnkeyError::Api {
        code: "TEST_CODE".to_string(),
        message: "Test message".to_string(),
    };
    assert_eq!(api_error.to_string(), "API error [TEST_CODE]: Test message");
}

#[test]
fn test_address_handling() {
    let addr: Address = address();
    assert_eq!(addr.to_string().len(), 42); // 0x + 40 hex chars
}

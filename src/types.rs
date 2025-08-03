use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: String,
    pub organization_id: String,
    pub status: String,
    pub result: Option<ActivityResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityResult {
    #[serde(flatten)]
    pub data: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityResponse {
    pub activity: Activity,
}

// Sign Transaction Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignTransactionRequest {
    #[serde(rename = "type")]
    pub activity_type: String,
    #[serde(rename = "organizationId")]
    pub organization_id: String,
    pub parameters: SignTransactionParameters,
    #[serde(rename = "timestampMs")]
    pub timestamp_ms: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignTransactionParameters {
    #[serde(rename = "privateKeyId", skip_serializing_if = "Option::is_none")]
    pub private_key_id: Option<String>,
    #[serde(rename = "walletId", skip_serializing_if = "Option::is_none")]
    pub wallet_id: Option<String>,
    #[serde(rename = "unsignedTransaction")]
    pub unsigned_transaction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignTransactionResult {
    #[serde(rename = "signedTransaction")]
    pub signed_transaction: String,
}

// Sign Raw Payload Types (for message signing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignRawPayloadRequest {
    #[serde(rename = "type")]
    pub activity_type: String,
    #[serde(rename = "organizationId")]
    pub organization_id: String,
    pub parameters: SignRawPayloadParameters,
    #[serde(rename = "timestampMs")]
    pub timestamp_ms: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignRawPayloadParameters {
    #[serde(rename = "signWith")]
    pub sign_with: String,
    pub payload: String,
    pub encoding: String,
    #[serde(rename = "hashFunction")]
    pub hash_function: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignRawPayloadResult {
    pub r: String,
    pub s: String,
    pub v: String,
}

// Generic API Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnkeyApiResponse<T> {
    pub activity: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub message: String,
    pub code: String,
}

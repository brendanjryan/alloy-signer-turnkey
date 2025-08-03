use crate::error::{Result, TurnkeyError};
use crate::types::*;
use reqwest::Client;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use turnkey_api_key_stamper::TurnkeyP256ApiKey;

const TURNKEY_API_BASE_URL: &str = "https://api.turnkey.com";

#[cfg(not(test))]
const ACTIVITY_POLL_INTERVAL_MS: u64 = 1000; // 1 second
#[cfg(not(test))]
const ACTIVITY_TIMEOUT_MS: u64 = 30000; // 30 seconds

#[cfg(test)]
const ACTIVITY_POLL_INTERVAL_MS: u64 = 10; // 10ms for tests
#[cfg(test)]
const ACTIVITY_TIMEOUT_MS: u64 = 1000; // 1 second for tests

pub struct TurnkeyClient {
    client: Client,
    api_key: TurnkeyP256ApiKey,
    organization_id: String,
    base_url: String,
}

impl TurnkeyClient {
    pub fn new(organization_id: String, api_key: TurnkeyP256ApiKey) -> Result<Self> {
        let client = Client::new();

        Ok(Self {
            client,
            api_key,
            organization_id,
            base_url: TURNKEY_API_BASE_URL.to_string(),
        })
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    /// Sign a transaction using either a private key ID or wallet ID
    pub async fn sign_transaction(
        &self,
        unsigned_transaction: &str,
        private_key_id: Option<&str>,
        wallet_id: Option<&str>,
    ) -> Result<SignTransactionResult> {
        if private_key_id.is_none() && wallet_id.is_none() {
            return Err(TurnkeyError::Configuration(
                "Either private_key_id or wallet_id must be provided".to_string(),
            ));
        }

        let request = SignTransactionRequest {
            activity_type: "ACTIVITY_TYPE_SIGN_TRANSACTION".to_string(),
            organization_id: self.organization_id.clone(),
            parameters: SignTransactionParameters {
                private_key_id: private_key_id.map(|s| s.to_string()),
                wallet_id: wallet_id.map(|s| s.to_string()),
                unsigned_transaction: unsigned_transaction.to_string(),
            },
            timestamp_ms: self.current_timestamp_ms(),
        };

        let activity = self.submit_activity(&request).await?;
        let completed_activity = self.poll_activity(&activity.id).await?;

        if let Some(result) = completed_activity.result {
            let sign_result: SignTransactionResult = serde_json::from_value(
                result
                    .data
                    .get("signTransactionResult")
                    .ok_or_else(|| TurnkeyError::Api {
                        code: "MISSING_RESULT".to_string(),
                        message: "Missing signTransactionResult in response".to_string(),
                    })?
                    .clone(),
            )?;
            Ok(sign_result)
        } else {
            Err(TurnkeyError::ActivityFailed(
                "No result in completed activity".to_string(),
            ))
        }
    }

    /// Sign raw payload for message signing
    pub async fn sign_raw_payload(
        &self,
        private_key_id: &str,
        payload: &str,
        encoding: &str,
        hash_function: &str,
    ) -> Result<SignRawPayloadResult> {
        let request = SignRawPayloadRequest {
            activity_type: "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD".to_string(),
            organization_id: self.organization_id.clone(),
            parameters: SignRawPayloadParameters {
                sign_with: private_key_id.to_string(),
                payload: payload.to_string(),
                encoding: encoding.to_string(),
                hash_function: hash_function.to_string(),
            },
            timestamp_ms: self.current_timestamp_ms(),
        };

        let activity = self.submit_activity(&request).await?;
        let completed_activity = self.poll_activity(&activity.id).await?;

        if let Some(result) = completed_activity.result {
            let sign_result: SignRawPayloadResult = serde_json::from_value(
                result
                    .data
                    .get("signRawPayloadResult")
                    .ok_or_else(|| TurnkeyError::Api {
                        code: "MISSING_RESULT".to_string(),
                        message: "Missing signRawPayloadResult in response".to_string(),
                    })?
                    .clone(),
            )?;
            Ok(sign_result)
        } else {
            Err(TurnkeyError::ActivityFailed(
                "No result in completed activity".to_string(),
            ))
        }
    }

    // Helper methods for activity management
    async fn submit_activity<T: serde::Serialize>(&self, request: &T) -> Result<Activity> {
        let body = serde_json::to_vec(request)?;
        
        // Use the TurnkeyP256ApiKey to create proper headers
        let body_string = String::from_utf8(body.clone())?;
        let stamped_headers = self.api_key.stamp(&body_string, &self.current_timestamp_ms())?;

        let response = self
            .client
            .post(format!("{}/public/v1/submit/sign_transaction", self.base_url))
            .header("Content-Type", "application/json")
            .header("X-Stamp-WebAuthn", &stamped_headers)
            .body(body)
            .send()
            .await?;

        if response.status().is_success() {
            let activity_response: ActivityResponse = response.json().await?;
            Ok(activity_response.activity)
        } else {
            let error_text = response.text().await?;
            Err(TurnkeyError::Api {
                code: "HTTP_ERROR".to_string(),
                message: format!("HTTP error: {}", error_text),
            })
        }
    }

    async fn poll_activity(&self, activity_id: &str) -> Result<Activity> {
        let start_time = SystemTime::now();
        let timeout = Duration::from_millis(ACTIVITY_TIMEOUT_MS);

        loop {
            if start_time.elapsed().unwrap_or(Duration::ZERO) > timeout {
                return Err(TurnkeyError::Api {
                    code: "TIMEOUT".to_string(),
                    message: format!("Activity {} timed out", activity_id),
                });
            }

            let response = self
                .client
                .get(format!("{}/public/v1/query/get_activity?activityId={}&organizationId={}", 
                    self.base_url, activity_id, self.organization_id))
                .send()
                .await?;

            if response.status().is_success() {
                let activity_response: ActivityResponse = response.json().await?;
                let activity = activity_response.activity;

                if activity.status == "ACTIVITY_STATUS_COMPLETED" {
                    return Ok(activity);
                } else if activity.status == "ACTIVITY_STATUS_FAILED" {
                    return Err(TurnkeyError::ActivityFailed(
                        activity.result.map_or("Unknown error".to_string(), |r| r.error),
                    ));
                }
            }

            tokio::time::sleep(Duration::from_millis(ACTIVITY_POLL_INTERVAL_MS)).await;
        }
    }

    fn current_timestamp_ms(&self) -> String {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string()
    }
}

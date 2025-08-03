//! Turnkey-based signer implementation for Alloy
//!
//! This crate provides a signer implementation that uses Turnkey's secure infrastructure
//! for managing private keys and signing transactions.

pub mod error;
pub mod signer;

pub use error::{Result, TurnkeyError};
pub use signer::TurnkeySigner;

// Re-export key types from the official Turnkey SDK
pub use turnkey_client::TurnkeyP256ApiKey;

// Re-export Alloy signer trait
pub use alloy_signer::Signer;

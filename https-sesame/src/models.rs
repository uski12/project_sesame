use std::time::Instant;
use serde::{Deserialize, Serialize};


#[derive(Debug, Default)]
pub struct FailedIpInfo {
    pub attempts: u8,
    pub blocked_expiry: Option<Instant>,
}

#[derive(Deserialize)]
pub struct KnockRequest {
    pub passphrase: String,
    // timestamp: u64,
    // nonce: String,
}

#[derive(Serialize)]
pub struct KnockResponse {
    pub status: String,
}

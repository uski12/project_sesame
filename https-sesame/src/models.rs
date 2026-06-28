use std::time::Instant;
use serde::{
    Deserialize,
    // Serialize
};


#[derive(Debug, Default)]
pub struct FailedIpInfo {
    pub attempts: u8,
    pub blocked_expiry: Option<Instant>,
}

#[derive(Deserialize)]
pub struct KnockRequest {
    pub passphrase: String,
    pub timestamp: u64,
    pub nonce: String,
}

// #[derive(Serialize)]
// pub struct KnockResponse {
//     pub status: String,
// }

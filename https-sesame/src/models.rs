use serde::{
    Deserialize,
    // Serialize
};
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, RwLock},
};

use crate::{
    config::EnvConfig,
    database::Database,
};

use chrono::{DateTime, Utc};

#[derive(Debug, Default)]
pub struct FailedIpInfo {
    pub attempts: u8,
    pub blocked_expiry: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct AppState {
    pub database: Option<Arc<Database>>,
    pub authorised_ips: Arc<RwLock<HashMap<IpAddr, DateTime<Utc>>>>,
    pub failed_ips: Arc<RwLock<HashMap<IpAddr, FailedIpInfo>>>,
    pub used_nonces: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    pub config: EnvConfig,
}


#[derive(Deserialize)]
pub struct KnockRequest {
    pub passphrase: String,
    pub timestamp: DateTime<Utc>,
    pub nonce: String,
}

#[derive(Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

// #[derive(Serialize)]
// pub struct KnockResponse {
//     pub status: String,
// }

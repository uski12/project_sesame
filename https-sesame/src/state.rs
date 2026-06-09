use crate::{
    config::EnvConfig,
    models::FailedIpInfo,
};
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, RwLock},
    time::Instant,
};


#[derive(Clone)]
pub struct AppState {
    pub authorized_ips: Arc<RwLock<HashMap<IpAddr, Instant>>>,
    pub failed_ips: Arc<RwLock<HashMap<IpAddr, FailedIpInfo>>>,
    pub used_nonces: Arc<RwLock<HashMap<String, Instant>>>,
    pub config: EnvConfig,
}

use dotenvy::{
    from_path,
    dotenv,
};

use std::{
    net::IpAddr,
    env,
};

#[derive(Clone)]
pub struct EnvConfig {
    pub gateway_host: IpAddr,
    pub gateway_port: u16,

    pub server_host: IpAddr,
    pub server_port: u16,

    pub passphrase: String,
    pub auth_dur: u16,

    pub max_time_drift: u16,

    pub max_failed_attempts: u8,
    pub timeout_dur: u16,
}

impl EnvConfig {
    pub fn load_env() -> Self {
        from_path("../../.env").ok();
        dotenv().ok();

        Self {
            gateway_host: env::var("GATEWAY_HOST")
            .expect("GATEWAY_HOST field missing")
            .parse()
            .expect("Invalid GATEWAY_HOST field"),

            gateway_port: env::var("GATEWAY_PORT")
            .expect("GATEWAY_PORT field missing")
            .parse()
            .expect("Invalid GATEWAY_PORT field"),


            server_host: env::var("SERVER_HOST")
            .expect("SERVER_HOST field missing")
            .parse()
            .expect("Invalid SERVER_HOST field"),

            server_port: env::var("SERVER_PORT")
            .expect("SERVER_PORT field missing")
            .parse()
            .expect("Invalid SERVER_PORT field"),


            passphrase: env::var("PASSPHRASE")
            .expect("PASSPHRASE field missing"),

            auth_dur: env::var("AUTH_DURATION_SECONDS")
            .expect("AUTH_DURATION_SECONDS field missing")
            .parse()
            .expect("Invalid AUTH_DURATION_SECONDS field"),


            max_failed_attempts: env::var("MAX_FAILED_ATTEMPTS")
            .expect("MAX_FAILED_ATTEMPTS field missing")
            .parse()
            .expect("Invalid MAX_FAILED_ATTEMPTS field"),

            timeout_dur: env::var("TIMEOUT_DURATION_SECONDS")
            .expect("TIMEOUT_DURATION_SECONDS field missing")
            .parse()
            .expect("Invalid TIMEOUT_DURATION_SECONDS field"),
        }
    }
}

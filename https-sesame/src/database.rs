use sqlx::{PgPool, postgres::PgPoolOptions};
use chrono::{DateTime, Utc};
use std::net::IpAddr;
// use tracing::{info, warn};

use crate::models::{DatabaseConfig};

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn connect(config: &DatabaseConfig) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .connect(&config.url)
            .await?;

        Ok(Self { pool })
    }

    pub async fn init(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS authorised_ips (
                ip TEXT PRIMARY KEY,
                expires_at TIMESTAMPTZ NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS failed_ips (
                ip TEXT PRIMARY KEY,
                attempts INTEGER NOT NULL,
                blocked_until TIMESTAMPTZ
            );
            "#,
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS used_nonces (
                nonce TEXT PRIMARY KEY,
                used_at TIMESTAMPTZ NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn save_authorised_ip(
        &self,
        ip: IpAddr,
        expires_at: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO authorised_ips(ip, expires_at)
            VALUES ($1, $2)
            ON CONFLICT(ip)
            DO UPDATE SET expires_at = EXCLUDED.expires_at
            "#,
        )
        .bind(ip.to_string())
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn remove_authorised_ip(&self, ip: IpAddr) -> Result<(), sqlx::Error> {
        sqlx::query(
            "DELETE FROM authorised_ips WHERE ip = $1"
        )
        .bind(ip.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn load_authorised_ips(&self) -> Result<Vec<(IpAddr, DateTime<Utc>)>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (String, DateTime<Utc>)>("SELECT ip, expires_at FROM authorised_ips")
        .fetch_all(&self.pool)
        .await?;


        rows.into_iter()
            .map(|(ip, expires_at)| {
                ip.parse::<IpAddr>()
                    .map(|ip| (ip, expires_at))
                    .map_err(|_| {
                        sqlx::Error::Protocol(
                            format!("Invalid IP address in database: {ip}")
                        )
                    })
            })
            .collect()
    }

    pub async fn save_failed_ip(&self, ip: IpAddr,
                                 attempts: u8,
                                 blocked_until: Option<DateTime<Utc>>
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO failed_ips(ip, attempts, blocked_until)
            VALUES ($1, $2, $3)
            ON CONFLICT (ip)
            DO UPDATE SET attempts = EXCLUDED.attempts, blocked_until = EXCLUDED.blocked_until
            "#,
        )
        .bind(ip.to_string())
        .bind(attempts as i32)
        .bind(blocked_until)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn remove_failed_ip(&self, ip: IpAddr) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM failed_ips WHERE ip = $1")
        .bind(ip.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn load_failed_ips(&self) -> Result<Vec<(IpAddr, u8, Option<DateTime<Utc>>)>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (String, i32, Option<DateTime<Utc>>),>(
            "SELECT ip, attempts, blocked_until FROM failed_ips",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|(ip, attempts, blocked_until)| {
                let ip = ip.parse::<IpAddr>().map_err(|_| {
                    sqlx::Error::Protocol(format!("Invalid IP address in database: {ip}"))
                })?;


                let attempts = u8::try_from(attempts).map_err(|_| {
                    sqlx::Error::Protocol(format!("Invalid attempt count for {ip} (attempts)"))
                })?;

                Ok((ip, attempts, blocked_until))
            })
            .collect()
    }


    pub async fn save_nonce(&self, nonce: &str, used_at: DateTime<Utc>) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO used_nonces (nonce, used_at) VALUES ($1, $2) ON CONFLICT (nonce) DO NOTHING")
        .bind(nonce)
        .bind(used_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
    // pub async fn nonce_exists(&) {} NOT IMPLEMENTING - NONCES ALWAYS CHECKED IN-MEMORY.

    pub async fn load_nonces(&self) -> Result<Vec<(String, DateTime<Utc>)>, sqlx::Error> {
        sqlx::query_as::<_, (String, DateTime<Utc>)>("SELECT nonce, used_at FROM used_nonces")
        .fetch_all(&self.pool)
        .await
    }
}

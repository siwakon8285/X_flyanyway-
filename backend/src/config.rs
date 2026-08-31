use std::{env, net::SocketAddr, time::Duration};

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub database_url: String,
    pub bind_address: SocketAddr,
    pub frontend_origin: String,
    pub seat_hold_ttl: Duration,
    pub secure_cookies: bool,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, String> {
        let database_url =
            env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be configured".to_owned())?;
        let bind_address = env::var("BACKEND_BIND_ADDRESS")
            .unwrap_or_else(|_| "127.0.0.1:8080".to_owned())
            .parse()
            .map_err(|error| format!("invalid BACKEND_BIND_ADDRESS: {error}"))?;
        let seat_hold_ttl = env::var("SEAT_HOLD_TTL_SECONDS")
            .unwrap_or_else(|_| "600".to_owned())
            .parse::<u64>()
            .map(Duration::from_secs)
            .map_err(|error| format!("invalid SEAT_HOLD_TTL_SECONDS: {error}"))?;
        let secure_cookies = env::var("APP_ENV")
            .map(|value| value.eq_ignore_ascii_case("production"))
            .unwrap_or(false);
        let frontend_origin =
            env::var("FRONTEND_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".to_owned());

        Ok(Self {
            database_url,
            bind_address,
            frontend_origin,
            seat_hold_ttl,
            secure_cookies,
        })
    }
}

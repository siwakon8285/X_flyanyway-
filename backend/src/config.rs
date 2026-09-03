use std::{env, net::SocketAddr, time::Duration};

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub database_url: String,
    pub bind_address: SocketAddr,
    pub frontend_origin: String,
    pub seat_hold_ttl: Duration,
    pub secure_cookies: bool,
    pub stripe_secret_key: Option<String>,
    pub stripe_webhook_secret: Option<String>,
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
        let stripe_secret_key = env::var("STRIPE_SECRET_KEY").ok();
        let stripe_webhook_secret = env::var("STRIPE_WEBHOOK_SECRET").ok();
        for (name, value, prefix) in [
            (
                "STRIPE_SECRET_KEY",
                stripe_secret_key.as_deref(),
                "sk_test_",
            ),
            (
                "STRIPE_WEBHOOK_SECRET",
                stripe_webhook_secret.as_deref(),
                "whsec_",
            ),
        ] {
            if let Some(value) = value {
                if value.starts_with("sk_live_") || !value.starts_with(prefix) {
                    return Err(format!(
                        "{name} must be a Stripe Test Mode key with the required prefix"
                    ));
                }
            }
        }
        if stripe_secret_key.is_some() != stripe_webhook_secret.is_some() {
            return Err("STRIPE_SECRET_KEY and STRIPE_WEBHOOK_SECRET must be configured together to enable Card payments".to_owned());
        }

        Ok(Self {
            database_url,
            bind_address,
            frontend_origin,
            stripe_secret_key,
            stripe_webhook_secret,
            seat_hold_ttl,
            secure_cookies,
        })
    }
}

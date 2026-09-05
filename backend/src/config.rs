use std::{env, net::SocketAddr, time::Duration};

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub database_url: String,
    pub bind_address: SocketAddr,
    pub frontend_origin: String,
    pub public_site_origin: String,
    pub email_transport: String,
    pub resend_api_key: Option<String>,
    pub email_from: Option<String>,
    pub seat_hold_ttl: Duration,
    pub secure_cookies: bool,
    pub stripe_secret_key: Option<String>,
    pub stripe_webhook_secret: Option<String>,
    pub ticket_qr_signing_secret: String,
    pub manage_booking_signing_secret: String,
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
        let public_site_origin = validate_public_site_origin(
            &env::var("PUBLIC_SITE_ORIGIN").unwrap_or_else(|_| frontend_origin.clone()),
            secure_cookies,
        )?;
        let email_transport = env::var("EMAIL_TRANSPORT").unwrap_or_else(|_| "disabled".to_owned());
        if !email_transport.eq_ignore_ascii_case("disabled")
            && !email_transport.eq_ignore_ascii_case("resend")
        {
            return Err("EMAIL_TRANSPORT must be disabled or resend".to_owned());
        }
        let resend_api_key = env::var("RESEND_API_KEY").ok();
        let email_from = env::var("EMAIL_FROM")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(|value| validate_email_from(&value))
            .transpose()?;
        if email_transport.eq_ignore_ascii_case("resend")
            && (resend_api_key.is_none() || email_from.is_none())
        {
            return Err(
                "RESEND_API_KEY and EMAIL_FROM must be configured when EMAIL_TRANSPORT=resend"
                    .to_owned(),
            );
        }
        let stripe_secret_key = env::var("STRIPE_SECRET_KEY").ok();
        let stripe_webhook_secret = env::var("STRIPE_WEBHOOK_SECRET").ok();
        let ticket_qr_signing_secret = env::var("TICKET_QR_SIGNING_SECRET")
            .map_err(|_| "TICKET_QR_SIGNING_SECRET must be configured".to_owned())?;
        if ticket_qr_signing_secret.len() < 32 {
            return Err("TICKET_QR_SIGNING_SECRET must be at least 32 characters".to_owned());
        }
        let manage_booking_signing_secret = env::var("MANAGE_BOOKING_SIGNING_SECRET")
            .map_err(|_| "MANAGE_BOOKING_SIGNING_SECRET must be configured".to_owned())?;
        if manage_booking_signing_secret.len() < 32 {
            return Err("MANAGE_BOOKING_SIGNING_SECRET must be at least 32 characters".to_owned());
        }
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
            public_site_origin,
            email_transport,
            resend_api_key,
            email_from,
            stripe_secret_key,
            stripe_webhook_secret,
            ticket_qr_signing_secret,
            manage_booking_signing_secret,
            seat_hold_ttl,
            secure_cookies,
        })
    }
}

fn validate_public_site_origin(value: &str, production: bool) -> Result<String, String> {
    let parsed = reqwest::Url::parse(value)
        .map_err(|_| "PUBLIC_SITE_ORIGIN must be an absolute URL".to_owned())?;
    if !matches!(parsed.scheme(), "http" | "https")
        || parsed.host_str().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.path() != "/" && !parsed.path().is_empty()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
    {
        return Err(
            "PUBLIC_SITE_ORIGIN must be an HTTP(S) origin without a path, query, or fragment"
                .to_owned(),
        );
    }
    if production && parsed.scheme() != "https" {
        return Err("PUBLIC_SITE_ORIGIN must use HTTPS in production".to_owned());
    }
    Ok(value.trim_end_matches('/').to_owned())
}

fn validate_email_from(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed
            .chars()
            .any(|character| character == '\r' || character == '\n' || character.is_control())
    {
        return Err("EMAIL_FROM must be a non-empty sender without control characters".to_owned());
    }
    let address = trimmed
        .rsplit_once('<')
        .map(|(_, address)| address.trim_end_matches('>').trim())
        .unwrap_or(trimmed);
    if !address.contains('@')
        || address.starts_with('@')
        || address.ends_with('@')
        || address.contains(' ')
    {
        return Err("EMAIL_FROM must contain a valid sender address".to_owned());
    }
    Ok(trimmed.to_owned())
}

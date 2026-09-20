use hmac::{Hmac, Mac};
use sha2::Sha256;
use thiserror::Error;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

const VERSION: &str = "v1";
const PURPOSE: &str = "x-fly/boarding-pass";

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum QrTokenError {
    #[error("boarding pass QR token is invalid")]
    Invalid,
    #[error("boarding pass QR signing secret is invalid")]
    InvalidSecret,
}

fn message(boarding_pass_id: Uuid) -> String {
    format!("{PURPOSE}.{VERSION}.{}", boarding_pass_id.hyphenated())
}

pub fn sign(boarding_pass_id: Uuid, secret: &str) -> Result<String, QrTokenError> {
    let message = message(boarding_pass_id);
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| QrTokenError::InvalidSecret)?;
    mac.update(message.as_bytes());
    Ok(format!(
        "{VERSION}.{}.{}",
        boarding_pass_id.hyphenated(),
        hex::encode(mac.finalize().into_bytes())
    ))
}

pub fn verify(token: &str, secret: &str) -> Result<Uuid, QrTokenError> {
    let mut parts = token.split('.');
    let (Some(version), Some(id), Some(signature), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(QrTokenError::Invalid);
    };
    if version != VERSION {
        return Err(QrTokenError::Invalid);
    }
    let boarding_pass_id = Uuid::parse_str(id).map_err(|_| QrTokenError::Invalid)?;
    let signature = hex::decode(signature).map_err(|_| QrTokenError::Invalid)?;
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| QrTokenError::InvalidSecret)?;
    mac.update(message(boarding_pass_id).as_bytes());
    mac.verify_slice(&signature)
        .map_err(|_| QrTokenError::Invalid)?;
    Ok(boarding_pass_id)
}

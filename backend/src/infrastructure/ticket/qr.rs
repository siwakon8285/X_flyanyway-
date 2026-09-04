use hmac::{Hmac, Mac};
use sha2::Sha256;
use thiserror::Error;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

const VERSION: &str = "v1";

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum QrTokenError {
    #[error("ticket QR token is invalid")]
    Invalid,
    #[error("ticket QR signing secret is invalid")]
    InvalidSecret,
}

pub fn sign(ticket_id: Uuid, secret: &str) -> Result<String, QrTokenError> {
    let message = format!("{VERSION}.{}", ticket_id.hyphenated());
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| QrTokenError::InvalidSecret)?;
    mac.update(message.as_bytes());
    Ok(format!(
        "{message}.{}",
        hex::encode(mac.finalize().into_bytes())
    ))
}

pub fn verify(token: &str, secret: &str) -> Result<Uuid, QrTokenError> {
    let mut parts = token.split('.');
    let (Some(version), Some(ticket_id), Some(signature), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(QrTokenError::Invalid);
    };
    if version != VERSION {
        return Err(QrTokenError::Invalid);
    }
    let ticket_id = Uuid::parse_str(ticket_id).map_err(|_| QrTokenError::Invalid)?;
    let signature = hex::decode(signature).map_err(|_| QrTokenError::Invalid)?;
    let message = format!("{VERSION}.{}", ticket_id.hyphenated());
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| QrTokenError::InvalidSecret)?;
    mac.update(message.as_bytes());
    mac.verify_slice(&signature)
        .map_err(|_| QrTokenError::Invalid)?;
    Ok(ticket_id)
}

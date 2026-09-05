use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use thiserror::Error;
use uuid::Uuid;

const VERSION: &str = "v1";

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum ManageBookingAccessError {
    #[error("manage booking access is invalid")]
    Invalid,
    #[error("manage booking access has expired")]
    Expired,
}

pub fn sign(
    ticket_id: Uuid,
    expires_at: DateTime<Utc>,
    nonce: [u8; 16],
    secret: &str,
) -> Result<String, ManageBookingAccessError> {
    let message = format!(
        "{VERSION}.{}.{}.{}",
        ticket_id.hyphenated(),
        expires_at.timestamp(),
        hex::encode(nonce)
    );
    let signature = signature(&message, secret)?;
    Ok(format!("{message}.{}", hex::encode(signature)))
}

pub fn verify(
    token: &str,
    secret: &str,
    now: DateTime<Utc>,
) -> Result<Uuid, ManageBookingAccessError> {
    let parts: Vec<_> = token.split('.').collect();
    if parts.len() != 5 || parts[0] != VERSION {
        return Err(ManageBookingAccessError::Invalid);
    }
    let ticket_id = Uuid::parse_str(parts[1]).map_err(|_| ManageBookingAccessError::Invalid)?;
    let expires_timestamp = parts[2]
        .parse::<i64>()
        .map_err(|_| ManageBookingAccessError::Invalid)?;
    let nonce = hex::decode(parts[3]).map_err(|_| ManageBookingAccessError::Invalid)?;
    let supplied = hex::decode(parts[4]).map_err(|_| ManageBookingAccessError::Invalid)?;
    if nonce.len() != 16 || supplied.len() != 32 {
        return Err(ManageBookingAccessError::Invalid);
    }
    let message = parts[..4].join(".");
    let expected = signature(&message, secret)?;
    if expected.ct_eq(&supplied).unwrap_u8() != 1 {
        return Err(ManageBookingAccessError::Invalid);
    }
    if now.timestamp() >= expires_timestamp {
        return Err(ManageBookingAccessError::Expired);
    }
    Ok(ticket_id)
}

fn signature(message: &str, secret: &str) -> Result<[u8; 32], ManageBookingAccessError> {
    if secret.len() < 32 {
        return Err(ManageBookingAccessError::Invalid);
    }
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .map_err(|_| ManageBookingAccessError::Invalid)?;
    mac.update(message.as_bytes());
    Ok(mac.finalize().into_bytes().into())
}

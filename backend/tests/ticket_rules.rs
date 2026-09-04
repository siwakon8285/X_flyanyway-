use uuid::Uuid;

use x_fly_api::infrastructure::ticket::qr::{sign, verify, QrTokenError};

const SECRET: &str = "ticket-signing-secret-for-unit-tests-must-be-long-enough";

#[test]
fn signed_ticket_token_verifies_only_for_its_original_ticket() {
    let ticket_id = Uuid::parse_str("018f6f52-2247-7b29-8d22-111111111111").unwrap();
    let token = sign(ticket_id, SECRET).unwrap();

    assert_eq!(verify(&token, SECRET), Ok(ticket_id));
    assert_eq!(
        verify(
            &token,
            "a-different-ticket-signing-secret-that-is-long-enough"
        ),
        Err(QrTokenError::Invalid)
    );
}

#[test]
fn tampered_or_malformed_ticket_token_is_invalid() {
    let ticket_id = Uuid::parse_str("018f6f52-2247-7b29-8d22-111111111111").unwrap();
    let token = sign(ticket_id, SECRET).unwrap();
    let tampered = format!("{}x", token);

    assert_eq!(verify(&tampered, SECRET), Err(QrTokenError::Invalid));
    assert_eq!(
        verify("v1.not-a-uuid.deadbeef", SECRET),
        Err(QrTokenError::Invalid)
    );
}

use uuid::Uuid;
use x_fly_api::infrastructure::{boarding_pass::qr as boarding_pass_qr, ticket::qr as ticket_qr};

#[test]
fn boarding_pass_tokens_are_purpose_separated_and_tamper_evident() {
    let id = Uuid::new_v4();
    let secret = "boarding-pass-test-secret-with-at-least-32-bytes";
    let token = boarding_pass_qr::sign(id, secret).unwrap();

    assert!(token.starts_with("v1."));
    assert_eq!(boarding_pass_qr::verify(&token, secret).unwrap(), id);
    assert!(boarding_pass_qr::verify(&ticket_qr::sign(id, secret).unwrap(), secret).is_err());

    let mut tampered = token.clone();
    tampered.push('0');
    assert!(boarding_pass_qr::verify(&tampered, secret).is_err());
}

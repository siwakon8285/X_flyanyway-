use x_fly_api::domain::payment::{
    build_demo_bitcoin_invoice, demo_btc_satoshis, PaymentStatus, PaymentTransitionError,
    DEMO_BTC_RATE_THB_PER_BTC,
};
use x_fly_api::{domain::extras::Money, infrastructure::payment::stripe::StripeAmount};

#[test]
fn stripe_thb_amount_conversion_is_exact_and_checked() {
    let stripe = StripeAmount::from_xfly_money(&Money {
        amount: 23_400,
        currency_code: "THB".to_owned(),
    })
    .unwrap();
    assert_eq!(stripe.amount(), 2_340_000);
    assert_eq!(stripe.currency(), "thb");
    assert_eq!(stripe.to_xfly_money().unwrap().amount, 23_400);

    assert!(StripeAmount::from_xfly_money(&Money {
        amount: 1,
        currency_code: "USD".to_owned(),
    })
    .is_err());
    assert!(StripeAmount::from_xfly_money(&Money {
        amount: i64::MAX,
        currency_code: "THB".to_owned(),
    })
    .is_err());
}

#[test]
fn converts_authoritative_whole_baht_to_fixed_rate_demo_satoshis() {
    assert_eq!(DEMO_BTC_RATE_THB_PER_BTC, 2_000_000);
    assert_eq!(demo_btc_satoshis(49_300), Ok(2_465_000));
    assert_eq!(demo_btc_satoshis(1), Ok(50));
}

#[test]
fn rejects_invalid_demo_btc_amounts() {
    assert!(demo_btc_satoshis(0).is_err());
    assert!(demo_btc_satoshis(-1).is_err());
}

#[test]
fn enforces_the_small_payment_state_machine() {
    assert_eq!(
        PaymentStatus::Created.transition(PaymentStatus::Processing),
        Ok(PaymentStatus::Processing)
    );
    assert_eq!(
        PaymentStatus::Created.transition(PaymentStatus::AwaitingPayment),
        Ok(PaymentStatus::AwaitingPayment)
    );
    assert_eq!(
        PaymentStatus::Processing.transition(PaymentStatus::Succeeded),
        Ok(PaymentStatus::Succeeded)
    );
    assert_eq!(
        PaymentStatus::AwaitingPayment.transition(PaymentStatus::Cancelled),
        Ok(PaymentStatus::Cancelled)
    );
    assert_eq!(
        PaymentStatus::Succeeded.transition(PaymentStatus::Failed),
        Err(PaymentTransitionError::InvalidTransition)
    );
    assert_eq!(
        PaymentStatus::Failed.transition(PaymentStatus::Processing),
        Err(PaymentTransitionError::InvalidTransition)
    );
}

#[test]
fn builds_an_unmistakably_invalid_deterministic_demo_bitcoin_invoice() {
    let attempt_id = uuid::Uuid::parse_str("018f6f52-2247-7b29-8d22-111111111111").unwrap();
    let invoice = build_demo_bitcoin_invoice(attempt_id, 49_300, "XFBTC-DEMO").unwrap();

    assert_eq!(invoice.amount_satoshis, 2_465_000);
    assert_eq!(invoice.display_amount, "0.02465000");
    assert_eq!(
        invoice.demo_address,
        "DEMO-ONLY-NOT-A-BITCOIN-ADDRESS-018F6F522247"
    );
    assert_eq!(invoice.invoice_reference, "XFBTC-DEMO");
    assert_eq!(invoice.rate_thb_per_btc, 2_000_000);
}

#[test]
fn stripe_webhook_signature_verification_enforces_security_rules() {
    use x_fly_api::infrastructure::payment::stripe::{
        compute_stripe_signature, verify_stripe_signature, WebhookVerificationError,
        STRIPE_SIGNATURE_TOLERANCE_SECONDS,
    };

    let secret = "whsec_test_secret_for_unit_tests_12345";
    let now = 1_700_000_000;
    let payload = b"{\"id\":\"evt_123\",\"type\":\"payment_intent.succeeded\"}";
    let valid_sig = compute_stripe_signature(now, payload, secret).unwrap();

    // 1. Missing header
    assert_eq!(
        verify_stripe_signature("", payload, secret, now, STRIPE_SIGNATURE_TOLERANCE_SECONDS),
        Err(WebhookVerificationError::MissingHeader)
    );

    // 2. Malformed headers
    assert_eq!(
        verify_stripe_signature(
            "not_a_valid_header",
            payload,
            secret,
            now,
            STRIPE_SIGNATURE_TOLERANCE_SECONDS
        ),
        Err(WebhookVerificationError::MalformedHeader)
    );
    assert_eq!(
        verify_stripe_signature(
            &format!("v1={valid_sig}"),
            payload,
            secret,
            now,
            STRIPE_SIGNATURE_TOLERANCE_SECONDS
        ),
        Err(WebhookVerificationError::MalformedHeader)
    );
    assert_eq!(
        verify_stripe_signature(
            &format!("t=notanumber,v1={valid_sig}"),
            payload,
            secret,
            now,
            STRIPE_SIGNATURE_TOLERANCE_SECONDS
        ),
        Err(WebhookVerificationError::MalformedHeader)
    );
    assert_eq!(
        verify_stripe_signature(
            &format!("t={now}"),
            payload,
            secret,
            now,
            STRIPE_SIGNATURE_TOLERANCE_SECONDS
        ),
        Err(WebhookVerificationError::MalformedHeader)
    );

    // 3. Invalid signature
    let header_invalid = format!("t={now},v1=deadbeefcafebabe0123456789abcdef");
    assert_eq!(
        verify_stripe_signature(
            &header_invalid,
            payload,
            secret,
            now,
            STRIPE_SIGNATURE_TOLERANCE_SECONDS
        ),
        Err(WebhookVerificationError::InvalidSignature)
    );

    // 4. Valid signature accepted
    let header_valid = format!("t={now},v1={valid_sig}");
    assert_eq!(
        verify_stripe_signature(
            &header_valid,
            payload,
            secret,
            now,
            STRIPE_SIGNATURE_TOLERANCE_SECONDS
        ),
        Ok(())
    );

    // 5. Stale timestamp rejected
    let stale_past = now - STRIPE_SIGNATURE_TOLERANCE_SECONDS - 1;
    let stale_past_sig = compute_stripe_signature(stale_past, payload, secret).unwrap();
    assert_eq!(
        verify_stripe_signature(
            &format!("t={stale_past},v1={stale_past_sig}"),
            payload,
            secret,
            now,
            STRIPE_SIGNATURE_TOLERANCE_SECONDS
        ),
        Err(WebhookVerificationError::TimestampOutOfTolerance)
    );
    let stale_future = now + STRIPE_SIGNATURE_TOLERANCE_SECONDS + 1;
    let stale_future_sig = compute_stripe_signature(stale_future, payload, secret).unwrap();
    assert_eq!(
        verify_stripe_signature(
            &format!("t={stale_future},v1={stale_future_sig}"),
            payload,
            secret,
            now,
            STRIPE_SIGNATURE_TOLERANCE_SECONDS
        ),
        Err(WebhookVerificationError::TimestampOutOfTolerance)
    );

    // 6. Multiple v1 signatures supported
    let header_multiple = format!("t={now},v1=bad_signature,v1={valid_sig}");
    assert_eq!(
        verify_stripe_signature(
            &header_multiple,
            payload,
            secret,
            now,
            STRIPE_SIGNATURE_TOLERANCE_SECONDS
        ),
        Ok(())
    );

    // 7. Raw-body verification: altering any byte fails
    let modified_payload = b"{\"id\": \"evt_123\",\"type\":\"payment_intent.succeeded\"}";
    assert_eq!(
        verify_stripe_signature(
            &header_valid,
            modified_payload,
            secret,
            now,
            STRIPE_SIGNATURE_TOLERANCE_SECONDS
        ),
        Err(WebhookVerificationError::InvalidSignature)
    );
}

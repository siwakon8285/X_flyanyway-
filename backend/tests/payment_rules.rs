use x_fly_api::domain::payment::{
    build_demo_bitcoin_invoice, demo_btc_satoshis, PaymentStatus, PaymentTransitionError,
    DEMO_BTC_RATE_THB_PER_BTC,
};

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

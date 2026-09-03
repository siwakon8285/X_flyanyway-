ALTER TABLE payment_attempts
    DROP CONSTRAINT payment_attempts_provider_check,
    DROP CONSTRAINT payment_attempts_check,
    ADD COLUMN payment_finalization_deadline TIMESTAMPTZ;

UPDATE payment_attempts AS attempt
SET provider = 'STRIPE',
    payment_finalization_deadline = hold.expires_at + INTERVAL '5 minutes',
    provider_reference = CASE
        WHEN attempt.provider_reference IS NULL THEN NULL
        ELSE 'legacy_mock_card_' || attempt.id::TEXT
    END
FROM seat_holds AS hold
WHERE attempt.seat_hold_id = hold.id
  AND attempt.provider = 'MOCK_CARD'
  AND attempt.payment_method = 'CARD';

ALTER TABLE payment_attempts
    ADD CONSTRAINT payment_attempts_provider_check CHECK (
        provider IN ('STRIPE', 'MOCK_BITCOIN')
    ),
    ADD CONSTRAINT payment_attempts_provider_payment_method_check CHECK (
        (provider = 'STRIPE' AND payment_method = 'CARD')
        OR (provider = 'MOCK_BITCOIN' AND payment_method = 'BITCOIN')
    ),
    ADD CONSTRAINT payment_attempts_finalization_reservation_check CHECK (
        (provider = 'STRIPE' AND payment_method = 'CARD')
        = (payment_finalization_deadline IS NOT NULL)
    );

COMMENT ON COLUMN payment_attempts.payment_finalization_deadline IS
    'Immutable policy boundary set to the original seat hold expiry plus five minutes; unresolved Stripe Card attempts remain inventory-protected even after this timestamp.';

CREATE FUNCTION has_protected_stripe_card_finalization(target_hold_id UUID)
RETURNS BOOLEAN
LANGUAGE SQL
STABLE
PARALLEL SAFE
AS $$
    SELECT EXISTS (
        SELECT 1
        FROM payment_attempts
        WHERE seat_hold_id = target_hold_id
          AND provider = 'STRIPE'
          AND payment_method = 'CARD'
          AND status IN ('CREATED', 'PROCESSING', 'AWAITING_PAYMENT')
          AND payment_finalization_deadline IS NOT NULL
    )
$$;

CREATE INDEX idx_payment_attempts_protected_stripe_finalization
    ON payment_attempts (seat_hold_id)
    WHERE provider = 'STRIPE'
      AND payment_method = 'CARD'
      AND status IN ('CREATED', 'PROCESSING', 'AWAITING_PAYMENT')
      AND payment_finalization_deadline IS NOT NULL;

CREATE UNIQUE INDEX idx_payment_attempts_stripe_reference
    ON payment_attempts (provider_reference)
    WHERE provider = 'STRIPE' AND provider_reference IS NOT NULL;

CREATE TABLE stripe_webhook_events (
    stripe_event_id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    payment_intent_id TEXT NOT NULL,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

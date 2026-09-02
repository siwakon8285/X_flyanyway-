-- Mock payment attempts copy the authoritative Review amount. They never store card data,
-- hold authorization, Bitcoin key material, or provider secrets.
CREATE TABLE payment_attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    seat_hold_id UUID NOT NULL REFERENCES seat_holds(id) ON DELETE CASCADE,
    request_id UUID NOT NULL,
    request_fingerprint BYTEA NOT NULL CHECK (octet_length(request_fingerprint) = 32),
    provider TEXT NOT NULL CHECK (provider IN ('MOCK_CARD', 'MOCK_BITCOIN')),
    payment_method TEXT NOT NULL CHECK (payment_method IN ('CARD', 'BITCOIN')),
    status TEXT NOT NULL CHECK (status IN (
        'CREATED', 'PROCESSING', 'AWAITING_PAYMENT', 'SUCCEEDED', 'FAILED', 'CANCELLED'
    )),
    amount BIGINT NOT NULL CHECK (amount > 0),
    currency_code TEXT NOT NULL CHECK (currency_code ~ '^[A-Z]{3}$'),
    review_priced_at TIMESTAMPTZ NOT NULL,
    provider_reference TEXT,
    failure_code TEXT,
    failure_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    succeeded_at TIMESTAMPTZ,
    UNIQUE (seat_hold_id, request_id),
    CHECK (
        (provider = 'MOCK_CARD' AND payment_method = 'CARD')
        OR (provider = 'MOCK_BITCOIN' AND payment_method = 'BITCOIN')
    ),
    CHECK ((status = 'SUCCEEDED') = (succeeded_at IS NOT NULL)),
    CHECK ((status IN ('FAILED', 'CANCELLED')) = (failure_code IS NOT NULL AND failure_message IS NOT NULL))
);

CREATE UNIQUE INDEX idx_payment_attempts_one_success_per_hold
    ON payment_attempts (seat_hold_id)
    WHERE status = 'SUCCEEDED';

CREATE UNIQUE INDEX idx_payment_attempts_one_open_per_hold
    ON payment_attempts (seat_hold_id)
    WHERE status IN ('CREATED', 'PROCESSING', 'AWAITING_PAYMENT');

CREATE INDEX idx_payment_attempts_hold_created
    ON payment_attempts (seat_hold_id, created_at DESC);

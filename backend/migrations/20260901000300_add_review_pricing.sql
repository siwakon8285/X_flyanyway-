-- Review reference data remains nullable at the schema level so a failed/missing demo seed
-- produces REVIEW_PRICING_UNAVAILABLE rather than a fabricated fare or schedule.
ALTER TABLE flight_services
    ADD COLUMN departure_time TIME,
    ADD COLUMN arrival_time TIME,
    ADD COLUMN arrival_day_offset SMALLINT CHECK (arrival_day_offset IN (0, 1)),
    ADD COLUMN duration_minutes SMALLINT CHECK (duration_minutes > 0),
    ADD COLUMN stops TEXT CHECK (stops IN ('DIRECT', 'ONE_STOP'));

ALTER TABLE flight_service_cabins
    ADD COLUMN base_fare_amount BIGINT CHECK (base_fare_amount > 0),
    ADD COLUMN currency_code TEXT CHECK (currency_code ~ '^[A-Z]{3}$'),
    ADD CONSTRAINT flight_service_cabin_fare_complete CHECK (
        (base_fare_amount IS NULL AND currency_code IS NULL)
        OR (base_fare_amount IS NOT NULL AND currency_code IS NOT NULL)
    );

-- An idempotent internal materialization/cache of authoritative Review pricing only.
-- This is not payment, booking confirmation, ticket issuance, or a general precedent
-- for arbitrary GET side effects. One hold can own at most one current snapshot.
CREATE TABLE hold_review_pricing (
    seat_hold_id UUID PRIMARY KEY REFERENCES seat_holds(id) ON DELETE CASCADE,
    source_extras_saved_at TIMESTAMPTZ NOT NULL,
    currency_code TEXT NOT NULL CHECK (currency_code ~ '^[A-Z]{3}$'),
    seated_base_fare_unit_amount BIGINT NOT NULL CHECK (seated_base_fare_unit_amount > 0),
    infant_base_fare_unit_amount BIGINT NOT NULL CHECK (infant_base_fare_unit_amount = 0),
    base_fare_amount BIGINT NOT NULL CHECK (base_fare_amount >= 0),
    extras_amount BIGINT NOT NULL CHECK (extras_amount >= 0),
    demo_passenger_tax_unit_amount BIGINT NOT NULL CHECK (demo_passenger_tax_unit_amount >= 0),
    taxes_amount BIGINT NOT NULL CHECK (taxes_amount >= 0),
    demo_airport_fee_unit_amount BIGINT NOT NULL CHECK (demo_airport_fee_unit_amount >= 0),
    demo_airport_fee_amount BIGINT NOT NULL CHECK (demo_airport_fee_amount >= 0),
    demo_booking_fee_amount BIGINT NOT NULL CHECK (demo_booking_fee_amount >= 0),
    fees_amount BIGINT NOT NULL CHECK (fees_amount >= 0),
    grand_total_amount BIGINT NOT NULL CHECK (grand_total_amount >= 0),
    priced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (grand_total_amount = base_fare_amount + extras_amount + taxes_amount + fees_amount),
    CHECK (fees_amount = demo_airport_fee_amount + demo_booking_fee_amount)
);

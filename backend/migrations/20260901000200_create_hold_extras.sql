-- extras_saved_at is only the timestamp at which the customer explicitly reviewed and
-- saved the Travel Extras workflow step. It is not a payment, booking confirmation,
-- or ticket-issuance timestamp and must not be reused as one by later booking branches.
ALTER TABLE seat_holds
    ADD COLUMN extras_saved_at TIMESTAMPTZ;

CREATE TABLE hold_extras (
    seat_hold_id UUID NOT NULL REFERENCES seat_holds(id) ON DELETE CASCADE,
    passenger_ordinal SMALLINT NOT NULL CHECK (passenger_ordinal >= 1),
    product_code TEXT NOT NULL CHECK (product_code ~ '^[A-Z0-9_]+$'),
    category TEXT NOT NULL CHECK (category IN ('BAGGAGE', 'MEAL', 'ASSISTANCE')),
    quantity SMALLINT NOT NULL CHECK (quantity = 1),
    unit_price_amount BIGINT NOT NULL CHECK (unit_price_amount >= 0),
    currency_code TEXT NOT NULL CHECK (currency_code ~ '^[A-Z]{3}$'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (seat_hold_id, passenger_ordinal, product_code)
);

CREATE INDEX idx_hold_extras_hold_ordinal
    ON hold_extras (seat_hold_id, passenger_ordinal);

CREATE UNIQUE INDEX idx_hold_extras_one_baggage_per_passenger
    ON hold_extras (seat_hold_id, passenger_ordinal)
    WHERE category = 'BAGGAGE';

CREATE UNIQUE INDEX idx_hold_extras_one_meal_per_passenger
    ON hold_extras (seat_hold_id, passenger_ordinal)
    WHERE category = 'MEAL';

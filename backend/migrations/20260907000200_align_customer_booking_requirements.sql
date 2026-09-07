-- Branch 20A keeps historical contact data intact while allowing the active
-- phone-only customer flow to persist no email on passengers or bookings.
ALTER TABLE hold_passengers
    ALTER COLUMN email DROP NOT NULL,
    ALTER COLUMN phone_country_code DROP NOT NULL,
    ALTER COLUMN phone_number DROP NOT NULL;

ALTER TABLE booking_contacts
    ALTER COLUMN email DROP NOT NULL,
    ADD COLUMN phone_country_code TEXT,
    ADD COLUMN phone_number TEXT,
    ALTER COLUMN preferred_locale SET DEFAULT 'EN',
    ADD CONSTRAINT booking_contact_phone_complete CHECK (
        (
            phone_country_code IS NULL
            AND phone_number IS NULL
        )
        OR
        (
            phone_country_code ~ '^\+[1-9][0-9]{0,2}$'
            AND phone_number ~ '^[0-9]{4,14}$'
        )
    );

-- Review pricing remains an authoritative snapshot, but its freshness is now
-- tied to passenger details rather than the removed Travel Extras step.
ALTER TABLE seat_holds
    ADD COLUMN passenger_details_saved_at TIMESTAMPTZ;

UPDATE seat_holds AS hold
SET passenger_details_saved_at = COALESCE(hold.extras_saved_at, hold.updated_at)
WHERE EXISTS (
    SELECT 1 FROM hold_passengers AS passenger
    WHERE passenger.seat_hold_id = hold.id
);

ALTER TABLE hold_review_pricing
    RENAME COLUMN source_extras_saved_at TO source_passenger_details_saved_at;

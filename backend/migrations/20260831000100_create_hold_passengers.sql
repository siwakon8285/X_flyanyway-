CREATE TABLE hold_passengers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    seat_hold_id UUID NOT NULL REFERENCES seat_holds(id) ON DELETE CASCADE,
    ordinal SMALLINT NOT NULL CHECK (ordinal >= 1),
    passenger_type TEXT NOT NULL CHECK (passenger_type IN ('ADULT', 'CHILD', 'INFANT')),
    title TEXT NOT NULL CHECK (title IN ('MR', 'MS', 'MRS', 'MX')),
    given_name TEXT NOT NULL CHECK (length(btrim(given_name)) > 0),
    middle_name TEXT,
    family_name TEXT NOT NULL CHECK (length(btrim(family_name)) > 0),
    date_of_birth DATE NOT NULL,
    gender TEXT NOT NULL CHECK (gender IN ('MALE', 'FEMALE', 'UNSPECIFIED')),
    nationality_code TEXT NOT NULL CHECK (nationality_code ~ '^[A-Z]{2}$'),
    passport_number TEXT NOT NULL CHECK (passport_number ~ '^[A-Z0-9]{3,20}$'),
    passport_issuing_country_code TEXT NOT NULL CHECK (passport_issuing_country_code ~ '^[A-Z]{2}$'),
    passport_issue_date DATE,
    passport_expiry_date DATE NOT NULL,
    email TEXT NOT NULL CHECK (length(btrim(email)) > 0),
    phone_country_code TEXT NOT NULL CHECK (phone_country_code ~ '^\+[1-9][0-9]{0,2}$'),
    phone_number TEXT NOT NULL CHECK (phone_number ~ '^[0-9]{4,14}$'),
    emergency_contact_name TEXT,
    emergency_contact_relationship TEXT,
    emergency_contact_phone_country_code TEXT,
    emergency_contact_phone_number TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (seat_hold_id, ordinal),
    UNIQUE (seat_hold_id, passport_number),
    CHECK (middle_name IS NULL OR length(btrim(middle_name)) > 0),
    CHECK (passport_issue_date IS NULL OR passport_issue_date < passport_expiry_date),
    CHECK (
        (
            emergency_contact_name IS NULL
            AND emergency_contact_relationship IS NULL
            AND emergency_contact_phone_country_code IS NULL
            AND emergency_contact_phone_number IS NULL
        )
        OR
        (
            emergency_contact_name IS NOT NULL
            AND emergency_contact_relationship IS NOT NULL
            AND emergency_contact_phone_country_code IS NOT NULL
            AND emergency_contact_phone_number IS NOT NULL
            AND
            length(btrim(emergency_contact_name)) > 0
            AND length(btrim(emergency_contact_relationship)) > 0
            AND emergency_contact_phone_country_code ~ '^\+[1-9][0-9]{0,2}$'
            AND emergency_contact_phone_number ~ '^[0-9]{4,14}$'
        )
    )
);

CREATE INDEX idx_hold_passengers_hold_ordinal
    ON hold_passengers (seat_hold_id, ordinal);

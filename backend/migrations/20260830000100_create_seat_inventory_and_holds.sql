CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE flight_services (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    public_id TEXT NOT NULL UNIQUE,
    flight_number TEXT NOT NULL UNIQUE,
    origin_code TEXT NOT NULL CHECK (origin_code ~ '^[A-Z]{3}$'),
    destination_code TEXT NOT NULL CHECK (destination_code ~ '^[A-Z]{3}$'),
    aircraft_code TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (origin_code <> destination_code)
);

CREATE TABLE flight_service_cabins (
    flight_service_id UUID NOT NULL REFERENCES flight_services(id) ON DELETE CASCADE,
    cabin TEXT NOT NULL CHECK (cabin IN ('economy', 'premium-economy', 'business', 'first')),
    PRIMARY KEY (flight_service_id, cabin)
);

CREATE TABLE aircraft_seat_templates (
    aircraft_code TEXT NOT NULL,
    seat_number TEXT NOT NULL CHECK (seat_number ~ '^[1-9][0-9]*[A-Z]+$'),
    row_number SMALLINT NOT NULL CHECK (row_number > 0),
    column_code TEXT NOT NULL CHECK (column_code ~ '^[A-Z]+$'),
    cabin TEXT NOT NULL CHECK (cabin IN ('economy', 'premium-economy', 'business', 'first')),
    position TEXT NOT NULL CHECK (position IN ('window', 'middle', 'aisle')),
    sellable BOOLEAN NOT NULL DEFAULT TRUE,
    PRIMARY KEY (aircraft_code, cabin, seat_number)
);

CREATE TABLE flight_instances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    flight_service_id UUID NOT NULL REFERENCES flight_services(id),
    departure_date DATE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (flight_service_id, departure_date)
);

CREATE TABLE seat_holds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    flight_instance_id UUID NOT NULL REFERENCES flight_instances(id),
    cabin TEXT NOT NULL CHECK (cabin IN ('economy', 'premium-economy', 'business', 'first')),
    adults SMALLINT NOT NULL CHECK (adults >= 1),
    children SMALLINT NOT NULL CHECK (children >= 0),
    infants SMALLINT NOT NULL CHECK (infants >= 0),
    access_token_hash BYTEA NOT NULL CHECK (octet_length(access_token_hash) = 32),
    expires_at TIMESTAMPTZ NOT NULL,
    released_at TIMESTAMPTZ,
    consumed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (released_at IS NULL OR consumed_at IS NULL)
);

CREATE TABLE flight_seats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    flight_instance_id UUID NOT NULL REFERENCES flight_instances(id) ON DELETE CASCADE,
    seat_number TEXT NOT NULL CHECK (seat_number ~ '^[1-9][0-9]*[A-Z]+$'),
    row_number SMALLINT NOT NULL CHECK (row_number > 0),
    column_code TEXT NOT NULL CHECK (column_code ~ '^[A-Z]+$'),
    cabin TEXT NOT NULL CHECK (cabin IN ('economy', 'premium-economy', 'business', 'first')),
    position TEXT NOT NULL CHECK (position IN ('window', 'middle', 'aisle')),
    sellable BOOLEAN NOT NULL DEFAULT TRUE,
    booking_status TEXT NOT NULL DEFAULT 'AVAILABLE' CHECK (booking_status IN ('AVAILABLE', 'BOOKED')),
    hold_id UUID REFERENCES seat_holds(id) ON DELETE SET NULL,
    booked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (flight_instance_id, cabin, seat_number),
    CHECK (booking_status <> 'BOOKED' OR (hold_id IS NULL AND booked_at IS NOT NULL)),
    CHECK (booking_status <> 'AVAILABLE' OR booked_at IS NULL)
);

CREATE INDEX idx_flight_instances_service_date
    ON flight_instances (flight_service_id, departure_date);
CREATE INDEX idx_flight_seats_inventory
    ON flight_seats (flight_instance_id, cabin, seat_number);
CREATE INDEX idx_flight_seats_hold_id
    ON flight_seats (hold_id) WHERE hold_id IS NOT NULL;
CREATE INDEX idx_seat_holds_active_expiry
    ON seat_holds (expires_at)
    WHERE released_at IS NULL AND consumed_at IS NULL;

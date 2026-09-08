CREATE TABLE airports (
    code TEXT PRIMARY KEY CHECK (code ~ '^[A-Z]{3}$'),
    name TEXT NOT NULL CHECK (length(btrim(name)) > 0),
    city TEXT NOT NULL CHECK (length(btrim(city)) > 0),
    country_code TEXT NOT NULL CHECK (country_code ~ '^[A-Z]{2}$'),
    time_zone TEXT NOT NULL CHECK (length(btrim(time_zone)) > 0)
);

INSERT INTO airports (code, name, city, country_code, time_zone) VALUES
    ('BKK', 'Suvarnabhumi Airport', 'Bangkok', 'TH', 'Asia/Bangkok'),
    ('DXB', 'Dubai International Airport', 'Dubai', 'AE', 'Asia/Dubai'),
    ('HND', 'Haneda Airport', 'Tokyo', 'JP', 'Asia/Tokyo'),
    ('JFK', 'John F. Kennedy International Airport', 'New York', 'US', 'America/New_York'),
    ('LHR', 'Heathrow Airport', 'London', 'GB', 'Europe/London')
ON CONFLICT (code) DO NOTHING;

ALTER TABLE flight_services
    ADD COLUMN status TEXT NOT NULL DEFAULT 'SCHEDULED' CHECK (status IN ('SCHEDULED', 'CANCELLED')),
    ADD COLUMN operating_date DATE,
    ADD COLUMN version BIGINT NOT NULL DEFAULT 1 CHECK (version > 0),
    ADD COLUMN cancelled_at TIMESTAMPTZ;

ALTER TABLE flight_services ADD CONSTRAINT flight_services_cancellation_complete
    CHECK ((status = 'CANCELLED') = (cancelled_at IS NOT NULL));

CREATE INDEX flight_services_management_list_idx
    ON flight_services (operating_date DESC NULLS LAST, flight_number);
CREATE INDEX flight_services_public_search_idx
    ON flight_services (origin_code, destination_code, operating_date, status)
    WHERE status = 'SCHEDULED';

-- Per-service templates isolate managed inventory from the shared legacy aircraft fixture.
CREATE TABLE flight_service_seat_templates (
    flight_service_id UUID NOT NULL REFERENCES flight_services(id) ON DELETE RESTRICT,
    seat_number TEXT NOT NULL CHECK (seat_number ~ '^[1-9][0-9]*[A-Z]+$'),
    row_number SMALLINT NOT NULL CHECK (row_number > 0),
    column_code TEXT NOT NULL CHECK (column_code ~ '^[A-Z]+$'),
    cabin TEXT NOT NULL CHECK (cabin IN ('business', 'first')),
    position TEXT NOT NULL CHECK (position IN ('window', 'middle', 'aisle')),
    sellable BOOLEAN NOT NULL DEFAULT TRUE,
    PRIMARY KEY (flight_service_id, cabin, seat_number)
);

CREATE TABLE flight_management_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_staff_user_id UUID NOT NULL REFERENCES staff_users(id) ON DELETE RESTRICT,
    flight_service_id UUID NOT NULL REFERENCES flight_services(id) ON DELETE RESTRICT,
    action TEXT NOT NULL CHECK (action IN ('FLIGHT_CREATED', 'FLIGHT_EDITED', 'FLIGHT_CANCELLED')),
    before_state JSONB,
    after_state JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX flight_management_audit_flight_idx
    ON flight_management_audit (flight_service_id, created_at DESC);

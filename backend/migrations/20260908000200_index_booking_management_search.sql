-- Branch 22 list ordering and normalized substring passenger lookup are central
-- staff workflows. These indexes preserve every historical row and avoid
-- relying on the browser-side result bound to constrain database work.
CREATE INDEX tickets_booking_management_order_idx
    ON tickets (created_at DESC, booking_reference);

CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE INDEX hold_passengers_booking_name_trgm_idx
    ON hold_passengers USING GIN (
        lower(given_name || ' ' || COALESCE(middle_name || ' ', '') || family_name)
        gin_trgm_ops
    );

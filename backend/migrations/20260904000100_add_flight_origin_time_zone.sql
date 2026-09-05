-- Scheduled service times are local airport times. Persist the IANA origin zone so
-- customer-action cutoffs can be converted to an authoritative instant, including DST.
ALTER TABLE flight_services
    ADD COLUMN origin_time_zone TEXT;

UPDATE flight_services
SET origin_time_zone = CASE origin_code
    WHEN 'BKK' THEN 'Asia/Bangkok'
    WHEN 'HND' THEN 'Asia/Tokyo'
    WHEN 'LHR' THEN 'Europe/London'
    WHEN 'JFK' THEN 'America/New_York'
    WHEN 'DXB' THEN 'Asia/Dubai'
END
WHERE departure_time IS NOT NULL;

-- Unknown/custom origins intentionally remain NULL. Manage Booking then reports
-- eligibility as unavailable instead of guessing a zone or blocking an upgrade.

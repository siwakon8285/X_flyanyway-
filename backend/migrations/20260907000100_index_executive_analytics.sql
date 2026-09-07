CREATE INDEX idx_payment_attempts_executive_analytics
    ON payment_attempts (provider, succeeded_at, seat_hold_id)
    INCLUDE (amount)
    WHERE status = 'SUCCEEDED' AND currency_code = 'THB';

CREATE INDEX idx_flight_instances_executive_departures
    ON flight_instances (departure_date, flight_service_id);

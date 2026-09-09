-- Preserve the existing booking-level E-ticket model while making the passenger-to-seat
-- relationship explicit. Infants do not consume a seat. Historical assignments follow the
-- same stable passenger-ordinal / physical-seat ordering used by the product summaries.
ALTER TABLE payment_attempt_seats ADD COLUMN passenger_ordinal SMALLINT;

WITH ranked_seats AS (
    SELECT finalized.payment_attempt_id, finalized.flight_seat_id,
           row_number() OVER (
               PARTITION BY finalized.payment_attempt_id
               ORDER BY seat.row_number, seat.column_code, seat.id
           ) AS seat_rank
    FROM payment_attempt_seats finalized
    JOIN flight_seats seat ON seat.id = finalized.flight_seat_id
), ranked_passengers AS (
    SELECT attempt.id AS payment_attempt_id, passenger.ordinal,
           row_number() OVER (
               PARTITION BY attempt.id ORDER BY passenger.ordinal
           ) AS passenger_rank
    FROM payment_attempts attempt
    JOIN hold_passengers passenger ON passenger.seat_hold_id = attempt.seat_hold_id
    WHERE passenger.passenger_type <> 'INFANT'
)
UPDATE payment_attempt_seats finalized
SET passenger_ordinal = passenger.ordinal
FROM ranked_seats seat, ranked_passengers passenger
WHERE seat.payment_attempt_id = finalized.payment_attempt_id
  AND seat.flight_seat_id = finalized.flight_seat_id
  AND passenger.payment_attempt_id = seat.payment_attempt_id
  AND passenger.passenger_rank = seat.seat_rank;

ALTER TABLE payment_attempt_seats
    ALTER COLUMN passenger_ordinal SET NOT NULL,
    ADD CONSTRAINT payment_attempt_seats_passenger_ordinal_positive
        CHECK (passenger_ordinal > 0),
    ADD CONSTRAINT payment_attempt_seats_attempt_passenger_unique
        UNIQUE (payment_attempt_id, passenger_ordinal);

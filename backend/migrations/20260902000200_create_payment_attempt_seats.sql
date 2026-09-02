-- Preserve the paid hold-to-seat association after flight_seats.hold_id is cleared by the
-- existing BOOKED inventory constraint. This is finalization metadata, not a Ticket.
CREATE TABLE payment_attempt_seats (
    payment_attempt_id UUID NOT NULL REFERENCES payment_attempts(id) ON DELETE CASCADE,
    flight_seat_id UUID NOT NULL REFERENCES flight_seats(id) ON DELETE RESTRICT,
    PRIMARY KEY (payment_attempt_id, flight_seat_id),
    UNIQUE (flight_seat_id)
);

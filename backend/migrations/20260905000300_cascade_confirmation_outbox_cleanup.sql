ALTER TABLE booking_confirmation_email_outbox
    DROP CONSTRAINT booking_confirmation_email_outbox_payment_attempt_id_fkey;

ALTER TABLE booking_confirmation_email_outbox
    ADD CONSTRAINT booking_confirmation_email_outbox_payment_attempt_id_fkey
    FOREIGN KEY (payment_attempt_id) REFERENCES payment_attempts(id) ON DELETE CASCADE;

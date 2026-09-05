-- New holds require an explicit booking contact. Rows that predate this
-- additive migration remain valid for historical compatibility and are not
-- assigned a guessed recipient.
ALTER TABLE seat_holds
    ADD COLUMN booking_contact_required BOOLEAN NOT NULL DEFAULT TRUE;

UPDATE seat_holds
SET booking_contact_required = FALSE
WHERE created_at < NOW();

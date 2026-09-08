ALTER TABLE flight_management_audit ADD COLUMN actor_email TEXT;

UPDATE flight_management_audit audit
SET actor_email=staff.email
FROM staff_users staff
WHERE staff.id=audit.actor_staff_user_id;

ALTER TABLE flight_management_audit
    ALTER COLUMN actor_email SET NOT NULL,
    ADD CONSTRAINT flight_management_audit_actor_email_present CHECK (length(btrim(actor_email)) > 0),
    DROP CONSTRAINT flight_management_audit_actor_staff_user_id_fkey;

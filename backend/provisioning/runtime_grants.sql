\set ON_ERROR_STOP on

SELECT current_database() = :'project_database' AS target_matches \gset
\if :target_matches
\else
  \echo 'Connected database does not match project_database; refusing runtime grants.'
  \quit 3
\endif

BEGIN;

REVOKE ALL ON SCHEMA public FROM x_fly_runtime;
GRANT USAGE ON SCHEMA public TO x_fly_runtime;
REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA public FROM x_fly_runtime;

-- Table-level and column-level INSERT privileges are cumulative. Remove the
-- historical broad audit grant before restoring only the approved columns.
REVOKE ALL PRIVILEGES ON TABLE public.api_client_management_audit
FROM x_fly_runtime, PUBLIC;
REVOKE INSERT ON TABLE public.api_client_management_audit
FROM x_fly_runtime, PUBLIC;
REVOKE ALL PRIVILEGES ON TABLE public.staff_security_audit
FROM x_fly_runtime, PUBLIC;

REVOKE ALL PRIVILEGES ON TABLE
    public.api_client_credentials,
    public.external_access_tokens
FROM x_fly_runtime, PUBLIC;

GRANT SELECT ON TABLE
    public._sqlx_migrations,
    public.aircraft_seat_templates,
    public.airports,
    public.api_client_allowed_scopes,
    public.api_client_management_audit,
    public.api_clients,
    public.api_scope_catalog,
    public.booking_cancellations,
    public.booking_confirmation_email_outbox,
    public.booking_contacts,
    public.booking_operations_audit,
    public.boarding_pass_operations_audit,
    public.boarding_passes,
    public.flight_instances,
    public.flight_management_audit,
    public.flight_seats,
    public.flight_service_cabins,
    public.flight_service_seat_templates,
    public.flight_services,
    public.hold_extras,
    public.hold_passengers,
    public.hold_review_pricing,
    public.payment_attempt_seats,
    public.payment_attempts,
    public.permissions,
    public.role_permissions,
    public.roles,
    public.seat_holds,
    public.staff_login_throttles,
    public.staff_sessions,
    public.staff_user_roles,
    public.staff_users,
    public.stripe_refund_events,
    public.stripe_webhook_events,
    public.supported_countries,
    public.tickets
TO x_fly_runtime;

GRANT INSERT, UPDATE ON TABLE
    public.api_clients,
    public.booking_cancellations,
    public.booking_contacts,
    public.flight_instances,
    public.flight_seats,
    public.flight_service_cabins,
    public.flight_services,
    public.hold_review_pricing,
    public.payment_attempt_seats,
    public.payment_attempts,
    public.seat_holds,
    public.staff_login_throttles,
    public.staff_sessions,
    public.tickets
TO x_fly_runtime;

GRANT INSERT ON TABLE
    public.api_client_allowed_scopes,
    public.booking_operations_audit,
    public.flight_management_audit,
    public.flight_service_seat_templates,
    public.hold_passengers,
    public.stripe_refund_events,
    public.stripe_webhook_events
TO x_fly_runtime;

GRANT UPDATE ON TABLE
    public.staff_users
TO x_fly_runtime;

GRANT DELETE ON TABLE
    public.api_client_allowed_scopes,
    public.flight_service_seat_templates,
    public.hold_passengers,
    public.hold_review_pricing,
    public.staff_login_throttles
TO x_fly_runtime;

GRANT INSERT (
    api_client_id,
    actor_staff_user_id,
    action,
    before_state,
    after_state,
    credential_id,
    created_at
)
ON TABLE public.api_client_management_audit
TO x_fly_runtime;

GRANT INSERT (
    ticket_id,
    passenger_ordinal,
    flight_instance_id,
    seat_snapshot,
    cabin_snapshot,
    checked_in_at,
    issued_at,
    issued_by_staff_user_id
)
ON TABLE public.boarding_passes
TO x_fly_runtime;

GRANT INSERT (
    boarding_pass_id,
    ticket_id,
    passenger_ordinal,
    actor_staff_user_id,
    action,
    before_state,
    after_state
)
ON TABLE public.boarding_pass_operations_audit
TO x_fly_runtime;

GRANT INSERT (
    action,
    actor_staff_user_id,
    session_id,
    permission_code,
    request_id
)
ON TABLE public.staff_security_audit
TO x_fly_runtime;

GRANT SELECT (
    id,
    api_client_id,
    secret_digest,
    digest_version,
    issued_at,
    revoked_at,
    revocation_reason
)
ON TABLE public.api_client_credentials
TO x_fly_runtime;

GRANT INSERT (
    api_client_id,
    secret_digest,
    digest_version,
    issued_at,
    issued_by_staff_user_id
)
ON TABLE public.api_client_credentials
TO x_fly_runtime;

GRANT UPDATE (
    revoked_at,
    revoked_by_staff_user_id,
    revocation_reason
)
ON TABLE public.api_client_credentials
TO x_fly_runtime;

GRANT SELECT (
    id,
    api_client_credential_id,
    token_hash,
    issued_at,
    expires_at,
    revoked_at
)
ON TABLE public.external_access_tokens
TO x_fly_runtime;

GRANT INSERT (
    api_client_credential_id,
    token_hash,
    issued_at,
    expires_at
)
ON TABLE public.external_access_tokens
TO x_fly_runtime;

GRANT UPDATE (revoked_at)
ON TABLE public.external_access_tokens
TO x_fly_runtime;

REVOKE ALL ON FUNCTION public.has_protected_stripe_card_finalization(uuid) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.has_protected_stripe_card_finalization(uuid)
TO x_fly_runtime;

COMMIT;

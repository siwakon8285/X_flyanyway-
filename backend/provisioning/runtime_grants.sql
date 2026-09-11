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
    public.api_client_management_audit,
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

REVOKE ALL ON FUNCTION public.has_protected_stripe_card_finalization(uuid) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.has_protected_stripe_card_finalization(uuid)
TO x_fly_runtime;

COMMIT;

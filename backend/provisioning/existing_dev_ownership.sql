\set ON_ERROR_STOP on

SELECT current_database() = :'project_database' AS target_matches \gset
\if :target_matches
\else
  \echo 'Connected database does not match project_database; refusing ownership transition.'
  \quit 3
\endif

BEGIN;
SET LOCAL lock_timeout = '5s';

ALTER TABLE public._sqlx_migrations OWNER TO x_fly_migrator;
ALTER TABLE public.aircraft_seat_templates OWNER TO x_fly_migrator;
ALTER TABLE public.airports OWNER TO x_fly_migrator;
ALTER TABLE public.api_client_allowed_scopes OWNER TO x_fly_migrator;
ALTER TABLE public.api_client_management_audit OWNER TO x_fly_migrator;
ALTER TABLE public.api_clients OWNER TO x_fly_migrator;
ALTER TABLE public.api_scope_catalog OWNER TO x_fly_migrator;
ALTER TABLE public.booking_cancellations OWNER TO x_fly_migrator;
ALTER TABLE public.booking_confirmation_email_outbox OWNER TO x_fly_migrator;
ALTER TABLE public.booking_contacts OWNER TO x_fly_migrator;
ALTER TABLE public.booking_operations_audit OWNER TO x_fly_migrator;
ALTER TABLE public.flight_instances OWNER TO x_fly_migrator;
ALTER TABLE public.flight_management_audit OWNER TO x_fly_migrator;
ALTER TABLE public.flight_seats OWNER TO x_fly_migrator;
ALTER TABLE public.flight_service_cabins OWNER TO x_fly_migrator;
ALTER TABLE public.flight_service_seat_templates OWNER TO x_fly_migrator;
ALTER TABLE public.flight_services OWNER TO x_fly_migrator;
ALTER TABLE public.hold_extras OWNER TO x_fly_migrator;
ALTER TABLE public.hold_passengers OWNER TO x_fly_migrator;
ALTER TABLE public.hold_review_pricing OWNER TO x_fly_migrator;
ALTER TABLE public.payment_attempt_seats OWNER TO x_fly_migrator;
ALTER TABLE public.payment_attempts OWNER TO x_fly_migrator;
ALTER TABLE public.permissions OWNER TO x_fly_migrator;
ALTER TABLE public.role_permissions OWNER TO x_fly_migrator;
ALTER TABLE public.roles OWNER TO x_fly_migrator;
ALTER TABLE public.seat_holds OWNER TO x_fly_migrator;
ALTER TABLE public.staff_login_throttles OWNER TO x_fly_migrator;
ALTER TABLE public.staff_sessions OWNER TO x_fly_migrator;
ALTER TABLE public.staff_user_roles OWNER TO x_fly_migrator;
ALTER TABLE public.staff_users OWNER TO x_fly_migrator;
ALTER TABLE public.stripe_refund_events OWNER TO x_fly_migrator;
ALTER TABLE public.stripe_webhook_events OWNER TO x_fly_migrator;
ALTER TABLE public.supported_countries OWNER TO x_fly_migrator;
ALTER TABLE public.tickets OWNER TO x_fly_migrator;
ALTER FUNCTION public.has_protected_stripe_card_finalization(uuid) OWNER TO x_fly_migrator;

COMMIT;

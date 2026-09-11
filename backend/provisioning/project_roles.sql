\set ON_ERROR_STOP on

SELECT current_database() = :'project_database' AS target_matches \gset
\if :target_matches
\else
  \echo 'Connected database does not match project_database; refusing provisioning.'
  \quit 3
\endif

BEGIN;

CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS pg_trgm;

DO $roles$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'x_fly_migrator') THEN
        CREATE ROLE x_fly_migrator LOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE
            NOREPLICATION NOBYPASSRLS;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'x_fly_runtime') THEN
        CREATE ROLE x_fly_runtime LOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE
            NOREPLICATION NOBYPASSRLS;
    END IF;
END
$roles$;

GRANT CONNECT ON DATABASE :"project_database" TO x_fly_migrator, x_fly_runtime;
GRANT USAGE, CREATE ON SCHEMA public TO x_fly_migrator;
GRANT USAGE ON SCHEMA public TO x_fly_runtime;
REVOKE CREATE ON SCHEMA public FROM x_fly_runtime;

COMMIT;

\echo 'Set both role passwords interactively with psql \password before using their URLs.'

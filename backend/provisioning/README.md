# PostgreSQL role provisioning

These scripts establish X-Fly's project roles without storing credentials:

- the infrastructure administrator creates roles and performs reviewed
  ownership/grant transitions;
- `x_fly_migrator` owns application objects and runs `db_admin migrate`;
- `x_fly_runtime` owns no application objects and runs the API with explicit
  DML privileges only; operator commands use `x_fly_migrator`.

All scripts require psql variable `project_database` to exactly match the
connected database. Run them from a trusted administrative terminal and set
the two role passwords interactively with psql `\password`; never put a real
password in these files or a command-line argument.

For a fresh database, the order is:

1. run `project_roles.sql` as the infrastructure administrator;
2. set both project-role passwords interactively;
3. configure `MIGRATION_DATABASE_URL` and run `db_admin migrate`;
4. run `runtime_grants.sql` as the infrastructure administrator;
5. optionally run `db_admin seed-demo` with an exact
   `DEMO_SEED_DATABASE` confirmation;
6. start the application with `DATABASE_URL` as `x_fly_runtime`.

For an existing X-Fly database, take and verify a recoverable backup first.
Inventory the actual objects and compare them byte-for-byte with the explicit
allowlist in `existing_dev_ownership.sql`. If anything differs, stop. Then run
`project_roles.sql`, set credentials interactively, run the bounded ownership
transition, and run `runtime_grants.sql`. The ownership script changes only
the listed X-Fly tables and function; it does not use `REASSIGN OWNED`, change
extension/database ownership, or modify any migration-ledger row.

The migration and runtime URLs may use local loopback ports or a private
Docker service name such as `postgres:5432`; no source-code change is needed.

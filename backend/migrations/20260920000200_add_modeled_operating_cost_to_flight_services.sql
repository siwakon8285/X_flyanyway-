ALTER TABLE flight_services
    ADD COLUMN modeled_operating_cost_amount BIGINT;

ALTER TABLE flight_services
    ADD CONSTRAINT flight_services_modeled_operating_cost_check
    CHECK (
        modeled_operating_cost_amount IS NULL
        OR modeled_operating_cost_amount BETWEEN 0 AND 100000000
    );

COMMENT ON COLUMN flight_services.modeled_operating_cost_amount IS
    'Approved/demo planning estimate in whole THB for one scheduled flight occurrence; not audited accounting cost.';

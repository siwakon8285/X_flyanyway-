INSERT INTO flight_services (
    public_id,
    flight_number,
    origin_code,
    destination_code,
    aircraft_code
)
VALUES
    ('xf-201', 'XF 201', 'BKK', 'LHR', 'Airbus A350-900'),
    ('xf-315', 'XF 315', 'BKK', 'LHR', 'Boeing 787-9'),
    ('xf-428', 'XF 428', 'BKK', 'LHR', 'Airbus A350-1000'),
    ('xf-512', 'XF 512', 'BKK', 'LHR', 'Boeing 787-9'),
    ('xf-621', 'XF 621', 'BKK', 'HND', 'Airbus A330-900'),
    ('xf-637', 'XF 637', 'BKK', 'HND', 'Boeing 787-9'),
    ('xf-649', 'XF 649', 'BKK', 'HND', 'Airbus A350-900'),
    ('xf-701', 'XF 701', 'BKK', 'DXB', 'Boeing 787-9'),
    ('xf-719', 'XF 719', 'BKK', 'DXB', 'Airbus A330-900'),
    ('xf-733', 'XF 733', 'BKK', 'DXB', 'Airbus A350-900'),
    ('xf-802', 'XF 802', 'HND', 'BKK', 'Boeing 787-9'),
    ('xf-816', 'XF 816', 'HND', 'BKK', 'Airbus A330-900'),
    ('xf-828', 'XF 828', 'HND', 'BKK', 'Airbus A350-900'),
    ('xf-202', 'XF 202', 'LHR', 'BKK', 'Airbus A350-900'),
    ('xf-316', 'XF 316', 'LHR', 'BKK', 'Boeing 787-9'),
    ('xf-430', 'XF 430', 'LHR', 'BKK', 'Airbus A350-900'),
    ('xf-901', 'XF 901', 'JFK', 'LHR', 'Boeing 787-9'),
    ('xf-915', 'XF 915', 'JFK', 'LHR', 'Airbus A330-900'),
    ('xf-927', 'XF 927', 'JFK', 'LHR', 'Airbus A350-900')
ON CONFLICT (public_id) DO UPDATE SET
    flight_number = EXCLUDED.flight_number,
    origin_code = EXCLUDED.origin_code,
    destination_code = EXCLUDED.destination_code,
    aircraft_code = EXCLUDED.aircraft_code,
    updated_at = NOW();

WITH cabin_codes(cabin) AS (
    VALUES ('economy'), ('premium-economy'), ('business'), ('first')
)
INSERT INTO flight_service_cabins (flight_service_id, cabin)
SELECT services.id, cabin_codes.cabin
FROM flight_services AS services
CROSS JOIN cabin_codes
WHERE NOT (
    services.public_id IN ('xf-315', 'xf-512')
    AND cabin_codes.cabin = 'first'
)
ON CONFLICT DO NOTHING;

WITH aircraft_cabins(aircraft_code, cabin, row_start, row_count) AS (
    VALUES
        ('Airbus A330-900', 'economy', 20, 6),
        ('Airbus A330-900', 'premium-economy', 12, 4),
        ('Airbus A330-900', 'business', 3, 4),
        ('Airbus A330-900', 'first', 1, 2),
        ('Airbus A350-900', 'economy', 20, 7),
        ('Airbus A350-900', 'premium-economy', 11, 5),
        ('Airbus A350-900', 'business', 3, 5),
        ('Airbus A350-900', 'first', 1, 3),
        ('Airbus A350-1000', 'economy', 22, 8),
        ('Airbus A350-1000', 'premium-economy', 12, 5),
        ('Airbus A350-1000', 'business', 3, 6),
        ('Airbus A350-1000', 'first', 1, 3),
        ('Boeing 787-9', 'economy', 18, 6),
        ('Boeing 787-9', 'premium-economy', 10, 4),
        ('Boeing 787-9', 'business', 3, 4),
        ('Boeing 787-9', 'first', 1, 2)
), cabin_columns(cabin, column_code, position) AS (
    VALUES
        ('economy', 'A', 'window'),
        ('economy', 'B', 'middle'),
        ('economy', 'C', 'aisle'),
        ('economy', 'D', 'aisle'),
        ('economy', 'E', 'middle'),
        ('economy', 'F', 'window'),
        ('premium-economy', 'A', 'window'),
        ('premium-economy', 'C', 'aisle'),
        ('premium-economy', 'D', 'aisle'),
        ('premium-economy', 'F', 'window'),
        ('business', 'A', 'window'),
        ('business', 'D', 'aisle'),
        ('business', 'G', 'aisle'),
        ('business', 'K', 'window'),
        ('first', 'A', 'window'),
        ('first', 'K', 'window')
)
INSERT INTO aircraft_seat_templates (
    aircraft_code,
    seat_number,
    row_number,
    column_code,
    cabin,
    position
)
SELECT
    aircraft_cabins.aircraft_code,
    generated.row_number::TEXT || cabin_columns.column_code,
    generated.row_number,
    cabin_columns.column_code,
    aircraft_cabins.cabin,
    cabin_columns.position
FROM aircraft_cabins
JOIN cabin_columns ON cabin_columns.cabin = aircraft_cabins.cabin
CROSS JOIN LATERAL generate_series(
    aircraft_cabins.row_start,
    aircraft_cabins.row_start + aircraft_cabins.row_count - 1
) AS generated(row_number)
ON CONFLICT (aircraft_code, cabin, seat_number) DO UPDATE SET
    position = EXCLUDED.position,
    sellable = EXCLUDED.sellable;

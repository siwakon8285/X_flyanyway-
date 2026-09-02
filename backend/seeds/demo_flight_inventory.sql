INSERT INTO flight_services (
    public_id,
    flight_number,
    origin_code,
    destination_code,
    aircraft_code,
    departure_time,
    arrival_time,
    arrival_day_offset,
    duration_minutes,
    stops
)
VALUES
    ('xf-201', 'XF 201', 'BKK', 'LHR', 'Airbus A350-900', '09:15', '16:40', 0, 805, 'DIRECT'),
    ('xf-315', 'XF 315', 'BKK', 'LHR', 'Boeing 787-9', '07:30', '15:40', 0, 850, 'DIRECT'),
    ('xf-428', 'XF 428', 'BKK', 'LHR', 'Airbus A350-1000', '12:40', '19:50', 0, 790, 'DIRECT'),
    ('xf-512', 'XF 512', 'BKK', 'LHR', 'Boeing 787-9', '23:40', '06:15', 1, 925, 'ONE_STOP'),
    ('xf-621', 'XF 621', 'BKK', 'HND', 'Airbus A330-900', '08:10', '16:20', 0, 370, 'DIRECT'),
    ('xf-637', 'XF 637', 'BKK', 'HND', 'Boeing 787-9', '14:35', '22:45', 0, 370, 'DIRECT'),
    ('xf-649', 'XF 649', 'BKK', 'HND', 'Airbus A350-900', '23:55', '08:05', 1, 370, 'DIRECT'),
    ('xf-701', 'XF 701', 'BKK', 'DXB', 'Boeing 787-9', '09:20', '13:05', 0, 405, 'DIRECT'),
    ('xf-719', 'XF 719', 'BKK', 'DXB', 'Airbus A330-900', '16:10', '19:55', 0, 405, 'DIRECT'),
    ('xf-733', 'XF 733', 'BKK', 'DXB', 'Airbus A350-900', '22:45', '02:30', 1, 405, 'DIRECT'),
    ('xf-802', 'XF 802', 'HND', 'BKK', 'Boeing 787-9', '09:00', '14:10', 0, 430, 'DIRECT'),
    ('xf-816', 'XF 816', 'HND', 'BKK', 'Airbus A330-900', '15:40', '20:45', 0, 425, 'DIRECT'),
    ('xf-828', 'XF 828', 'HND', 'BKK', 'Airbus A350-900', '23:20', '04:35', 1, 435, 'DIRECT'),
    ('xf-202', 'XF 202', 'LHR', 'BKK', 'Airbus A350-900', '10:30', '05:45', 1, 795, 'DIRECT'),
    ('xf-316', 'XF 316', 'LHR', 'BKK', 'Boeing 787-9', '18:20', '13:35', 1, 795, 'DIRECT'),
    ('xf-430', 'XF 430', 'LHR', 'BKK', 'Airbus A350-900', '22:05', '17:10', 1, 785, 'DIRECT'),
    ('xf-901', 'XF 901', 'JFK', 'LHR', 'Boeing 787-9', '08:30', '20:25', 0, 415, 'DIRECT'),
    ('xf-915', 'XF 915', 'JFK', 'LHR', 'Airbus A330-900', '18:40', '06:35', 1, 415, 'DIRECT'),
    ('xf-927', 'XF 927', 'JFK', 'LHR', 'Airbus A350-900', '22:15', '10:10', 1, 415, 'DIRECT')
ON CONFLICT (public_id) DO UPDATE SET
    flight_number = EXCLUDED.flight_number,
    origin_code = EXCLUDED.origin_code,
    destination_code = EXCLUDED.destination_code,
    aircraft_code = EXCLUDED.aircraft_code,
    departure_time = EXCLUDED.departure_time,
    arrival_time = EXCLUDED.arrival_time,
    arrival_day_offset = EXCLUDED.arrival_day_offset,
    duration_minutes = EXCLUDED.duration_minutes,
    stops = EXCLUDED.stops,
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

WITH fares(public_id, cabin, amount) AS (
    VALUES
        ('xf-201', 'economy', 21900), ('xf-201', 'premium-economy', 32900), ('xf-201', 'business', 68900), ('xf-201', 'first', 114900),
        ('xf-315', 'economy', 20500), ('xf-315', 'premium-economy', 29900), ('xf-315', 'business', 64900),
        ('xf-428', 'economy', 22900), ('xf-428', 'premium-economy', 34900), ('xf-428', 'business', 71500), ('xf-428', 'first', 119500),
        ('xf-512', 'economy', 21200), ('xf-512', 'premium-economy', 35900), ('xf-512', 'business', 67500),
        ('xf-621', 'economy', 14500), ('xf-621', 'premium-economy', 21900), ('xf-621', 'business', 39900), ('xf-621', 'first', 65900),
        ('xf-637', 'economy', 13700), ('xf-637', 'premium-economy', 20800), ('xf-637', 'business', 38200), ('xf-637', 'first', 63800),
        ('xf-649', 'economy', 15100), ('xf-649', 'premium-economy', 22600), ('xf-649', 'business', 41500), ('xf-649', 'first', 67900),
        ('xf-701', 'economy', 16900), ('xf-701', 'premium-economy', 25900), ('xf-701', 'business', 46900), ('xf-701', 'first', 78900),
        ('xf-719', 'economy', 15800), ('xf-719', 'premium-economy', 24400), ('xf-719', 'business', 45200), ('xf-719', 'first', 76500),
        ('xf-733', 'economy', 17600), ('xf-733', 'premium-economy', 26800), ('xf-733', 'business', 48800), ('xf-733', 'first', 81200),
        ('xf-802', 'economy', 14900), ('xf-802', 'premium-economy', 22400), ('xf-802', 'business', 40500), ('xf-802', 'first', 66800),
        ('xf-816', 'economy', 14100), ('xf-816', 'premium-economy', 21300), ('xf-816', 'business', 38900), ('xf-816', 'first', 64900),
        ('xf-828', 'economy', 15600), ('xf-828', 'premium-economy', 23100), ('xf-828', 'business', 42100), ('xf-828', 'first', 69100),
        ('xf-202', 'economy', 24500), ('xf-202', 'premium-economy', 36900), ('xf-202', 'business', 72900), ('xf-202', 'first', 119900),
        ('xf-316', 'economy', 22900), ('xf-316', 'premium-economy', 34800), ('xf-316', 'business', 69800), ('xf-316', 'first', 115500),
        ('xf-430', 'economy', 25800), ('xf-430', 'premium-economy', 38400), ('xf-430', 'business', 74800), ('xf-430', 'first', 122500),
        ('xf-901', 'economy', 28500), ('xf-901', 'premium-economy', 42500), ('xf-901', 'business', 82900), ('xf-901', 'first', 135000),
        ('xf-915', 'economy', 26900), ('xf-915', 'premium-economy', 40700), ('xf-915', 'business', 79800), ('xf-915', 'first', 131500),
        ('xf-927', 'economy', 29800), ('xf-927', 'premium-economy', 44800), ('xf-927', 'business', 85600), ('xf-927', 'first', 139500)
)
UPDATE flight_service_cabins AS cabins
SET base_fare_amount = fares.amount,
    currency_code = 'THB'
FROM fares
JOIN flight_services AS services ON services.public_id = fares.public_id
WHERE cabins.flight_service_id = services.id
  AND cabins.cabin = fares.cabin;

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

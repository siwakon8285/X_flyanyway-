-- Existing services keep their historical cabin rows. Branch 21 only copies the
-- currently sellable Business/First geometry into the isolated per-service template.
INSERT INTO flight_service_seat_templates (
    flight_service_id, seat_number, row_number, column_code, cabin, position, sellable
)
SELECT service.id, template.seat_number, template.row_number, template.column_code,
       template.cabin, template.position, template.sellable
FROM flight_services service
JOIN aircraft_seat_templates template ON template.aircraft_code=service.aircraft_code
WHERE template.cabin IN ('business','first')
  AND NOT EXISTS (
      SELECT 1 FROM flight_service_seat_templates existing
      WHERE existing.flight_service_id=service.id
  )
ON CONFLICT (flight_service_id,cabin,seat_number) DO NOTHING;

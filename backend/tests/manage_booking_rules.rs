use chrono::{Duration, TimeZone, Utc};
use uuid::Uuid;

use x_fly_api::{
    domain::{
        manage_booking::{
            derive_travel_eligibility, CancellationEligibility, ManageBookingLookup,
            ManageBookingLookupError,
        },
        ticket::TicketStatus,
    },
    infrastructure::manage_booking::access::{sign, verify, ManageBookingAccessError},
};

const SECRET: &str = "manage-booking-test-secret-that-is-at-least-32-characters";

#[test]
fn lookup_normalizes_only_harmless_presentation_differences() {
    let lookup =
        ManageBookingLookup::new("  xfabcdefgh  ".to_owned(), "  Van   der Meer ".to_owned())
            .unwrap();

    assert_eq!(lookup.booking_reference(), "XFABCDEFGH");
    assert_eq!(lookup.family_name(), "van der meer");
}

#[test]
fn lookup_rejects_noncanonical_reference_and_invalid_identity_fields() {
    assert_eq!(
        ManageBookingLookup::new("XF4B7K".to_owned(), "Suri".to_owned()),
        Err(ManageBookingLookupError::InvalidBookingReference)
    );
    assert_eq!(
        ManageBookingLookup::new("XFABCDEFGH".to_owned(), "Suri%".to_owned()),
        Err(ManageBookingLookupError::InvalidFamilyName)
    );
}

#[test]
fn eligibility_uses_the_exact_twenty_four_hour_boundary() {
    let now = Utc.with_ymd_and_hms(2026, 9, 4, 4, 0, 0).unwrap();

    let at_cutoff =
        derive_travel_eligibility(Some(now + Duration::hours(24)), TicketStatus::Issued, now);
    assert_eq!(at_cutoff.cancellation, CancellationEligibility::Eligible);

    let inside_cutoff = derive_travel_eligibility(
        Some(now + Duration::hours(24) - Duration::seconds(1)),
        TicketStatus::Issued,
        now,
    );
    assert_eq!(
        inside_cutoff.cancellation,
        CancellationEligibility::Unavailable
    );

    let before_window =
        derive_travel_eligibility(Some(now + Duration::hours(25)), TicketStatus::Issued, now);
    assert_eq!(
        before_window.cancellation,
        CancellationEligibility::Eligible
    );
}

#[test]
fn cancelled_or_departed_travel_is_unavailable_for_customer_actions() {
    let now = Utc.with_ymd_and_hms(2026, 9, 4, 4, 0, 0).unwrap();
    for (departure, status) in [
        (now + Duration::days(2), TicketStatus::Cancelled),
        (now - Duration::seconds(1), TicketStatus::Issued),
    ] {
        let eligibility = derive_travel_eligibility(Some(departure), status, now);
        assert_eq!(
            eligibility.cancellation,
            CancellationEligibility::Unavailable
        );
    }
}

#[test]
fn signed_access_is_scoped_expires_and_rejects_tampering() {
    let ticket_id = Uuid::parse_str("018f6f52-2247-7b29-8d22-111111111111").unwrap();
    let expires_at = Utc.with_ymd_and_hms(2026, 9, 4, 4, 30, 0).unwrap();
    let token = sign(ticket_id, expires_at, [7; 16], SECRET).unwrap();

    assert_eq!(
        verify(
            &token,
            SECRET,
            Utc.with_ymd_and_hms(2026, 9, 4, 4, 29, 59).unwrap()
        )
        .unwrap(),
        ticket_id
    );
    assert_eq!(
        verify(&format!("{token}x"), SECRET, expires_at),
        Err(ManageBookingAccessError::Invalid)
    );
    assert_eq!(
        verify(&token, SECRET, expires_at),
        Err(ManageBookingAccessError::Expired)
    );
}

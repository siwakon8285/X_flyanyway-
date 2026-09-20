use chrono::{Duration, Utc};
use x_fly_api::domain::{
    boarding_pass::{derive_check_in_state, CheckInState, CheckInUnavailableReason},
    flight::FlightStatus,
    ticket::TicketStatus,
};

#[test]
fn check_in_opens_strictly_after_the_cancellation_cutoff_and_closes_at_departure() {
    let departure = Utc::now() + Duration::hours(48);

    assert_eq!(
        derive_check_in_state(
            departure - Duration::hours(24),
            Some(departure),
            TicketStatus::Issued,
            FlightStatus::Scheduled,
            true,
            true,
            false,
        ),
        CheckInState::Unavailable(CheckInUnavailableReason::TooEarly)
    );
    assert_eq!(
        derive_check_in_state(
            departure - Duration::hours(24) + Duration::seconds(1),
            Some(departure),
            TicketStatus::Issued,
            FlightStatus::Scheduled,
            true,
            true,
            false,
        ),
        CheckInState::Ready
    );
    assert_eq!(
        derive_check_in_state(
            departure,
            Some(departure),
            TicketStatus::Issued,
            FlightStatus::Scheduled,
            true,
            true,
            false,
        ),
        CheckInState::Unavailable(CheckInUnavailableReason::AlreadyDeparted)
    );
}

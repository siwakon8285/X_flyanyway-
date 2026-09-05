use chrono::{NaiveDate, NaiveTime};

use x_fly_api::domain::booking_confirmation::{
    BookingConfirmationEmail, BookingConfirmationLocale, DeliveryFailure, RetryDisposition,
};

fn email(locale: BookingConfirmationLocale) -> BookingConfirmationEmail {
    BookingConfirmationEmail {
        recipient: "contact@example.test".to_owned(),
        locale,
        flight_number: "XF 701".to_owned(),
        origin_code: "BKK".to_owned(),
        destination_code: "DXB".to_owned(),
        departure_date: NaiveDate::from_ymd_opt(2026, 10, 15).unwrap(),
        departure_time: NaiveTime::from_hms_opt(9, 15, 0),
        departure_time_zone: Some("Asia/Bangkok".to_owned()),
        cabin: "business".to_owned(),
        booking_reference: "XFABCDEFGH".to_owned(),
        ticket_number: Some("XFTABCDEFGHJKLM".to_owned()),
        manage_booking_url: "https://x-fly.example.test/manage-booking".to_owned(),
    }
}

#[test]
fn english_confirmation_renders_safe_html_and_plain_text_with_clean_manage_booking_url() {
    let rendered = email(BookingConfirmationLocale::En).render();

    assert_eq!(rendered.subject, "X-Fly Anyway — Booking Confirmed");
    assert!(rendered.html.contains("Booking Confirmed"));
    assert!(rendered.text.contains("Booking Reference: XFABCDEFGH"));
    assert!(rendered
        .html
        .contains("https://x-fly.example.test/manage-booking"));
    assert!(!rendered.html.contains("payment_intent"));
    assert!(!rendered.text.contains("?"));
}

#[test]
fn thai_confirmation_escapes_interpolated_values() {
    let mut value = email(BookingConfirmationLocale::Th);
    value.flight_number = "XF <701> & \"special\"".to_owned();
    let rendered = value.render();

    assert!(rendered.subject.contains("ยืนยันการจอง"));
    assert!(rendered
        .html
        .contains("XF &lt;701&gt; &amp; &quot;special&quot;"));
    assert!(!rendered.html.contains("XF <701>"));
    assert!(rendered.text.contains("XF <701> & \"special\""));
}

#[test]
fn thai_confirmation_localizes_departure_and_cabin() {
    let rendered = email(BookingConfirmationLocale::Th).render();

    assert!(rendered.text.contains("15 ตุลาคม 2026 เวลา 09:15"));
    assert!(rendered.text.contains("ชั้นโดยสาร: ชั้นธุรกิจ"));
    assert!(!rendered.text.contains("business"));

    for (raw, expected) in [
        ("economy", "ชั้นประหยัด"),
        ("premium-economy", "ชั้นประหยัดพรีเมียม"),
        ("business", "ชั้นธุรกิจ"),
        ("first", "ชั้นหนึ่ง"),
    ] {
        let mut value = email(BookingConfirmationLocale::Th);
        value.cabin = raw.to_owned();
        assert!(value
            .render()
            .text
            .contains(&format!("ชั้นโดยสาร: {expected}")));
    }
}

#[test]
fn english_confirmation_uses_customer_facing_cabin_names() {
    for (raw, expected) in [
        ("economy", "Economy"),
        ("premium-economy", "Premium Economy"),
        ("business", "Business"),
        ("first", "First"),
    ] {
        let mut value = email(BookingConfirmationLocale::En);
        value.cabin = raw.to_owned();
        assert!(value.render().text.contains(&format!("Cabin: {expected}")));
    }
}

#[test]
fn retry_disposition_is_bounded_and_classifies_transient_and_permanent_failures() {
    assert_eq!(
        DeliveryFailure::Timeout.retry_disposition(1),
        RetryDisposition::RetryAfter(std::time::Duration::from_secs(60))
    );
    assert_eq!(
        DeliveryFailure::ProviderStatus(500).retry_disposition(5),
        RetryDisposition::RetryAfter(std::time::Duration::from_secs(21_600))
    );
    assert_eq!(
        DeliveryFailure::ProviderStatus(401).retry_disposition(1),
        RetryDisposition::Permanent
    );
    assert_eq!(
        DeliveryFailure::Timeout.retry_disposition(6),
        RetryDisposition::Permanent
    );
}

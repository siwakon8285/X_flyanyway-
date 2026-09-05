use std::time::Duration;

use chrono::{Datelike, NaiveDate, NaiveTime};
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BookingConfirmationLocale {
    En,
    Th,
}

impl BookingConfirmationLocale {
    pub fn parse_database(value: &str) -> Option<Self> {
        match value {
            "EN" => Some(Self::En),
            "TH" => Some(Self::Th),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::En => "EN",
            Self::Th => "TH",
        }
    }
}

#[derive(Clone, Debug)]
pub struct BookingConfirmationEmail {
    pub recipient: String,
    pub locale: BookingConfirmationLocale,
    pub flight_number: String,
    pub origin_code: String,
    pub destination_code: String,
    pub departure_date: NaiveDate,
    pub departure_time: Option<NaiveTime>,
    pub departure_time_zone: Option<String>,
    pub cabin: String,
    pub booking_reference: String,
    pub ticket_number: Option<String>,
    pub manage_booking_url: String,
}

pub struct RenderedBookingConfirmationEmail {
    pub subject: String,
    pub html: String,
    pub text: String,
}

impl BookingConfirmationEmail {
    pub fn render(&self) -> RenderedBookingConfirmationEmail {
        let (
            subject,
            heading,
            intro,
            flight_label,
            route_label,
            departure_label,
            cabin_label,
            reference_label,
            ticket_label,
            cta,
        ) = match self.locale {
            BookingConfirmationLocale::En => (
                "X-Fly Anyway — Booking Confirmed",
                "Booking Confirmed",
                "Your booking has been successfully confirmed.",
                "Flight",
                "Journey",
                "Departure",
                "Cabin",
                "Booking Reference",
                "Ticket Number",
                "MANAGE BOOKING",
            ),
            BookingConfirmationLocale::Th => (
                "X-Fly Anyway — ยืนยันการจองแล้ว",
                "ยืนยันการจองแล้ว",
                "การจองของคุณได้รับการยืนยันเรียบร้อยแล้ว",
                "เที่ยวบิน",
                "เส้นทาง",
                "ออกเดินทาง",
                "ชั้นโดยสาร",
                "รหัสการจอง",
                "หมายเลขบัตรโดยสาร",
                "จัดการการจอง",
            ),
        };
        let departure = self.departure();
        let cabin = self.localized_cabin();
        let ticket_line = self
            .ticket_number
            .as_ref()
            .map(|ticket| format!("{ticket_label}: {ticket}"));
        let text = [
            "X-FLY ANYWAY".to_owned(),
            heading.to_owned(),
            intro.to_owned(),
            format!("{flight_label}: {}", self.flight_number),
            format!(
                "{route_label}: {} → {}",
                self.origin_code, self.destination_code
            ),
            format!("{departure_label}: {departure}"),
            format!("{cabin_label}: {cabin}"),
            format!("{reference_label}: {}", self.booking_reference),
            ticket_line.unwrap_or_default(),
            String::new(),
            format!("{cta}: {}", self.manage_booking_url),
        ]
        .into_iter()
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
        let ticket_html = self.ticket_number.as_ref().map(|ticket| format!("<tr><td style=\"padding:8px 0;color:#6b655a\">{}</td><td style=\"padding:8px 0;font-weight:700;text-align:right\">{}</td></tr>", escape(ticket_label), escape(ticket))).unwrap_or_default();
        let html = format!(
            "<!doctype html><html lang=\"{}\"><body style=\"margin:0;background:#f8f4ec;color:#15130f;font-family:Arial,sans-serif\"><table role=\"presentation\" width=\"100%\" cellspacing=\"0\" cellpadding=\"0\"><tr><td style=\"padding:32px 16px\"><table role=\"presentation\" width=\"100%\" cellspacing=\"0\" cellpadding=\"0\" style=\"max-width:600px;margin:auto;background:#fffdf8;border:1px solid #e8dfd0\"><tr><td style=\"height:5px;background:#ffd400\"></td></tr><tr><td style=\"padding:36px\"><p style=\"margin:0;color:#8a7400;font-size:12px;font-weight:700;letter-spacing:2px\">X-FLY ANYWAY</p><h1 style=\"margin:16px 0 8px;font-size:28px\">{}</h1><p style=\"margin:0 0 24px;color:#5f594f\">{}</p><table role=\"presentation\" width=\"100%\" cellspacing=\"0\" cellpadding=\"0\"><tr><td style=\"padding:8px 0;color:#6b655a\">{}</td><td style=\"padding:8px 0;font-weight:700;text-align:right\">{}</td></tr><tr><td style=\"padding:8px 0;color:#6b655a\">{}</td><td style=\"padding:8px 0;font-weight:700;text-align:right\">{} → {}</td></tr><tr><td style=\"padding:8px 0;color:#6b655a\">{}</td><td style=\"padding:8px 0;font-weight:700;text-align:right\">{}</td></tr><tr><td style=\"padding:8px 0;color:#6b655a\">{}</td><td style=\"padding:8px 0;font-weight:700;text-align:right\">{}</td></tr><tr><td style=\"padding:8px 0;color:#6b655a\">{}</td><td style=\"padding:8px 0;font-weight:700;text-align:right\">{}</td></tr>{}</table><p style=\"margin:32px 0 0\"><a href=\"{}\" style=\"display:inline-block;background:#15130f;color:#ffd400;padding:14px 20px;text-decoration:none;font-size:12px;font-weight:700;letter-spacing:1px\">{}</a></p></td></tr></table></td></tr></table></body></html>",
            self.locale.as_str().to_lowercase(), escape(heading), escape(intro), escape(flight_label), escape(&self.flight_number), escape(route_label), escape(&self.origin_code), escape(&self.destination_code), escape(departure_label), escape(&departure), escape(cabin_label), escape(cabin), escape(reference_label), escape(&self.booking_reference), ticket_html, escape(&self.manage_booking_url), escape(cta)
        );
        RenderedBookingConfirmationEmail {
            subject: subject.to_owned(),
            html,
            text,
        }
    }

    fn departure(&self) -> String {
        let date = match self.locale {
            BookingConfirmationLocale::En => self.departure_date.format("%-d %B %Y").to_string(),
            BookingConfirmationLocale::Th => {
                const MONTHS: [&str; 12] = [
                    "มกราคม",
                    "กุมภาพันธ์",
                    "มีนาคม",
                    "เมษายน",
                    "พฤษภาคม",
                    "มิถุนายน",
                    "กรกฎาคม",
                    "สิงหาคม",
                    "กันยายน",
                    "ตุลาคม",
                    "พฤศจิกายน",
                    "ธันวาคม",
                ];
                format!(
                    "{} {} {}",
                    self.departure_date.day(),
                    MONTHS[self.departure_date.month0() as usize],
                    self.departure_date.year()
                )
            }
        };
        let time = self
            .departure_time
            .map(|value| value.format("%H:%M").to_string())
            .unwrap_or_else(|| "TBC".to_owned());
        let date_time = match self.locale {
            BookingConfirmationLocale::En => format!("{date}, {time}"),
            BookingConfirmationLocale::Th => format!("{date} เวลา {time}"),
        };
        match &self.departure_time_zone {
            Some(zone) => format!("{date_time} ({zone})"),
            None => date_time,
        }
    }

    fn localized_cabin(&self) -> &str {
        match (self.locale, self.cabin.as_str()) {
            (BookingConfirmationLocale::En, "economy") => "Economy",
            (BookingConfirmationLocale::En, "premium-economy") => "Premium Economy",
            (BookingConfirmationLocale::En, "business") => "Business",
            (BookingConfirmationLocale::En, "first") => "First",
            (BookingConfirmationLocale::Th, "economy") => "ชั้นประหยัด",
            (BookingConfirmationLocale::Th, "premium-economy") => "ชั้นประหยัดพรีเมียม",
            (BookingConfirmationLocale::Th, "business") => "ชั้นธุรกิจ",
            (BookingConfirmationLocale::Th, "first") => "ชั้นหนึ่ง",
            _ => self.cabin.as_str(),
        }
    }
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryFailure {
    Timeout,
    Connectivity,
    ProviderStatus(u16),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetryDisposition {
    RetryAfter(Duration),
    Permanent,
}

impl DeliveryFailure {
    pub fn retry_disposition(self, attempt_count: u8) -> RetryDisposition {
        if attempt_count >= 6 {
            return RetryDisposition::Permanent;
        }
        match self {
            Self::Timeout | Self::Connectivity | Self::ProviderStatus(429 | 500..=599) => {
                let seconds =
                    [60, 300, 1_800, 7_200, 21_600][usize::from(attempt_count.saturating_sub(1))];
                RetryDisposition::RetryAfter(Duration::from_secs(seconds))
            }
            Self::ProviderStatus(_) => RetryDisposition::Permanent,
        }
    }
}

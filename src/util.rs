use chrono::{Datelike, NaiveDate, Utc};
use rand::{distributions::Alphanumeric, thread_rng, Rng};

use crate::data_structs::Months;

/// Returns the first day of a month passed as an unsigned integer as a `NaiveDate`.
/// Example: `get_first_day_from_month(1)` returns 2023-01-01.
pub fn get_first_day_from_month(month: u32) -> Option<NaiveDate> {
    return Utc::now()
        .with_day(1)
        .and_then(|date| return date.naive_utc().date().with_month(month));
}

/// If a month is passed, as a Some(Months), returns the first day of that month as a `NaiveDate`.
/// If None is passed, returns None.
/// Months is an enum with the months of the year.
pub fn get_first_day_from_month_or_none(
    option_month: Option<Months>,
) -> Result<Option<NaiveDate>, &'static str> {
    let Some(month) = option_month else {
        return Ok(None);
    };
    let month_n: u32 = month
        .try_into()
        .map_err(|_ignore| return "Could not convert month to u32.")?;
    match get_first_day_from_month(month_n) {
        Some(date) => return Ok(Some(date)),
        None => return Err("Could not get first day from month."),
    }
}

/// Returns the last day of a month passed as an unsigned integer as a `NaiveDate`.
/// Example: `get_last_day_from_month(1)` returns 2023-01-31.
pub fn get_last_day_from_month(month: u32) -> Option<NaiveDate> {
    let first_day_of_month = get_first_day_from_month(month)?;

    if first_day_of_month.month() == 12 {
        return first_day_of_month.with_day(31);
    }
    return first_day_of_month
        .with_month(month.checked_add(1)?)
        .and_then(|date| return date.checked_sub_days(chrono::Days::new(1)));
}

/// If a month is passed, as a Some(Months), returns the last day of that month as a `NaiveDate`.
/// If None is passed, returns None.
/// Months is an enum with the months of the year.
pub fn get_last_day_from_month_or_none(
    option_month: Option<Months>,
) -> Result<Option<NaiveDate>, &'static str> {
    let Some(month) = option_month else {
        return Ok(None);
    };
    let month_n: u32 = month
        .try_into()
        .map_err(|_ignore| return "Could not convert month to u32.")?;
    match get_last_day_from_month(month_n) {
        Some(date) => return Ok(Some(date)),
        None => return Err("Could not get last day from month, maybe it underflowed?"),
    }
}

/// Generates a random string of 128 characters to be used as an email verification token.
pub fn generate_verification_token() -> String {
    return thread_rng()
        .sample_iter(&Alphanumeric)
        .take(128)
        .map(char::from)
        .collect();
}

/// Generates a random string of 20 characters to be used as a one time password seed.
pub fn generate_otp_token() -> String {
    return thread_rng()
        .sample_iter(&Alphanumeric)
        .take(20) // recommended length of secret is 160 bits: https://www.rfc-editor.org/rfc/rfc4226#section-4
        .map(char::from)
        .collect();
}

/// Returns the current time in UTC plus 24 hours.
/// This is used to set the expiration time of the email verification token.
pub fn now_plus_24_hours() -> Option<chrono::DateTime<chrono::Utc>> {
    return chrono::Utc::now().checked_add_signed(chrono::Duration::hours(24));
}

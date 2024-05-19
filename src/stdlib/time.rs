//! Optional module to perform date and time operations using [`Value::Number`].
//!
//! While SLAC does not have a dedicated [`Value`] type for timestamps, this module
//! enables date, time and datetime manipulation on [`Value::Number`] floating
//! point values.
//!
//! All functions operate on a local timezone. (eg: 12:00 Noon = 0.5)
//! The RFC-functions do automatic conversion from the provided offset into the
//! systems local timezone.
//!
//! The integral part of the [`Value::Number`] float is the number of **days**,
//! which have passed since `midnight, January 1, 1970` (aka. the UNIX timestamp).
//! The fractional part is the `time of day` as a **fraction of 24 hours**
//! (e.g: 0.25 = 06:00 h, 0.75 = 18:00 h).
//!
//! This way of representing time is similar to Delphi [`TDateTime`](https://docwiki.embarcadero.com/Libraries/Alexandria/en/System.TDateTime).
//! Note: In contrast to UNIX Time, Delphi `TDateTime` is 25569 Days ahead,
//! starting it's count on December 30, 1899.
//!
//! Since the integral is the number of days, calculating date offsets is just a
//! matter of adding integer Days.
//!
//! ```slac
//! encode_date(2023, 12, 24) + 7 = new_years_eve
//! ```
//!
//! Likewise the calculation of a timespan is a simple fraction of a day.
//!
//! ```slac
//! encode_time(12, 30, 00) + (1 / 24) <= now // valid from 13:30:00
//! ```
//!
//! # Chrono
//!
//! This module uses the [`chrono`] crate and can be included using
//! the `chrono` feature.
use chrono::{
    DateTime, Datelike, FixedOffset, Local, Months, NaiveDate, NaiveDateTime, NaiveTime, TimeZone,
    Timelike,
};

use crate::{
    environment::{Arity, Function},
    Value,
};

use super::{
    default_number, default_string,
    error::{NativeError, NativeResult},
};

/// Returns all time functions as a fixed size array.
#[rustfmt::skip]
pub fn functions() -> Vec<Function> {
    vec![
        Function::new(super::math::trunc, Arity::required(1), "date(datetime: Number): Number"),
        Function::new(super::math::frac, Arity::required(1), "time(datetime: Number): Number"),
        Function::new(date_to_string, Arity::required(2), "date_to_string(fmt: String, datetime: Number): String"),
        Function::new(date_to_string, Arity::required(2), "time_to_string(fmt: String, datetime: Number): String"),
        Function::new(string_to_date, Arity::optional(1, 1), "string_to_date(date: String, format: String = '%Y-%m-%d'): Number"),
        Function::new(string_to_time, Arity::optional(1, 1), "string_to_time(time: String, format: String = '%H:%M:%S'): Number"),
        Function::new(string_to_datetime, Arity::optional(1, 1), "string_to_datetime(datetime: String, format: String = '%Y-%m-%d %H:%M:%S'): Number"),
        Function::new(date_from_rfc2822, Arity::required(1), "date_from_rfc2822(datetime: String): Number"),
        Function::new(date_from_rfc3339, Arity::required(1), "date_from_rfc3339(datetime: String): Number"),
        Function::new(date_to_rfc2822, Arity::required(1), "date_to_rfc2822(datetime: Number): String"),
        Function::new(date_to_rfc3339, Arity::required(1), "date_to_rfc3339(datetime: Number): String"),
        Function::new(day_of_week, Arity::required(1), "day_of_week(datetime: Number): Number"),
        Function::new(encode_date, Arity::required(3), "encode_date(year: Number, month: Number, day: Number): Number"),
        Function::new(encode_time, Arity::optional(3, 1), "encode_time(hour: Number, minute: Number, second: Number, millisecond: Number = 0): Number"),
        Function::new(inc_month, Arity::optional(1, 1), "inc_month(datetime: Number, increment: Number = 1): Number"),
        Function::new(is_leap_year, Arity::required(1), "is_leap_year(datetime: Number): Number"),
        Function::new(year, Arity::required(1), "year(datetime: Number): Number"),
        Function::new(month, Arity::required(1), "month(datetime: Number): Number"),
        Function::new(day, Arity::required(1), "day(datetime: Number): Number"),
        Function::new(hour, Arity::required(1), "hour(datetime: Number): Number"),
        Function::new(minute, Arity::required(1), "minute(datetime: Number): Number"),
        Function::new(second, Arity::required(1), "second(datetime: Number): Number"),
        Function::new(millisecond, Arity::required(1), "millisecond(datetime: Number): Number"),
    ]
}

const MILLISECONDS_PER_DAY: f64 = 24. * 60. * 60. * 1000.;

impl TryFrom<&Value> for NaiveDateTime {
    type Error = NativeError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(value) => {
                let milliseconds = (value * MILLISECONDS_PER_DAY) as i64;

                DateTime::from_timestamp_millis(milliseconds)
                    .map(|dt| dt.naive_utc())
                    .ok_or(NativeError::from("datetime out of range"))
            }
            _ => Err(NativeError::WrongParameterType),
        }
    }
}

impl From<NaiveDateTime> for Value {
    fn from(val: NaiveDateTime) -> Self {
        let milliseconds = val.and_utc().timestamp_millis();

        Value::Number(milliseconds as f64 / MILLISECONDS_PER_DAY)
    }
}

/// Formats a datetime [`Value`] with the specified format string.
/// See [`chrono::format::strftime`] for info on the syntax.
///
/// * Declaration: `date_to_string(fmt: String, datetime: Number): String`
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn date_to_string(params: &[Value]) -> NativeResult {
    match params {
        [Value::String(fmt), value] => {
            let datetime = NaiveDateTime::try_from(value)?;

            Ok(Value::String(datetime.format(fmt).to_string()))
        }
        [_, _] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(2)),
    }
}

/// Parses a date string with the specified format string and returns a [`Value::Number`].
/// See [`chrono::format::strftime`] for info on the syntax.
///
/// * Declaration: `string_to_date(date: String, format: String = '%Y-%m-%d')`
///
/// # Errors
///
/// Will return [`NativeError::CustomError`] if the String can not be parsed.
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn string_to_date(params: &[Value]) -> NativeResult {
    let fmt = default_string(params, 1, "%Y-%m-%d")?;

    match params {
        [Value::String(s), ..] => {
            let datetime = NaiveDate::parse_from_str(s, fmt)
                .map_err(|e| NativeError::from(e.to_string()))?
                .and_time(NaiveTime::default());

            Ok(Value::from(datetime))
        }
        [_, ..] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Parses a time string with the specified format string and returns a [`Value::Number`].
/// See [`chrono::format::strftime`] for info on the syntax.
///
/// * Declaration: `string_to_time(time: String, format: String = '%H:%M:%S'): Number`
///
/// # Errors
///
/// Will return [`NativeError::CustomError`] if the String can not be parsed.
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
#[allow(clippy::module_name_repetitions)]
pub fn string_to_time(params: &[Value]) -> NativeResult {
    let fmt = default_string(params, 1, "%H:%M:%S")?;

    match params {
        [Value::String(s), ..] => {
            let time =
                NaiveTime::parse_from_str(s, fmt).map_err(|e| NativeError::from(e.to_string()))?;
            let datetime = NaiveDate::default().and_time(time);

            Ok(Value::from(datetime))
        }
        [_, ..] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Parses a datetime string with the specified format string and returns a [`Value::Number`].
/// See [`chrono::format::strftime`] for info on the syntax.
///
/// * Declaration: `string_to_datetime(datetime: String, format: String = '%Y-%m-%d %H:%M:%S'): Number`
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn string_to_datetime(params: &[Value]) -> NativeResult {
    let fmt = default_string(params, 1, "%Y-%m-%d %H:%M:%S")?;

    match params {
        [Value::String(s), ..] => {
            let datetime = NaiveDateTime::parse_from_str(s, fmt)
                .map_err(|e| NativeError::from(e.to_string()))?;

            Ok(Value::from(datetime))
        }
        [_, ..] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

fn naive_to_fixed(datetime: NaiveDateTime) -> Result<DateTime<FixedOffset>, NativeError> {
    Local
        .from_local_datetime(&datetime)
        .single()
        .map(|datetime| datetime.fixed_offset())
        .ok_or(NativeError::from("invalid datetime value"))
}

fn fixed_to_naive(datetime: DateTime<FixedOffset>) -> NaiveDateTime {
    Local.from_utc_datetime(&datetime.naive_utc()).naive_local()
}

/// Parses a [RFC 2822](https://www.rfc-editor.org/rfc/rfc2822) string
/// (e.g: `Fri, 21 Nov 1997 09:55:06 -0600`) and returns a [`Value::Number`].
///
/// * Declaration: `date_from_rfc2822(datetime: String): Number`
///
/// # Errors
///
/// Will return [`NativeError::CustomError`] if the String can not be parsed.
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn date_from_rfc2822(params: &[Value]) -> NativeResult {
    match params {
        [Value::String(value)] => {
            let datetime = DateTime::parse_from_rfc2822(value)
                .map_err(|e| NativeError::from(e.to_string()))?;

            Ok(Value::from(fixed_to_naive(datetime)))
        }
        [_] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Converts a datetime [`Value::Number`] into a [RFC 2822](https://www.rfc-editor.org/rfc/rfc2822)
/// [`Value::String`] (e.g: `Fri, 21 Nov 1997 09:55:06 +0000`).
///
/// * Declaration: `date_to_rfc2822(datetime: Number): String`
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn date_to_rfc2822(params: &[Value]) -> NativeResult {
    match params {
        [value] => {
            let datetime = NaiveDateTime::try_from(value)?;

            Ok(Value::String(naive_to_fixed(datetime)?.to_rfc2822()))
        }
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Parses a [RFC 3339](https://www.rfc-editor.org/rfc/rfc3339) [`Value::String`]
/// (e.g: `1997-11-21T09:55:06.00-06:00`) and returns a [`Value::Number`].
///
/// * Declaration: `date_from_rfc3339(datetime: String): Number`
///
/// # Errors
///
/// Will return [`NativeError::CustomError`] if the String can not be parsed.
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn date_from_rfc3339(params: &[Value]) -> NativeResult {
    match params {
        [Value::String(value)] => {
            let datetime = DateTime::parse_from_rfc3339(value)
                .map_err(|e| NativeError::from(e.to_string()))?;

            Ok(Value::from(fixed_to_naive(datetime)))
        }
        [_] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Converts a datetime [`Value::Number`] into a [RFC 3339](https://www.rfc-editor.org/rfc/rfc3339)
/// [`Value::String`] (e.g: `1997-11-21T09:55:06.00-06:00`).
///
/// * Declaration: `date_to_rfc3339(datetime: Number): String`
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn date_to_rfc3339(params: &[Value]) -> NativeResult {
    match params {
        [value] => {
            let datetime = NaiveDateTime::try_from(value)?;

            Ok(Value::String(naive_to_fixed(datetime)?.to_rfc3339()))
        }
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Returns the day of the week for a specified date.
///
/// * Declaration: `day_of_week(datetime: Number): Number`
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn day_of_week(params: &[Value]) -> NativeResult {
    match params {
        [value] => {
            let datetime = NaiveDateTime::try_from(value)?;

            Ok(Value::Number(f64::from(datetime.weekday() as u8)))
        }
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Constructs a datetime [`Value::Number`] from the `year`, `month`, and `day`.
///
/// * Declaration: `encode_date(year: Number, month: Number, day: Number): Number`
///
/// # Errors
///
/// Will return [`NativeError::CustomError`] if an under/overflow occures.
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn encode_date(params: &[Value]) -> NativeResult {
    match params {
        [Value::Number(year), Value::Number(month), Value::Number(day)] => {
            NaiveDate::from_ymd_opt(*year as i32, *month as u32, *day as u32)
                .map(|date| date.and_time(NaiveTime::default()))
                .map(Value::from)
                .ok_or(NativeError::from("invalid date parameters"))
        }
        [_, _, _] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(3)),
    }
}

/// Constructs a time [`Value`] from the `hour`, `minute`, `second`, and (optional) `millisecond`.
///
/// * Declaration: `encode_time(hour: Number, minute: Number, second: Number, millisecond: Number = 0): Number`
///
/// # Errors
///
/// Will return [`NativeError::CustomError`] if an under/overflow occures.
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
#[allow(clippy::module_name_repetitions)]
pub fn encode_time(params: &[Value]) -> NativeResult {
    let milli = default_number(params, 3, 0.0)?;

    match params {
        [Value::Number(hour), Value::Number(min), Value::Number(sec), ..] => NaiveDate::default()
            .and_hms_milli_opt(*hour as u32, *min as u32, *sec as u32, milli as u32)
            .map(Value::from)
            .ok_or(NativeError::from("invalid time parameters")),
        [_, _, _, ..] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(3)),
    }
}

/// Increases the month of the supplied datetime [`Value::Number`].
///
/// * Declaration: `inc_month(datetime: Number, increment: Number = 1): Number`
///
/// # Remarks
///
/// The increment parameter can be negative, which will decrement the month.
///
/// # Errors
///
/// Will return [`NativeError::CustomError`] if an under/overflow occures.
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn inc_month(params: &[Value]) -> NativeResult {
    let increment = default_number(params, 1, 1.0)?;

    match params {
        [value, ..] => {
            let datetime = NaiveDateTime::try_from(value).and_then(|datetime| {
                let delta = Months::new((increment as i32).unsigned_abs());

                if increment > 0.0 {
                    datetime
                        .checked_add_months(delta)
                        .ok_or(NativeError::from("inc_month increment overflow"))
                } else if increment < 0.0 {
                    datetime
                        .checked_sub_months(delta)
                        .ok_or(NativeError::from("inc_month decrement underflow"))
                } else {
                    Ok(datetime)
                }
            })?;

            Ok(Value::from(datetime))
        }
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Returns a [`Value::Boolean`] if the supplied datetime [`Value::Number`] is a leap year.
///
/// * Declaration: `is_leap_year(datetime: Number): Number`
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn is_leap_year(params: &[Value]) -> NativeResult {
    match params {
        [value] => {
            let is_leap_year = NaiveDateTime::try_from(value)
                .map(|datetime| datetime.year())
                .map(|year| year % 4 == 0 && (year % 100 != 0 || year % 400 == 0))?;

            Ok(Value::Boolean(is_leap_year))
        }
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Returns the year portion of a supplied Datetime as a [`Value::Number`].
///
/// * Declaration: `year(datetime: Number): Number`
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn year(params: &[Value]) -> NativeResult {
    match params {
        [value] => {
            let datetime = NaiveDateTime::try_from(value)?;

            Ok(Value::Number(f64::from(datetime.year())))
        }
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Returns the month portion of a supplied Datetime as a [`Value::Number`].
///
/// * Declaration: `month(datetime: Number): Number`
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn month(params: &[Value]) -> NativeResult {
    match params {
        [value] => {
            let datetime = NaiveDateTime::try_from(value)?;

            Ok(Value::Number(f64::from(datetime.month())))
        }
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Returns the day portion of a supplied Datetime as a [`Value::Number`].
///
/// * Declaration: `day(datetime: Number): Number`
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn day(params: &[Value]) -> NativeResult {
    match params {
        [value] => {
            let datetime = NaiveDateTime::try_from(value)?;

            Ok(Value::Number(f64::from(datetime.day())))
        }
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Returns the hour portion of a supplied Datetime as a [`Value::Number`].
///
/// * Declaration: `hour(datetime: Number): Number`
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn hour(params: &[Value]) -> NativeResult {
    match params {
        [value] => {
            let datetime = NaiveDateTime::try_from(value)?;

            Ok(Value::Number(f64::from(datetime.hour())))
        }
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Returns the minute portion of a supplied Datetime as a [`Value::Number`].
///
/// * Declaration: `minute(datetime: Number): Number`
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn minute(params: &[Value]) -> NativeResult {
    match params {
        [value] => {
            let datetime = NaiveDateTime::try_from(value)?;

            Ok(Value::Number(f64::from(datetime.minute())))
        }
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Returns the second portion of a supplied Datetime as a [`Value::Number`].
///
/// * Declaration: `second(datetime: Number): Number`
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn second(params: &[Value]) -> NativeResult {
    match params {
        [value] => {
            let datetime = NaiveDateTime::try_from(value)?;

            Ok(Value::Number(f64::from(datetime.second())))
        }
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Returns the millisecond portion of a supplied Datetime as a [`Value::Number`].
///
/// * Declaration: `millisecond(datetime: Number): Number`
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn millisecond(params: &[Value]) -> NativeResult {
    match params {
        [value] => {
            let datetime = NaiveDateTime::try_from(value)?;

            Ok(Value::Number(f64::from(datetime.nanosecond() / 1_000_000)))
        }
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

#[cfg(test)]
mod test {
    use chrono::NaiveDateTime;

    use super::*;
    use crate::Value;

    #[test]
    fn time_datetime_to_float() {
        let timestamp =
            NaiveDateTime::parse_from_str("2019-07-24 18:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let time_value = Value::from(timestamp);

        assert_eq!(Value::Number(18101.75), time_value);
        assert_eq!(NaiveDateTime::try_from(&time_value).unwrap(), timestamp);
    }

    #[test]
    fn time_date_to_string() {
        let date = date_to_string(&vec![
            Value::String(String::from("%Y-%m-%d %H:%M:%S")),
            Value::Number(18101.75),
        ])
        .unwrap();

        assert_eq!(Value::String(String::from("2019-07-24 18:00:00")), date);
    }

    #[test]
    fn time_string_to_date() {
        let date = string_to_date(&vec![Value::String(String::from("2019-07-24"))]).unwrap();

        assert_eq!(Value::Number(18101.0), date);

        let date = string_to_date(&vec![
            Value::String(String::from("07/24/2019")),
            Value::String(String::from("%m/%d/%Y")),
        ])
        .unwrap();

        assert_eq!(Value::Number(18101.0), date);
    }

    #[test]
    fn time_string_to_time() {
        let date = string_to_time(&vec![Value::String(String::from("12:00:00"))]).unwrap();

        assert_eq!(Value::Number(0.5), date);
    }

    #[test]
    fn time_string_to_datetime() {
        let date =
            string_to_datetime(&vec![Value::String(String::from("2019-07-24 12:00:00"))]).unwrap();

        assert_eq!(Value::Number(18101.5), date);
    }

    #[test]
    fn time_day_of_week() {
        assert_eq!(
            Ok(Value::Number(2.0)),
            day_of_week(&vec![Value::Number(18101.75)])
        );
    }

    #[test]
    fn time_encode_date() {
        let date = encode_date(&vec![
            Value::Number(2019.0),
            Value::Number(07.0),
            Value::Number(24.0),
        ]);

        assert_eq!(Ok(Value::Number(18101.0)), date);
    }

    #[test]
    fn time_encode_time() {
        let time = encode_time(&vec![
            Value::Number(18.0),
            Value::Number(0.0),
            Value::Number(0.0),
        ]);

        assert_eq!(Ok(Value::Number(0.75)), time);
    }

    #[test]
    fn time_encode_datetime() {
        let date = encode_date(&vec![
            Value::Number(2019.0),
            Value::Number(07.0),
            Value::Number(24.0),
        ]);

        let time = encode_time(&vec![
            Value::Number(18.0),
            Value::Number(0.0),
            Value::Number(0.0),
        ]);

        let datetime = date.unwrap() + time.unwrap();
        assert_eq!(Value::Number(18101.75), datetime.unwrap());
    }

    #[test]
    fn time_inc_month() {
        let date = encode_date(&vec![
            Value::Number(2023.0),
            Value::Number(12.0),
            Value::Number(1.0),
        ])
        .unwrap();

        let inc_one = encode_date(&vec![
            Value::Number(2024.0),
            Value::Number(01.0),
            Value::Number(1.0),
        ])
        .unwrap();

        let dec_one = encode_date(&vec![
            Value::Number(2023.0),
            Value::Number(11.0),
            Value::Number(1.0),
        ])
        .unwrap();

        assert_eq!(Ok(inc_one), inc_month(&vec![date.clone()]));
        assert_eq!(Ok(dec_one), inc_month(&vec![date, Value::Number(-1.0)]));
    }

    #[test]
    fn time_is_leap_year() {
        let year_2023 = encode_date(&vec![
            Value::Number(2023.0),
            Value::Number(1.0),
            Value::Number(1.0),
        ])
        .unwrap();

        let year_2024: Value = encode_date(&vec![
            Value::Number(2024.0),
            Value::Number(1.0),
            Value::Number(1.0),
        ])
        .unwrap();

        assert_eq!(Ok(Value::Boolean(false)), is_leap_year(&vec![year_2023]));
        assert_eq!(Ok(Value::Boolean(true)), is_leap_year(&vec![year_2024]));
    }

    #[allow(dead_code)]
    // #[test] // dependent on the local timezone
    fn time_rfc2822() {
        let rfc = Value::String(String::from("Fri, 28 Nov 2014 12:00:00 +0100"));
        let date = date_from_rfc2822(&vec![rfc.clone()]).unwrap();

        assert_eq!(Value::Number(16402.5), date);
        assert_eq!(Ok(rfc), date_to_rfc2822(&vec![date]));
    }

    #[allow(dead_code)]
    // #[test] // dependent on the local timezone
    fn time_rfc3339() {
        let rfc = Value::String(String::from("2014-11-28T12:00:00+01:00"));
        let date = date_from_rfc3339(&vec![rfc.clone()]).unwrap();

        assert_eq!(Value::Number(16402.5), date);
        assert_eq!(Ok(rfc), date_to_rfc3339(&vec![date]));

        let rfc = Value::String(String::from("2014-11-28T01:00:00+01:00"));
        let date = date_from_rfc3339(&vec![rfc.clone()]).unwrap();

        assert_eq!(Value::Number(16402.0 + 1. / 24.), date);
        assert_eq!(Ok(rfc), date_to_rfc3339(&vec![date.clone()]));

        let rfc = Value::String(String::from("2014-11-28T00:00:00Z"));
        let date_utc = date_from_rfc3339(&vec![rfc.clone()]).unwrap();

        assert_eq!(date, date_utc);
    }

    #[test]
    fn time_extract_functions() {
        let date = Value::Number(13734.424444594908); // 2007-08-09 10:11:12.013

        assert_eq!(Ok(Value::Number(2007.0)), year(&vec![date.clone()]));
        assert_eq!(Ok(Value::Number(08.0)), month(&vec![date.clone()]));
        assert_eq!(Ok(Value::Number(09.0)), day(&vec![date.clone()]));
        assert_eq!(Ok(Value::Number(10.0)), hour(&vec![date.clone()]));
        assert_eq!(Ok(Value::Number(11.0)), minute(&vec![date.clone()]));
        assert_eq!(Ok(Value::Number(12.0)), second(&vec![date.clone()]));
        assert_eq!(Ok(Value::Number(13.0)), millisecond(&vec![date.clone()]));
    }
}

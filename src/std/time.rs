//! Optional module to perform date and time operations using [`Value::Number`].
//!
//! While SLAC does not have a dedicated [`Value`] type for timestamps, this module
//! enables date, time and datetime manipulation on [`Value::Number`] floating
//! point values.
//!
//! The integral part of the [`Value::Number`] float is the number of **days**,
//! which have passed since `midnight, January 1, 1970, UTC` (aka. the UNIX timestamp).
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
//! This module uses the [`chrono`] library and can be included using
//! the `chrono` feature.
use chrono::{DateTime, Datelike, Months, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};

use crate::{StaticEnvironment, Value};

use super::error::{NativeError, NativeResult};

/// Extends a [`StaticEnvironment`] with `time` conversion functions.
pub fn extend_environment(env: &mut StaticEnvironment) {
    env.add_native_func("date_to_string", Some(2), date_to_string);
    env.add_native_func("time_to_string", Some(2), date_to_string);
    env.add_native_func("string_to_date", Some(1), string_to_date);
    env.add_native_func("string_to_time", Some(1), string_to_time);
    env.add_native_func("string_to_date_time", Some(1), string_to_date_time);
    env.add_native_func("day_of_week", Some(1), day_of_week);
    env.add_native_func("encode_date", Some(3), encode_date);
    env.add_native_func("encode_time", Some(3), encode_time);
    env.add_native_func("inc_month", Some(1), inc_month);
    env.add_native_func("is_leap_year", Some(1), is_leap_year);
    env.add_native_func("date_from_rfc2822", Some(1), date_from_rfc2822);
    env.add_native_func("date_from_rfc3339", Some(1), date_from_rfc3339);
}

const MILLISECONDS_PER_DAY: f64 = 24. * 60. * 60. * 1000.;

impl TryFrom<&Value> for NaiveDateTime {
    type Error = NativeError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(value) => {
                let milliseconds = (value * MILLISECONDS_PER_DAY) as i64;

                NaiveDateTime::from_timestamp_millis(milliseconds)
                    .ok_or(NativeError::from("datetime out of range"))
            }
            _ => Err(NativeError::WrongParameterType),
        }
    }
}

impl From<NaiveDateTime> for Value {
    fn from(val: NaiveDateTime) -> Self {
        let milliseconds = val.timestamp_millis();

        Value::Number(milliseconds as f64 / MILLISECONDS_PER_DAY)
    }
}

/// Formats a datetime [`Value`] with the specified format string.
/// See [`chrono::format::strftime`] for info on the syntax.
///
/// # Errors
///
/// Returns an error if there are not enough parameters or the parameters are of
/// the wrong [`Value`] type.
pub fn date_to_string(params: &[Value]) -> NativeResult {
    match (params.get(0), params.get(1)) {
        (Some(Value::String(fmt)), Some(value)) => {
            let datetime = NaiveDateTime::try_from(value)?;

            Ok(Value::String(datetime.format(fmt).to_string()))
        }
        (Some(_), _) => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::NotEnoughParameters(2)),
    }
}

/// Parses a date string with the specified format string and returns a [`Value::Number`].
/// See [`chrono::format::strftime`] for info on the syntax.
///
/// # Errors
///
/// Returns an error if there are not enough parameters or the parameters are of
/// the wrong [`Value`] type.
pub fn string_to_date(params: &[Value]) -> NativeResult {
    match (
        params.get(0),
        params
            .get(1)
            .unwrap_or(&Value::String(String::from("%Y-%m-%d"))),
    ) {
        (Some(Value::String(s)), Value::String(fmt)) => Ok(NaiveDate::parse_from_str(s, fmt)
            .map_err(|e| NativeError::from(e.to_string()))?
            .and_time(NaiveTime::default())
            .into()),
        (Some(_), _) => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::NotEnoughParameters(1)),
    }
}

/// Parses a time string with the specified format string and returns a [`Value::Number`].
/// See [`chrono::format::strftime`] for info on the syntax.
///
/// # Errors
///
/// Returns an error if there are not enough parameters or the parameters are of
/// the wrong [`Value`] type.
pub fn string_to_time(params: &[Value]) -> NativeResult {
    match (
        params.get(0),
        params
            .get(1)
            .unwrap_or(&Value::String(String::from("%H:%M:%S"))),
    ) {
        (Some(Value::String(s)), Value::String(fmt)) => {
            let time =
                NaiveTime::parse_from_str(s, fmt).map_err(|e| NativeError::from(e.to_string()))?;

            Ok(NaiveDate::default().and_time(time).into())
        }
        (Some(_), _) => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::NotEnoughParameters(1)),
    }
}

/// Parses a datetime string with the specified format string and returns a [`Value::Number`].
/// See [`chrono::format::strftime`] for info on the syntax.
///
/// # Errors
///
/// Returns an error if there are not enough parameters or the parameters are of
/// the wrong [`Value`] type.
pub fn string_to_date_time(params: &[Value]) -> NativeResult {
    match (
        params.get(0),
        params
            .get(1)
            .unwrap_or(&Value::String(String::from("%Y-%m-%d %H:%M:%S"))),
    ) {
        (Some(Value::String(s)), Value::String(fmt)) => Ok(NaiveDateTime::parse_from_str(s, fmt)
            .map_err(|e| NativeError::from(e.to_string()))?
            .into()),
        (Some(_), _) => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::NotEnoughParameters(1)),
    }
}

/// Parses a [RFC 2822](https://www.rfc-editor.org/rfc/rfc2822) string
/// (e.g: `Fri, 21 Nov 1997 09:55:06 -0600`) and returns a [`Value::Number`].
///
/// # Errors
///
/// Returns an error if there are not enough parameters or the parameters are of
/// the wrong [`Value`] type.
pub fn date_from_rfc2822(params: &[Value]) -> NativeResult {
    match params.first() {
        Some(Value::String(value)) => Ok(DateTime::parse_from_rfc2822(value)
            .map_err(|e| NativeError::from(e.to_string()))?
            .naive_utc()
            .into()),
        Some(_) => Err(NativeError::WrongParameterType),
        None => Err(NativeError::NotEnoughParameters(1)),
    }
}

/// Converts a datetime [`Value::Number`] into a [RFC 2822](https://www.rfc-editor.org/rfc/rfc2822)
/// [`Value::String`] (e.g: `Fri, 21 Nov 1997 09:55:06 +0000`).
///
/// # Errors
///
/// Returns an error if there are not enough parameters or the parameters are of
/// the wrong [`Value`] type.
pub fn date_to_rfc2822(params: &[Value]) -> NativeResult {
    match params.first() {
        Some(value) => {
            let datetime = NaiveDateTime::try_from(value)?;

            Ok(Value::String(Utc.from_utc_datetime(&datetime).to_rfc2822()))
        }
        None => Err(NativeError::NotEnoughParameters(1)),
    }
}

/// Parses a [RFC 3339](https://www.rfc-editor.org/rfc/rfc3339) [`Value::String`]
/// (e.g: `1997-11-21T09:55:06.00-06:00`) and returns a [`Value::Number`].
///
/// # Errors
///
/// Returns an error if there are not enough parameters or the parameters are of
/// the wrong [`Value`] type.
pub fn date_from_rfc3339(params: &[Value]) -> NativeResult {
    match params.first() {
        Some(Value::String(value)) => Ok(DateTime::parse_from_rfc3339(value)
            .map_err(|e| NativeError::from(e.to_string()))?
            .naive_utc()
            .into()),
        Some(_) => Err(NativeError::WrongParameterType),
        None => Err(NativeError::NotEnoughParameters(1)),
    }
}

/// Converts a datetime [`Value::Number`] into a [RFC 3339](https://www.rfc-editor.org/rfc/rfc3339)
/// [`Value::String`] (e.g: `1997-11-21T09:55:06.00-06:00`).
///
/// # Errors
///
/// Returns an error if there are not enough parameters or the parameters are of
/// the wrong [`Value`] type.
pub fn date_to_rfc3339(params: &[Value]) -> NativeResult {
    match params.first() {
        Some(value) => {
            let datetime = NaiveDateTime::try_from(value)?;

            Ok(Value::String(Utc.from_utc_datetime(&datetime).to_rfc3339()))
        }
        None => Err(NativeError::NotEnoughParameters(1)),
    }
}

/// Returns the day of the week for a specified date.
///
/// # Errors
///
/// Returns an error if there are not enough parameters or the parameters are of
/// the wrong [`Value`] type.
pub fn day_of_week(params: &[Value]) -> NativeResult {
    match params.first() {
        Some(value) => {
            let datetime = NaiveDateTime::try_from(value)?;
            Ok(Value::Number(f64::from(datetime.weekday() as u8)))
        }
        None => Err(NativeError::NotEnoughParameters(1)),
    }
}

/// Constructs a date [`Value`] according to the specified `year`, `month`, and `day`.
///
/// # Errors
///
/// Returns an error if there are not enough parameters or the parameters are of
/// the wrong [`Value`] type.
pub fn encode_date(params: &[Value]) -> NativeResult {
    match (params.get(0), params.get(1), params.get(2)) {
        (Some(Value::Number(year)), Some(Value::Number(month)), Some(Value::Number(day))) => {
            NaiveDate::from_ymd_opt(*year as i32, *month as u32, *day as u32)
                .map(|date| date.and_time(NaiveTime::default()))
                .and_then(|datetime| datetime.try_into().ok())
                .ok_or(NativeError::from("invalid date"))
        }
        (Some(_), Some(_), Some(_)) => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::NotEnoughParameters(3)),
    }
}

/// Constructs a time [`Value`] according to the specified `hour`, `minute`,
/// `second`, and (optionally) `millisecond`.
///
/// # Errors
///
/// Returns an error if there are not enough parameters or the parameters are of
/// the wrong [`Value`] type.
pub fn encode_time(params: &[Value]) -> NativeResult {
    match (
        params.get(0),
        params.get(1),
        params.get(2),
        params.get(3).unwrap_or(&Value::Number(0.0)),
    ) {
        (
            Some(Value::Number(hour)),
            Some(Value::Number(min)),
            Some(Value::Number(sec)),
            Value::Number(milli),
        ) => NaiveDate::default()
            .and_hms_milli_opt(*hour as u32, *min as u32, *sec as u32, *milli as u32)
            .map(|v| v.into())
            .ok_or(NativeError::from("invalid times parameters")),
        (Some(_), Some(_), Some(_), _) => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::NotEnoughParameters(3)),
    }
}

/// Increases the month of the supplied date [`Value`] by an optional integer
/// value or 1.
///
/// # Remarks
///
/// The increment parameter can be negative, which will decrement the month.
///
/// # Errors
///
/// Returns an error if there are not enough parameters or the parameters are of
/// the wrong [`Value`] type.
pub fn inc_month(params: &[Value]) -> NativeResult {
    match (params.get(0), params.get(1).unwrap_or(&Value::Number(1.0))) {
        (Some(value), Value::Number(increment)) => Ok(NaiveDateTime::try_from(value)
            .and_then(|datetime| {
                let delta = Months::new(increment.abs() as u32);
                if increment > &0.0 {
                    datetime
                        .checked_add_months(delta)
                        .ok_or(NativeError::from("inc_month overflow"))
                } else {
                    datetime
                        .checked_sub_months(delta)
                        .ok_or(NativeError::from("inc_month undeflow"))
                }
            })?
            .into()),
        (Some(_), _) => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::NotEnoughParameters(1)),
    }
}

/// Returns a [`Value::Boolean`] if the supplied date [`Value`] is a leap year.
///
/// # Errors
///
/// Returns an error if there are not enough parameters or the parameters are of
/// the wrong [`Value`] type.
pub fn is_leap_year(params: &[Value]) -> NativeResult {
    match params.first() {
        Some(value) => {
            let is_leap_year = NaiveDateTime::try_from(value)
                .map(|datetime| datetime.year())
                .map(|year| year % 4 == 0 && (year % 100 != 0 || year % 400 == 0))?;

            Ok(Value::Boolean(is_leap_year))
        }
        _ => Err(NativeError::NotEnoughParameters(1)),
    }
}

#[cfg(test)]
mod test {
    use chrono::NaiveDateTime;

    use super::{
        date_from_rfc2822, date_from_rfc3339, date_to_rfc2822, date_to_rfc3339, date_to_string,
        day_of_week, encode_date, encode_time, inc_month, is_leap_year, string_to_date,
        string_to_date_time, string_to_time,
    };
    use crate::Value;

    #[test]
    fn time_datetime_to_float() {
        let timestamp =
            NaiveDateTime::parse_from_str("2019-07-24 18:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let time_value = timestamp.into();

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
    fn time_string_to_date_time() {
        let date =
            string_to_date_time(&vec![Value::String(String::from("2019-07-24 12:00:00"))]).unwrap();

        assert_eq!(Value::Number(18101.5), date);
    }

    #[test]
    fn time_date_from_rfc() {
        assert_eq!(
            Ok(Value::Number(18101.75)),
            date_from_rfc2822(&vec![Value::String(String::from(
                "Wed, 24 Jul 2019 18:00:00 +0000"
            ))])
        );

        assert_eq!(
            Ok(Value::Number(18101.75)),
            date_from_rfc3339(&vec![Value::String(String::from("2019-07-24T18:00:00Z"))])
        );
    }

    #[test]
    fn time_day_of_week() {
        let day = day_of_week(&vec![Value::Number(18101.75)]);

        assert_eq!(Ok(Value::Number(2.0)), day);
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

    #[test]
    fn time_rfc2822() {
        let rfc = Value::String(String::from("Fri, 28 Nov 2014 12:00:00 +0000"));
        let date = date_from_rfc2822(&vec![rfc.clone()]).unwrap();

        assert_eq!(Value::Number(16402.5), date);
        assert_eq!(Ok(rfc), date_to_rfc2822(&vec![date]));
    }

    #[test]
    fn time_rfc3339() {
        let rfc = Value::String(String::from("2014-11-28T12:00:00+00:00"));
        let date = date_from_rfc3339(&vec![rfc.clone()]).unwrap();

        assert_eq!(Value::Number(16402.5), date);
        assert_eq!(Ok(rfc), date_to_rfc3339(&vec![date]));

        let rfc = Value::String(String::from("2014-11-28T00:00:00+00:00"));
        let date = date_from_rfc3339(&vec![rfc.clone()]).unwrap();

        assert_eq!(Value::Number(16402.0), date);
        assert_eq!(Ok(rfc), date_to_rfc3339(&vec![date.clone()]));

        let rfc = Value::String(String::from("2014-11-28T00:00:00Z"));
        let date_utc = date_from_rfc3339(&vec![rfc.clone()]).unwrap();

        assert_eq!(date, date_utc);
    }
}

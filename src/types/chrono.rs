use chrono::prelude::*;

use FromSql;
use Result;
use Timestamp;
use OracleType;
use IntervalDS;
use ToSql;
use Value;
use Error;
use error::ConversionError;
use chrono::Duration;

//
// chrono::DateTime<Utc>
// chrono::DateTime<Local>
// chrono::DateTime<FixedOffset>
//

// TODO: use TimeZone.ymd_opt and Data.and_hms_nano_opt instead of TimeZone.ymd and Data.and_hms_nano.

fn datetime_from_sql<Tz>(tz: &Tz, ts: &Timestamp) -> Result<DateTime<Tz>> where Tz: TimeZone {
    Ok(tz.ymd(ts.year, ts.month, ts.day).and_hms_nano(ts.hour, ts.minute, ts.second, ts.nanosecond))
}

impl FromSql for DateTime<Utc> {
    fn from(val: &Value) -> Result<DateTime<Utc>> {
        let ts = val.as_timestamp()?;
        datetime_from_sql(&Utc, &ts)
    }
}

impl FromSql for DateTime<Local> {
    fn from(val: &Value) -> Result<DateTime<Local>> {
        let ts = val.as_timestamp()?;
        datetime_from_sql(&Local, &ts)
    }
}

impl FromSql for DateTime<FixedOffset> {
    fn from(val: &Value) -> Result<DateTime<FixedOffset>> {
        let ts = val.as_timestamp()?;
        datetime_from_sql(&FixedOffset::east(ts.tz_offset()), &ts)
    }
}

impl<Tz> ToSql for DateTime<Tz> where Tz: TimeZone {
    fn oratype() -> OracleType {
        OracleType::Timestamp(9)
    }

    fn to(&self, val: &mut Value) -> Result<()> {
        let ts = Timestamp::new(self.year(), self.month(), self.day(),
                                self.hour(), self.minute(), self.second(),
                                self.nanosecond());
        let ts = ts.and_tz_offset(self.offset().fix().local_minus_utc());
        let ts = ts.with_precision(9);
        val.set_timestamp(&ts)
    }
}

//
// chrono::Date<Utc>
// chrono::Date<Local>
// chrono::Date<FixedOffset>
//

fn date_from_sql<Tz>(tz: &Tz, ts: &Timestamp) -> Result<Date<Tz>> where Tz: TimeZone {
    Ok(tz.ymd(ts.year, ts.month, ts.day))
}

impl FromSql for Date<Utc> {
    fn from(val: &Value) -> Result<Date<Utc>> {
        let ts = val.as_timestamp()?;
        date_from_sql(&Utc, &ts)
    }
}

impl FromSql for Date<Local> {
    fn from(val: &Value) -> Result<Date<Local>> {
        let ts = val.as_timestamp()?;
        date_from_sql(&Local, &ts)
    }
}

impl FromSql for Date<FixedOffset> {
    fn from(val: &Value) -> Result<Date<FixedOffset>> {
        let ts = val.as_timestamp()?;
        date_from_sql(&FixedOffset::east(ts.tz_offset()), &ts)
    }
}

impl<Tz> ToSql for Date<Tz> where Tz: TimeZone {
    fn oratype() -> OracleType {
        OracleType::Timestamp(9)
    }

    fn to(&self, val: &mut Value) -> Result<()> {
        let ts = Timestamp::new(self.year(), self.month(), self.day(),
                                0, 0, 0, 0);
        let ts = ts.and_tz_offset(self.offset().fix().local_minus_utc());
        val.set_timestamp(&ts)
    }
}

//
// chrono::Duration
//

impl FromSql for Duration {
    fn from(val: &Value) -> Result<Duration> {
        let err = |it: IntervalDS| Error::ConversionError(ConversionError::Overflow(it.to_string(), "Duration"));
        let it = val.as_interval_ds()?;
        let d = Duration::milliseconds(0);
        let d = d.checked_add(&Duration::days(it.days as i64)).ok_or(err(it))?;
        let d = d.checked_add(&Duration::hours(it.hours as i64)).ok_or(err(it))?;
        let d = d.checked_add(&Duration::minutes(it.minutes as i64)).ok_or(err(it))?;
        let d = d.checked_add(&Duration::seconds(it.seconds as i64)).ok_or(err(it))?;
        let d = d.checked_add(&Duration::nanoseconds(it.nanoseconds as i64)).ok_or(err(it))?;
        Ok(d)
    }
}

impl ToSql for Duration {
    fn oratype() -> OracleType {
        OracleType::IntervalDS(9, 9)
    }

    fn to(&self, val: &mut Value) -> Result<()> {
        let secs = self.num_seconds();
        let nsecs = (*self - Duration::seconds(secs)).num_nanoseconds().unwrap();
        let days = secs / (24 * 60 * 60);
        let secs = secs % (24 * 60 * 60);
        let hours = secs / (60 * 60);
        let secs = secs % (60 * 60);
        let minutes = secs / 60;
        let secs = secs % 60;
        if days.abs() >= 1000000000 {
            return Err(Error::ConversionError(ConversionError::Overflow(self.to_string(), "INTERVAL DAY TO SECOND")));
        }
        let it = IntervalDS::new(days as i32, hours as i32, minutes as i32, secs as i32, nsecs as i32);
        val.set_interval_ds(&it)
    }
}
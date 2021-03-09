use chrono::{DateTime, Utc};
use serde::Deserialize;

pub(crate) fn datetime_from_unix_timestamp<'de, D>(
    deserializer: D
) -> Result<DateTime<Utc>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let timestamp = chrono::NaiveDateTime::from_timestamp(i64::deserialize(deserializer)?, 0);
    Ok(DateTime::<Utc>::from_utc(timestamp, Utc))
}

pub(crate) fn datetime_from_nano_timestamp<'de, D>(
    deserializer: D
) -> Result<DateTime<Utc>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let timestamp_nano = u64::deserialize(deserializer)?;
    let timestamp = chrono::NaiveDateTime::from_timestamp(
        (timestamp_nano / 1_000_000_000) as i64,
        (timestamp_nano % 1_000_000_000) as u32,
    );
    Ok(DateTime::<Utc>::from_utc(timestamp, Utc))
}

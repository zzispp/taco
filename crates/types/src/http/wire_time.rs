use serde::Serializer;
use time::{OffsetDateTime, UtcOffset, format_description::BorrowedFormatItem};

const UTC_RFC3339_MILLIS: &[BorrowedFormatItem<'static>] =
    time::macros::format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z");

pub fn format_utc_rfc3339_millis(value: OffsetDateTime) -> Result<String, time::error::Format> {
    value.to_offset(UtcOffset::UTC).format(UTC_RFC3339_MILLIS)
}

pub fn serialize_utc_rfc3339_millis<S>(value: &OffsetDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let formatted = format_utc_rfc3339_millis(*value).map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(&formatted)
}

#[cfg(test)]
mod tests {
    use serde::Serialize;
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};

    use super::{format_utc_rfc3339_millis, serialize_utc_rfc3339_millis};

    #[test]
    fn wire_time_is_utc_with_exactly_three_fractional_digits() {
        assert_eq!(format("2026-07-15T09:23:45.678999+08:00"), "2026-07-15T01:23:45.678Z");
        assert_eq!(format("2026-07-15T01:23:45Z"), "2026-07-15T01:23:45.000Z");
    }

    #[test]
    fn serde_helper_uses_the_same_wire_contract() {
        #[derive(Serialize)]
        struct Fixture {
            #[serde(serialize_with = "serialize_utc_rfc3339_millis")]
            occurred_at: OffsetDateTime,
        }

        let occurred_at = OffsetDateTime::parse("2026-07-15T09:23:45.006+08:00", &Rfc3339).unwrap();
        let value = serde_json::to_value(Fixture { occurred_at }).unwrap();

        assert_eq!(value, serde_json::json!({"occurred_at": "2026-07-15T01:23:45.006Z"}));
    }

    fn format(value: &str) -> String {
        let value = OffsetDateTime::parse(value, &Rfc3339).unwrap();
        format_utc_rfc3339_millis(value).unwrap()
    }
}

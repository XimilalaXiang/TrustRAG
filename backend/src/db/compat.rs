use uuid::Uuid;

pub fn uuid_to_db(id: Uuid) -> String {
    id.to_string()
}

pub fn parse_uuid(s: &str) -> Result<Uuid, uuid::Error> {
    s.parse()
}

#[cfg(feature = "postgres")]
pub fn current_timestamp_sql() -> &'static str {
    "now()"
}

#[cfg(sqlite_mode)]
pub fn current_timestamp_sql() -> &'static str {
    "strftime('%Y-%m-%dT%H:%M:%fZ', 'now')"
}

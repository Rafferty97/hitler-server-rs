use chrono::prelude::{DateTime, Utc};

pub fn iso8601(st: std::time::SystemTime) -> String {
    let dt: DateTime<Utc> = st.into();
    dt.format("%+").to_string()
}

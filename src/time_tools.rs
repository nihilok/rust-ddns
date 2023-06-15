use chrono::prelude::{DateTime, Utc};
use std::time;

pub fn now_as_string() -> String {
    let t = time::SystemTime::now();
    iso8601(&t)
}

fn iso8601(st: &time::SystemTime) -> String {
    let dt: DateTime<Utc> = st.clone().into();
    format!("{}", dt.format("%+"))
    // formats like "2001-07-08T00:34:60.026490+09:30"
}

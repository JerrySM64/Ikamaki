pub mod exporter;
pub mod generator;
pub mod schedule;

// If no start time is specified, use the last even hour
#[must_use]
pub fn default_timestamp() -> i64 {
    let now = chrono::Utc::now().timestamp();
    let block = 2 * 60 * 60;
    now - (now % block)
}

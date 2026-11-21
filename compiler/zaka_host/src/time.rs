//! Clock units and UTC text shared across the VM and native runtime.

use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use chrono::DateTime;
use chrono::Datelike;

use crate::World;

/// Read the clock for elapsed time, in signed milliseconds.
///
/// # Arguments
///
/// * `world` - The clock of the current run.
///
/// # Returns
///
/// Milliseconds from the clock's origin.
pub fn elapsed_ms(world: &mut dyn World) -> i64
{
    i64::try_from(world.now_ms()).unwrap_or(i64::MAX)
}

/// Current Unix time in whole milliseconds.
pub(crate) fn unix_ms() -> i64
{
    let ms = match SystemTime::now().duration_since(UNIX_EPOCH)
    {
        Ok(elapsed) => elapsed.as_millis() as i128,
        Err(before) => -(before.duration().as_nanos().div_ceil(1_000_000) as i128),
    };
    ms.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
}

/// Format a Unix millisecond stamp in UTC for dates in 0000 through 9999.
///
/// # Arguments
///
/// * `ms` - The signed milliseconds since the Unix epoch.
///
/// # Returns
///
/// ISO 8601 text with three fractional digits, or `None` outside the range.
pub fn format_utc(ms: i64) -> Option<String>
{
    let stamp = DateTime::from_timestamp_millis(ms)?;
    (0..=9999)
        .contains(&stamp.year())
        .then(|| stamp.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
}

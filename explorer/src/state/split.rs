use zakac_rust_sugars::tern;

/// Smallest share a pane can take of its split.
pub const MIN_SPLIT: f64 = 0.2;

/// Largest share a pane can take of its split.
pub const MAX_SPLIT: f64 = 0.8;

/// Share of the width the editor takes at first.
pub const DEFAULT_EDITOR_SHARE: f64 = 0.5;

/// Share of the right column the stage view takes at first.
pub const DEFAULT_STAGE_SHARE: f64 = 0.65;

/// Step of a splitter moved with the arrows.
pub const SPLIT_STEP: f64 = 0.02;

/// Keep a split share inside [`MIN_SPLIT`] and [`MAX_SPLIT`].
///
/// # Arguments
///
/// * `share` - The share; a value that is not a number gives the middle.
///
/// # Returns
///
/// The clamped share.
pub fn clamp_split(share: f64) -> f64
{
    let share = tern!(share.is_nan(), 0.5, share);
    share.clamp(MIN_SPLIT, MAX_SPLIT)
}

/// The share of a container that lies before the pointer.
///
/// # Arguments
///
/// * `pointer` - The pointer position along the split axis.
/// * `start` - Where the container starts along that axis.
/// * `len` - The container length along that axis.
///
/// # Returns
///
/// The clamped share.
pub fn split_at(pointer: f64, start: f64, len: f64) -> f64
{
    let share = tern!(len > 0.0, (pointer - start) / len, 0.5);
    clamp_split(share)
}

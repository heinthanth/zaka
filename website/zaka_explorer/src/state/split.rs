use zakac_rust_sugars::tern;

/// Smallest share a pane can take of its split.
pub const MIN_SPLIT: f64 = 0.2;

/// Largest share a pane can take of its split.
pub const MAX_SPLIT: f64 = 0.8;

/// Share of the width the editor takes at first.
pub const DEFAULT_EDITOR_SHARE: f64 = 0.5;

/// Share of the right column the stage view takes at first.
pub const DEFAULT_STAGE_SHARE: f64 = 0.65;

/// Most of the right column the stage view keeps when errors show up, so
/// the problems panel gets at least the rest.
pub const ERROR_STAGE_SHARE: f64 = 0.5;

/// Share of the debug row the stage view takes at first; the dock takes
/// the rest.
pub const DEFAULT_DOCK_SHARE: f64 = 0.6;

/// Share of the dock panes the stack takes at first; the globals and the
/// heap take the rest.
pub const DEFAULT_STACK_SHARE: f64 = 0.45;

/// Share of the panes under the stack the globals take at first; the heap
/// takes the rest.
pub const DEFAULT_GLOBALS_SHARE: f64 = 0.3;

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

/// A split share as a grid track size.
///
/// # Arguments
///
/// * `share` - The share of a pane.
///
/// # Returns
///
/// The share, clamped, in `fr` units of a total of 1.
pub fn share_fr(share: f64) -> String { format!("{}fr", clamp_split(share)) }

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

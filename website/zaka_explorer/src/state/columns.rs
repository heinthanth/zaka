//! The hidden columns and the widths of the columns of a phase table. The
//! hidden list holds column numbers, from 1, with spaces between them, as the
//! head of the table carries it in `data-hide`; the width list holds a width
//! in px per column. Local storage keeps both.

use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

/// The column numbers of a hidden list that name a column of the table.
///
/// # Arguments
///
/// * `list` - The hidden list.
/// * `count` - The number of columns.
///
/// # Returns
///
/// The numbers, in the order of the list.
fn hidden_numbers(list: &str, count: usize) -> impl Iterator<Item = usize> + '_
{
    let numbers = list.split_whitespace().filter_map(|word| word.parse().ok());
    numbers.filter(move |n| (1..=count).contains(n))
}

/// Whether a column is in a hidden list.
///
/// # Arguments
///
/// * `list` - The hidden list.
/// * `n` - The column number, from 1.
///
/// # Returns
///
/// `true` when the column is hidden.
pub fn is_hidden(list: &str, n: usize) -> bool
{
    list.split_whitespace().any(|word| word.parse() == Ok(n))
}

/// The number of columns a hidden list leaves shown.
///
/// # Arguments
///
/// * `list` - The hidden list.
/// * `count` - The number of columns.
///
/// # Returns
///
/// The count.
pub fn shown_count(list: &str, count: usize) -> usize
{
    let mut hidden: Vec<usize> = hidden_numbers(list, count).collect();
    hidden.sort_unstable();
    hidden.dedup();
    count - hidden.len()
}

/// A hidden list with a column hidden, or shown again when it was hidden.
/// The last shown column remains, so a table never loses all its columns.
///
/// # Arguments
///
/// * `list` - The hidden list.
/// * `n` - The column number, from 1.
/// * `count` - The number of columns.
///
/// # Returns
///
/// The new list, in column order, without stale entries.
pub fn toggle_column(list: &str, n: usize, count: usize) -> String
{
    let hide = !is_hidden(list, n) && shown_count(list, count) > 1;
    let mut hidden: Vec<usize> =
        hidden_numbers(list, count).filter(|h| *h != n).collect();
    do_if!(hide, hidden.push(n));
    hidden.sort_unstable();
    hidden.dedup();
    let words: Vec<String> = hidden.iter().map(usize::to_string).collect();
    words.join(" ")
}

/// Least width in px a column can be dragged to.
pub const MIN_WIDTH: f64 = 32.0;

/// The widths of a width list, when it holds one per column.
///
/// # Arguments
///
/// * `list` - The width list.
/// * `count` - The number of columns.
///
/// # Returns
///
/// The widths in px, or `None` while the columns take their natural widths.
pub fn parse_widths(list: &str, count: usize) -> Option<Vec<f64>>
{
    let widths: Option<Vec<f64>> =
        list.split_whitespace().map(|word| word.parse().ok()).collect();
    widths.filter(|w| w.len() == count && count > 0)
}

/// A width list from widths, in whole px, cut down so the sum never goes over
/// the sum of the widths; a table with fixed widths spreads what is left over
/// its columns, but grows past its box for each extra px.
///
/// # Arguments
///
/// * `widths` - The widths in px.
///
/// # Returns
///
/// The list.
pub fn widths_list(widths: &[f64]) -> String
{
    let words: Vec<String> = widths.iter().map(|w| w.floor().to_string()).collect();
    words.join(" ")
}

/// The next shown column to the right of a column, whose left edge is the
/// right edge of the column.
///
/// # Arguments
///
/// * `list` - The hidden list.
/// * `n` - The column number, from 1.
/// * `count` - The number of columns.
///
/// # Returns
///
/// Its number, from 1, or `None` for the last shown column.
pub fn next_shown(list: &str, n: usize, count: usize) -> Option<usize>
{
    (n + 1..=count).find(|m| !is_hidden(list, *m))
}

/// The widths with the edge between two columns moved: one column grows as
/// much as the other gives up, so the table keeps its width, and neither goes
/// under [`MIN_WIDTH`].
///
/// # Arguments
///
/// * `widths` - The widths in px.
/// * `n` - The column left of the edge, from 1.
/// * `next` - The column right of the edge, from 1.
/// * `shift` - How far the edge moves, in px; to the right when positive.
///
/// # Returns
///
/// The new widths; the same widths when a column number is out of range.
pub fn drag_edge(widths: &[f64], n: usize, next: usize, shift: f64) -> Vec<f64>
{
    let mut widths = widths.to_vec();
    let (Some(left), Some(right)) = (n.checked_sub(1), next.checked_sub(1))
    else
    {
        return widths;
    };
    ret_if!(left.max(right) >= widths.len(), widths);
    let room = (widths[right] - MIN_WIDTH).max(0.0);
    let give = (widths[left] - MIN_WIDTH).max(0.0);
    let shift = shift.clamp(-give, room);
    widths[left] += shift;
    widths[right] -= shift;
    widths
}

/// The widths with the shown columns scaled so their sum is a total; the
/// hidden columns keep theirs.
///
/// # Arguments
///
/// * `widths` - The widths in px.
/// * `list` - The hidden list.
/// * `total` - The total in px.
///
/// # Returns
///
/// The new widths; the same widths when the shown ones add up to nothing.
pub fn fit_widths(widths: &[f64], list: &str, total: f64) -> Vec<f64>
{
    let sum = shown_width(widths, list);
    ret_if!(sum <= 0.0, widths.to_vec());
    let scale =
        |(i, w): (usize, &f64)| tern!(is_hidden(list, i + 1), *w, w * total / sum);
    widths.iter().enumerate().map(scale).collect()
}

/// The sum of the widths of the shown columns.
///
/// # Arguments
///
/// * `widths` - The widths in px.
/// * `list` - The hidden list.
///
/// # Returns
///
/// The sum in px.
pub fn shown_width(widths: &[f64], list: &str) -> f64
{
    let shown = widths.iter().enumerate().filter(|(i, _)| !is_hidden(list, i + 1));
    shown.map(|(_, w)| w).sum()
}

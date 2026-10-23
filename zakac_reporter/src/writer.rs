use termcolor::WriteColor;
use zakac_rust_sugars::ret_if;

/// Write text to the given output stream.
///
/// # Arguments
///
/// * `writer` - The writer to write the text to.
/// * `text` - The text to write.
///
/// # Returns
///
/// The result of the write operation.
pub fn write_text<W>(writer: &mut W, text: &str) -> Result<(), std::io::Error>
where
    W: WriteColor + ?Sized,
{
    let result = write!(writer, "{text}");
    ret_if!(result.is_ok(), Ok(()));

    let error = result.unwrap_err();
    ret_if!(error.kind() == std::io::ErrorKind::BrokenPipe, Ok(()));
    Err(error)
}

/// Write a line of text to the given output stream.
///
/// # Arguments
///
/// * `writer` - The writer to write the line to.
/// * `text` - The text to write.
///
/// # Returns
///
/// The result of the write operation.
pub fn write_line<W>(writer: &mut W, text: &str) -> Result<(), std::io::Error>
where
    W: WriteColor + ?Sized,
{
    write_text(writer, &format!("{text}\n"))
}

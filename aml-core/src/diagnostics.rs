use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::error::Rich;
use std::io::Write;

/// Print parse error diagnostics to a writer.
///
/// Formats and outputs parser errors in a human-readable format with source
/// location information. Useful for error reporting in CLIs and error handlers.
///
/// # Arguments
///
/// * `input` - The source text that was parsed
/// * `file` - The filename to display in error messages
/// * `errs` - A collection of parse errors from the parser
/// * `output` - The writer where formatted errors will be written
///
/// # Example
///
/// ```ignore
/// let errs = document_parser.parse(input).errors();
/// report(&input, "style.aml", errs, &mut std::io::stdout())?;
/// ```
pub fn report<W: Write>(
    input: &str,
    file: &str,
    errs: Vec<Rich<'_, char>>,
    output: &mut W,
) -> std::io::Result<()> {
    for err in errs {
        let span = err.span().into_range();

        Report::build(ReportKind::Error, (file, span.clone()))
            .with_message(err.reason().to_string())
            .with_label(
                Label::new((file, span))
                    .with_message(err.reason().to_string())
                    .with_color(Color::Red),
            )
            .finish()
            .write((file, Source::from(input)), &mut *output)?;
    }

    Ok(())
}

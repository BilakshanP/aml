use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::error::Rich;
use std::io::Write;

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

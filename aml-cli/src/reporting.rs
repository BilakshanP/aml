use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::error::Rich;

pub fn report_errors(input: &str, file: &str, errs: Vec<Rich<'_, char>>) {
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
            .eprint((file, Source::from(input)))
            .unwrap();
    }
}

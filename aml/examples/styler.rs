use aml::styler::Style;

fn main() {
    let s = Style::new("fR by mbi").unwrap();

    println!(
        "{}",
        s.paint(" Using a pre-computed style. Use me if the style will be used often. ")
    );

    println!(
        "{}",
        Style::paint_str(
            "f198 bW mu",
            " Using on-the-fly style. Use me if the style is used rarely. "
        )
        .unwrap()
    )
}

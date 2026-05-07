use aml::prelude::*;

fn main() {
    let input = include_str!("markups/shorthand.aml");
    let doc = Document::try_new(input).unwrap();
    print!("{}", doc.render());
}

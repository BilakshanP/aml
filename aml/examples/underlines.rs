use aml::prelude::*;

fn main() {
    let input = include_str!("markups/underlines.aml");
    let doc = Document::new(input);

    print!("{}", doc.render());
}

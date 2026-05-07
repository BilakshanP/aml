// Note: This example might not work correctly in every terminal app.
//
// Properly works on Windows Terminal app.
// Doesn't work on Kitty and Alacritty.

use aml::prelude::*;

fn main() {
    let input = include_str!("markups/xterm.aml");
    let doc = Document::try_new(input).unwrap();
    print!("{}", doc.render());
}

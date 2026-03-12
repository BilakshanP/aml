use aml::{style, styler::CompiledStyle};

const INFO: CompiledStyle = style!("fR");

fn main() {
    println!("{}", INFO.paint("Hiii"))
}

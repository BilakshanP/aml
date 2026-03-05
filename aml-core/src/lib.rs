#[cfg(test)]
mod tests;

pub mod parser;
pub mod render;

#[cfg(feature = "styler")]
pub mod styler;

pub mod prelude {
    pub use crate::parser::Document;
}

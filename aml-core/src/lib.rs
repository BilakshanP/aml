#[cfg(test)]
mod tests;

pub mod parser;
pub mod render;

#[cfg(feature = "styler")]
pub mod styler;

#[cfg(feature = "diagnostics")]
pub mod diagnostics;

pub mod prelude {
    pub use crate::parser::Document;

    #[cfg(feature = "diagnostics")]
    pub use crate::diagnostics::report;
}

use chumsky::prelude::{Parser, Rich};

use crate::{
    parser::{Colour, Modifiers, Tag, shorthand},
    render::{RESET, wrap},
};

/// A parsed style specification.
///
/// Created by parsing a style string like "f#ff0000 b#00ff00 mbi".
/// Use [`Style::paint`] for runtime styling or [`Style::compile`] only
/// in const/proc-macro contexts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Style {
    fg: Option<Colour>,
    bg: Option<Colour>,
    mdf: Option<Modifiers>,
}

impl Style {
    /// Parse a style specification string.
    ///
    /// Format: space-separated foreground color, background color, and/or modifiers.
    /// Examples:
    /// - "f#ff0000" - red foreground
    /// - "b#00ff00 mbi" - green background, bold, italic
    /// - "fR mbu" - red foreground, bold, underline
    ///
    /// Returns an error if the input is invalid.
    pub fn new(spec: &str) -> Result<Self, Vec<Rich<'_, char>>> {
        let shorthand = shorthand().parse(spec).into_result()?;

        match shorthand {
            Tag::Shorthand { fg, bg, mdf } => Ok(Self { fg, bg, mdf }),
            _ => unreachable!(),
        }
    }

    /// Build the SGR code sequence for this style.
    fn codes(&self) -> Vec<u8> {
        let mut parts = Vec::new();
        if let Some(fg) = self.fg {
            parts.extend(fg.fg_codes());
        }
        if let Some(bg) = self.bg {
            parts.extend(bg.bg_codes());
        }
        if let Some(mdf) = &self.mdf {
            parts.extend(mdf.sgr_codes());
        }
        parts
    }

    /// Apply this style to text, returning styled text with a trailing reset.
    ///
    /// This is the preferred runtime method — it does not leak memory.
    ///
    /// # Example
    ///
    /// ```
    /// use aml_core::styler::Style;
    /// let styled = Style::paint_str("fr mbi", "Hello").unwrap();
    /// assert!(styled.contains("Hello"));
    /// ```
    pub fn paint(&self, text: &str) -> String {
        format!("{}{text}{RESET}", wrap(&self.codes()))
    }

    /// Parse a style string and apply it to text in one step.
    ///
    /// # Example
    ///
    /// ```
    /// use aml_core::styler::Style;
    /// let styled = Style::paint_str("fr mbi", "Hello").unwrap();
    /// assert!(styled.contains("Hello"));
    /// ```
    pub fn paint_str<'src>(spec: &'src str, text: &'src str) -> Result<String, Vec<Rich<'src, char>>> {
        Ok(Style::new(spec)?.paint(text))
    }

    /// Compile this style into a `CompiledStyle` holding a `&'static str`.
    ///
    /// **Warning:** This leaks memory. It is intended only for use by the
    /// `style!` proc-macro to produce `const`-compatible values. For runtime
    /// styling, use [`Style::paint`] or [`Style::paint_str`] instead.
    #[doc(hidden)]
    pub fn compile(&self) -> CompiledStyle {
        CompiledStyle(wrap(&self.codes()).leak())
    }

    /// Deprecated: use [`Style::paint_str`] instead.
    #[deprecated(since = "0.1.1", note = "use Style::paint_str instead, Style::apply leaks memory")]
    pub fn apply<'src>(spec: &'src str, text: &'src str) -> Result<String, Vec<Rich<'src, char>>> {
        Style::paint_str(spec, text)
    }
}

/// A compiled ANSI escape code sequence for use in `const` contexts.
///
/// This type holds a `&'static str` and is produced by the `style!` proc-macro
/// at compile time. Do not construct at runtime (it leaks memory); use
/// [`Style::paint`] instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledStyle(pub &'static str);

impl CompiledStyle {
    /// Apply this style to text, returning styled text with a trailing reset.
    pub fn paint(&self, text: &str) -> String {
        format!("{}{text}{RESET}", self.0)
    }
}

impl std::fmt::Display for CompiledStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for CompiledStyle {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        let inner = &self.0;

        tokens.extend(quote::quote! {
            ::aml::styler::CompiledStyle(#inner)
        });
    }
}

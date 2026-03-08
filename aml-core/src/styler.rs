use chumsky::prelude::{Parser, Rich};

use crate::{
    parser::{Colour, Modifiers, Tag, shorthand},
    render::{RESET, wrap},
};

/// A compiled style specification.
///
/// Created by parsing a style string like "f#ff0000 b#00ff00 mbi" and compiled
/// into ANSI escape codes ready to apply to text.
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

    /// Compile this style into ANSI escape codes.
    pub fn compile(&self) -> CompiledStyle {
        let mut parts = Vec::new();

        if let Some(fg) = self.fg {
            parts.extend(fg.fg_codes());
        }

        if let Some(bg) = self.bg {
            parts.extend(bg.bg_codes());
        }

        if let Some(Modifiers(ms)) = &self.mdf {
            let mut mods = ms.iter().map(|m| *m as u8).collect::<Vec<_>>();
            mods.sort();
            parts.extend(mods);
        }

        CompiledStyle(wrap(&parts))
    }

    /// Parse a style string and apply it to text in one step.
    ///
    /// Example:
    /// ```ignore
    /// let styled = Style::apply("f#ff0000 mbi", "Hello")?;
    /// ```
    pub fn apply<'src>(spec: &'src str, text: &'src str) -> Result<String, Vec<Rich<'src, char>>> {
        Ok(Style::new(spec)?.compile().paint(text))
    }
}

/// A compiled ANSI escape code sequence ready to apply to text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledStyle(pub String);

impl CompiledStyle {
    /// Apply this style to text, returning styled text with a trailing reset.
    pub fn paint(&self, text: &str) -> String {
        format!("{}{text}{RESET}", self.0)
    }
}

impl std::fmt::Display for CompiledStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for CompiledStyle {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        let inner = &self.0;

        tokens.extend(quote::quote! {
            ::aml::styler::CompiledStyle(#inner.to_owned())
        });
    }
}

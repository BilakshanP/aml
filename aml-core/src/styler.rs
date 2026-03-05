use chumsky::prelude::{Parser, Rich};

use crate::{
    parser::{Colour, Modifiers, Tag, shorthand},
    render::{RESET, wrap},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Style {
    fg: Option<Colour>,
    bg: Option<Colour>,
    mdf: Option<Modifiers>,
}

impl Style {
    pub fn new(spec: &str) -> Result<Self, Vec<Rich<'_, char>>> {
        let shorthand = shorthand().parse(spec).into_result()?;

        match shorthand {
            Tag::Shorthand { fg, bg, mdf } => Ok(Self { fg, bg, mdf }),
            _ => unreachable!(),
        }
    }

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

    pub fn apply<'src>(spec: &'src str, text: &'src str) -> Result<String, Vec<Rich<'src, char>>> {
        Ok(Style::new(spec)?.compile().paint(text))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledStyle(pub String);

impl CompiledStyle {
    pub fn paint(&self, text: &str) -> String {
        format!("{}{text}{RESET}", self.0)
    }
}

impl std::fmt::Display for CompiledStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

use std::io::Read;

use clap::{Args, Parser};

/// ANSI Markup Language renderer.
#[derive(Parser)]
#[command(name = "aml", version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    pub input: Input,

    /// Apply a shorthand style to input text.
    #[arg(short = 't', long, value_name = "STYLE")]
    pub style: Option<String>,

    /// Emit raw output (debug representation).
    #[arg(short, long)]
    pub raw: bool,

    /// Do not print a trailing newline.
    #[arg(short = 'n', long = "no-newline")]
    pub no_newline: bool,
}

pub struct Info<'src> {
    pub name: &'src str,
    pub input: String,
}

impl<'src> Info<'src> {
    fn new(name: &'src str, input: String) -> Self {
        Info { name, input }
    }
}

#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct Input {
    /// AML markup string to render.
    pub markup: Option<String>,

    /// Read AML markup from stdin.
    #[arg(short, long)]
    pub stdin: bool,

    /// Read AML markup from a file.
    #[arg(short, long, value_name = "FILE")]
    pub file: Option<std::path::PathBuf>,
}

impl Input {
    pub fn read(&self) -> Result<Info<'_>, String> {
        if let Some(s) = &self.markup {
            return Ok(Info::new("stdin", s.clone()));
        }

        if let Some(path) = &self.file {
            let name = path.to_str().expect("Valid unicode file path");

            let content =
                std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;

            return Ok(Info::new(name, content));
        }

        if self.stdin {
            let mut input = String::new();
            std::io::stdin()
                .read_to_string(&mut input)
                .map_err(|e| format!("reading stdin: {e}"))?;
            return Ok(Info::new("stdin", input));
        }

        Err("one of <MARKUP>, --file, or --stdin must be provided".to_string())
    }
}

impl Cli {
    /// Get input from the Input group.
    /// One of markup/file/stdin must be specified.
    pub fn get_input(&self) -> Result<Info<'_>, String> {
        // Check if any input source is specified
        let has_input =
            self.input.markup.is_some() || self.input.file.is_some() || self.input.stdin;

        if !has_input {
            return Err("one of <MARKUP>, --file, or --stdin must be provided".to_string());
        }

        // Use the Input group
        self.input.read()
    }

    /// Apply a style to unstyled input text using the styler feature.
    /// Returns the styled text or the original text if no style is provided.
    pub fn apply_style(&self, input: String) -> Result<String, String> {
        match &self.style {
            Some(style) => {
                use aml::styler::Style;
                let trimmed = input.trim_end_matches('\n');
                Style::apply(style, trimmed)
                    .map_err(|_| format!("Invalid style specification: {}", style))
            }
            None => Ok(input),
        }
    }
}

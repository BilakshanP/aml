use clap::{Args, Parser};

/// ANSI Markup Language renderer.
#[derive(Parser)]
#[command(name = "aml", version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    pub input: Input,

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

        unreachable!("clap ensures one of the two is always set")
    }
}

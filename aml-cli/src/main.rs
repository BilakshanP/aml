mod cli;
mod reporting;

use std::process::ExitCode;

use clap::Parser;

use aml::prelude::*;

use cli::{Cli, Info};
use reporting::report_errors;

fn main() -> ExitCode {
    let cli = Cli::parse();

    let Info { name, input } = match cli.input.read() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    match Document::try_new(&input) {
        Ok(doc) => {
            let rendered = doc.render();

            if cli.raw {
                print!("{rendered:?}");
            } else {
                print!("{rendered}");
            }

            if !cli.no_newline {
                println!();
            }

            ExitCode::SUCCESS
        }
        Err(errs) => {
            report_errors(&input, name, errs);
            ExitCode::FAILURE
        }
    }
}

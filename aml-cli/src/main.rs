mod cli;

use std::process::ExitCode;

use clap::Parser;

use aml::prelude::*;

use cli::Cli;

fn main() -> ExitCode {
    let cli = Cli::parse();

    let info = match cli.get_input() {
        Ok(i) => i,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    // If --style is provided, apply style directly without parsing
    if cli.style.is_some() {
        let styled = match cli.apply_style(info.input) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
        };

        if cli.raw {
            print!("{styled:?}");
        } else {
            print!("{styled}");
        }

        if !cli.no_newline {
            println!();
        }

        return ExitCode::SUCCESS;
    }

    // Otherwise, parse as markup
    match Document::try_new(&info.input) {
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
            report(&info.input, info.name, errs, &mut std::io::stdout()).expect("print to stdout");
            ExitCode::FAILURE
        }
    }
}

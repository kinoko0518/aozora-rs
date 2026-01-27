use std::env;
use std::fs;
use std::path::PathBuf;

use aozora_rs::prelude::{Retokenized, retokenize, scopenize, tokenize};
use miette::{IntoDiagnostic, Result};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: retokenize <file_path>");
        return Ok(());
    }

    let path = PathBuf::from(&args[1]);
    let content = fs::read_to_string(&path).into_diagnostic()?;

    let (_meta, tokens) = tokenize(&content)?;
    let (deco, flat) = scopenize(tokens, &content)?;
    let retokenized = retokenize(flat, deco)?;

    for item in retokenized {
        match item {
            Retokenized::Text(t) => print!("{}", t),
            Retokenized::Break(_) => println!(),
            Retokenized::Odoriji(o) => print!("{}", o),
            Retokenized::Figure(f) => print!("{}", f),
            Retokenized::DecoBegin(d) => print!("{}", d),
            Retokenized::DecoEnd(d) => {
                let s = d.to_string();
                if s.len() > 2 && s.starts_with('[') && s.ends_with(']') {
                    print!("[/{}]", &s[1..s.len() - 1]);
                } else {
                    print!("[/{}]", s);
                }
            }
        }
    }

    Ok(())
}

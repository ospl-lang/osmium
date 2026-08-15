use crate::parse::{PE, Parser};
use std::fmt::Write;

pub fn print_diag(_p: &Parser, e: &PE) {
    eprintln!("DEBUG DIAGNOSTIC\n\n");
    match e {
        PE::Expected(te) => {
            // reconstruct nearby tokens
            let i = te.got.position();
            let reconstruction = _p.tokens.get(i.token_num - 5 .. i.token_num + 5);
            let mut reconstruction_tx = String::new();

            if let Some(reconstruction) = reconstruction {
                for token in reconstruction {
                    let t = token.token();
                    let _ = write!(&mut reconstruction_tx, "{:?} | ", t);
                }
            } else {
                reconstruction_tx = String::from("no reconstruction available...")
            }

            // print the err
            eprintln!(
                "expected: {}\nfound: {:?}\nin: {}\nsurrounding tokens: {}",
                te.expected,
                te.got.token(),
                te.got.position(),
                reconstruction_tx
            );
        },
        PE::EOF => {
            eprintln!("end-of-file.")
        },
    }
}
use crate::parse::{PE, Parser};

pub fn print_diag(_p: &Parser, e: PE) {
    match e {
        PE::Expected(te) => {
            // // reconstruct nearby tokens
            // let i = te.got.position();
            // let reconstruction = p.tokens.get(i.ch .. i.ch + 5);
            // let mut reconstruction_tx = String::new();

            // if let Some(reconstruction) = reconstruction {
            //     for token in reconstruction {
            //         let t = token.token();
            //         let _ = write!(&mut reconstruction_tx, "{:?} | ", t);
            //     }
            // } else {
            //     reconstruction_tx = String::from("no reconstruction available...")
            // }

            // print the err
            println!(
                "expected: {}\nfound: {:?}\nin: {}",
                te.expected,
                te.got.token(),
                te.got.position()
                // reconstruction_tx
            );
        },
        PE::UnexpectedEOF => {
            println!("unexpected end-of-file.")
        },
        PE::RequiredPrimitiveType => {
            println!("a primitive type is required.")
        }
    }
}
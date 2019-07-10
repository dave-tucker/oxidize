use std::fs::File;
use std::io::prelude::*;

use nom::error::convert_error;
use oxidize::parser;

fn main() -> std::io::Result<()> {
    let mut file = File::open("Makefile")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let res = parser::parse_makefile(&contents);
    match res {
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
            println!("{}", convert_error(&contents, e));
            panic!();
        }
        Err(nom::Err::Incomplete(_)) => {
            unreachable!()
        }
        Ok((_, o)) => {
            println!("{:#?}", o);
            Ok(())
        }
    }
}
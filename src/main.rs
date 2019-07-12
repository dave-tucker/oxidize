use std::fs::File;
use std::io::prelude::*;

use clap::{App, Arg};
use nom::error::convert_error;
use oxidize::parser;

fn main() -> std::io::Result<()> {
    let matches = App::new("oxidize")
        .version("0.1.0")
        .author("Dave Tucker <dave@dtucker.co.uk>")
        .about("A fast, flexible, make alternative")
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .takes_value(true)
                .help("A cool file"),
        )
        .get_matches();

    let filename = matches.value_of("file").unwrap_or("Makefile");
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let res = parser::parse_makefile(&contents);
    match res {
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
            println!("{}", convert_error(&contents, e));
            panic!();
        }
        Err(nom::Err::Incomplete(_)) => unreachable!(),
        Ok((_, o)) => {
            println!("{:#?}", o);
            Ok(())
        }
    }
}

extern crate clap;
extern crate env_logger;
extern crate piccolo;
extern crate rustyline;

use clap::{App, Arg};

use piccolo::prelude::*;

use rustyline::error::ReadlineError;
use rustyline::Editor;

use std::path::{Path, PathBuf};

fn main() {
    #[cfg(feature = "pc-debug")]
    env_logger::init();

    let matches = App::new("Piccolo compiler/interpreter")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Zack Hixon <zphixon@gmail.com>")
        .about("Compiles or interprets Piccolo source files")
        .arg(Arg::with_name("src").help("Piccolo source file").index(1))
        .arg(
            Arg::with_name("bin")
                .help("Piccolo binary file")
                .short("b")
                .long("bin")
                .conflicts_with("src")
                .conflicts_with("compile")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("compile")
                .help("Compile <src> into <output>")
                .short("c")
                .long("compile")
                .requires("src")
                .value_name("output")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("string")
                .help("Run string argument")
                .short("e")
                .value_name("string")
                .takes_value(true),
        )
        .get_matches();

    if !matches.is_present("src") && !matches.is_present("bin") && !matches.is_present("string") {
        repl();
    } else {
        if matches.is_present("compile") {
            todo!();
        //let src = PathBuf::from(matches.value_of("src").unwrap());
        //let out = PathBuf::from(matches.value_of("compile").unwrap());
        //if let Err(errors) = piccolo::compile(&src, &out) {
        //    print_errors(errors);
        //}
        } else if matches.is_present("bin") {
            todo!();
        //let src = PathBuf::from(matches.value_of("bin").unwrap());
        //if let Err(errors) = piccolo::run_bin(&src) {
        //    print_errors(errors);
        //}
        } else if matches.is_present("string") {
            let src = matches.value_of("string").unwrap();
            if let Err(errors) = piccolo::interpret(&src) {
                print_errors(errors);
            }
        } else {
            let src = PathBuf::from(matches.value_of("src").unwrap());
            file(&src);
        }
    }
}

fn print_errors(errors: Vec<PiccoloError>) {
    if errors.len() == 1 {
        println!("Error {}", errors[0])
    } else {
        println!("{} Errors:", errors.len());
        for e in errors.iter() {
            println!("    {}", e);
        }
    }
}

fn file(path: &Path) {
    if let Err(errors) = piccolo::do_file(path) {
        print_errors(errors);
    }
}

fn repl() {
    let mut rl = Editor::<()>::new();
    rl.load_history(".piccolo_history")
        .or_else(|_| std::fs::File::create(".piccolo_history").map(|_| ()))
        .unwrap();

    loop {
        match rl.readline("-- ") {
            Ok(line) => {
                rl.add_history_entry(&line);
                rl.save_history(".piccolo_history").unwrap();

                match piccolo::interpret(&line) {
                    Ok(v) => {
                        if v != Constant::Nil {
                            println!("{:?}", v);
                        }
                    }
                    Err(errors) => print_errors(errors),
                }
            }

            Err(ReadlineError::Interrupted) => {}
            _ => break,
        }
    }
}

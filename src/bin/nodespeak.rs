extern crate nodespeak;
extern crate text_io;

use std::env;
use std::fs;
use std::process;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: nodespeak [compile|interpret|[phase]] [path to file]");
        eprintln!("compile: compiles the specified file and outputs the result.");
        eprintln!("interpret: interprets the specified file using the built-in resolver.");
        eprintln!("[phase]: runs compilation of the file up until [phase] of compilation.");
        eprintln!("    phases: parse, structure, resolve, trivialize, specialize");
        process::exit(64);
    }

    let code = match fs::read_to_string(&args[2]) {
        Result::Ok(content) => content,
        Result::Err(_err) => {
            eprintln!("Could not read from {}", &args[2]);
            process::exit(74);
        }
    };
    let source_set = nodespeak::SourceSet::from_raw_string(&args[2], &code);

    println!("\nStarting...");
    let compile_start = Instant::now();
    match args[1].as_ref() {
        "ast" => match nodespeak::to_ast(&source_set) {
            Result::Ok(program) => println!("{:#?}", program),
            Result::Err(err) => {
                eprintln!("{}", err);
                process::exit(101);
            }
        },
        #[cfg(not(feature = "no-vague"))]
        "vague" => match nodespeak::to_vague(&source_set) {
            Result::Ok(program) => println!("{:?}", program),
            Result::Err(err) => {
                eprintln!("{}", err);
                process::exit(101);
            }
        },
        #[cfg(not(feature = "no-resolved"))]
        "resolved" => match nodespeak::to_resolved(&source_set) {
            Result::Ok(program) => println!("{:?}", program),
            Result::Err(err) => {
                eprintln!("{}", err);
                process::exit(101);
            }
        },
        #[cfg(not(feature = "no-trivial"))]
        "trivial" => match nodespeak::to_trivial(&source_set) {
            Result::Ok(program) => println!("{:?}", program),
            Result::Err(err) => {
                eprintln!("{}", err);
                process::exit(101);
            }
        },
        #[cfg(not(feature = "no-llvmir"))]
        "llvmir" => match nodespeak::to_llvmir(&source_set) {
            Result::Ok(program) => println!("{:?}", program),
            Result::Err(err) => {
                eprintln!("{}", err);
                process::exit(101);
            }
        },
        "execute" => {
            unimplemented!()
        }
        _ => {
            eprintln!(
                "Invalid mode '{}', expected compile or a phase.",
                args[1]
            );
            eprintln!("compile: compiles the specified file and outputs the result.");
            eprintln!("[phase]: runs compilation of the file up until [phase] of compilation.");
            eprintln!("    phases: ast, vague, resolved, trivial, llvmir");
            process::exit(64);
        }
    }
    println!(
        "Task completed sucessfully ({}ms.)\n",
        compile_start.elapsed().as_millis()
    );
}

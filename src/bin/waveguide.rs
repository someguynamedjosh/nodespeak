extern crate waveguide;
#[macro_use]
extern crate text_io;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: waveguide [compile|interpret] [path to file]");
        process::exit(64);
    }

    if args[1] != "compile" && args[1] != "interpret" {
        eprintln!("Invalid mode '{}', expected compile or interpret.", args[1]);
        process::exit(64);
    }
    let interpret = args[1] == "interpret";

    let code = match fs::read_to_string(&args[2]) {
        Result::Ok(content) => content,
        Result::Err(_err) => {
            eprintln!("Could not read from {}", &args[2]);
            process::exit(74);
        }
    };
    let source_set = waveguide::SourceSet::from_raw_string(&args[2], &code);

    println!("\nCompiling source...");
    let compile_start = Instant::now();
    let mut program = match waveguide::compile(&source_set) {
        Result::Ok(program) => program.program,
        Result::Err(err) => {
            eprintln!("{}", err);
            process::exit(101);
        }
    };

    if interpret {
        println!(
            "Compilation completed sucessfully ({}ms.)\n",
            compile_start.elapsed().as_millis()
        );
    } else {
        println!("{:#?}", program);
        return;
    }

    let mut inputs = Vec::new();
    let entry_point = program.borrow_scope(program.get_entry_point());
    for input_id in entry_point.borrow_inputs() {
        for (name, id) in entry_point.borrow_symbols() {
            if id == input_id {
                let data_type = program.borrow_variable(input_id.clone()).borrow_data_type();
                println!(
                    "Enter data for input '{}' (data type is {})",
                    name, data_type
                );
                let final_data;
                loop {
                    print!("> ");
                    io::stdout().flush().unwrap();
                    let line: String = read!("{}\n");
                    // TODO: Handle unclosed brackets and such.
                    match waveguide::util::parse_known_data(&line) {
                        Result::Ok(data) => {
                            if data.matches_data_type(data_type) {
                                final_data = data;
                                break;
                            } else {
                                eprintln!("The variable requires data of type {}, but you provided data of an incorrect type.", data_type);
                            }
                        }
                        Result::Err(err) => {
                            eprintln!("An error was encountered while parsing your data:\n{}", err);
                        }
                    }
                }
                inputs.push(final_data);
            }
        }
    }

    println!("\nInterpreting program...");
    let interpret_start = Instant::now();
    let results = waveguide::interpreter::interpret_from_entry_point(&mut program, inputs)
        .expect("Inputs were already checked to be valid.");
    println!(
        "Interpretation complete ({}ms.)\n",
        interpret_start.elapsed().as_millis()
    );

    // Have to reborrow it after program was borrowed as mut.
    let entry_point = program.borrow_scope(program.get_entry_point());
    for (index, output_id) in entry_point.borrow_outputs().iter().enumerate() {
        for (name, id) in entry_point.borrow_symbols() {
            if id == output_id {
                println!("Final value of '{}':", name);
                println!("> {}", results[index]);
            }
        }
    }
}

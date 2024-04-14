use std::path::Path;

use clap::{App, Arg};
use stack::{get_file_contents, input, Executor, Mode};

#[cfg(test)]
mod test;

fn main() {
    let matches = App::new("Stack")
        .version("1.11")
        .author("Stack Programming Community")
        .about("The powerful script language designed with a stack oriented approach for efficient execution. ")
        .arg(Arg::new("script")
            .index(1)
            .value_name("FILE")
            .help("Sets the script file to execution")
            .takes_value(true))
        .arg(Arg::new("debug")
            .short('d')
            .long("debug")
            .help("Enables debug mode"))
        .get_matches();

    if let Some(script) = matches.value_of("script") {
        if matches.is_present("debug") {
            let mut stack = Executor::new(Mode::Debug);
            stack.evaluate_program(match get_file_contents(Path::new(&script.to_string())) {
                Ok(code) => code,
                Err(err) => {
                    println!("Error! {err}");
                    return;
                }
            })
        } else {
            let mut stack = Executor::new(Mode::Script);
            stack.evaluate_program(match get_file_contents(Path::new(&script.to_string())) {
                Ok(code) => code,
                Err(err) => {
                    println!("Error! {err}");
                    return;
                }
            })
        }
    } else {
        // Show a title
        println!("Stack Programming Language");
        let mut executor = Executor::new(Mode::Debug);
        // REPL Execution
        loop {
            let mut code = String::new();
            loop {
                let enter = input("> ");
                code += &format!("{enter}\n");
                if enter.is_empty() {
                    break;
                }
            }

            executor.evaluate_program(code)
        }
    }
}
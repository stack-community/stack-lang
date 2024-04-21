use clap::{App, Arg};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Error, Read, Write};
use std::path::Path;
mod functions;

#[cfg(test)]
mod test;

fn main() {
    let app = App::new("Stack")
        .version("1.12.0")
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
            .help("Enables debug mode"));
    let matches = app.clone().get_matches();

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
        println!("Version {}", { app.get_version().unwrap_or("unknown") });
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

/// Read string of the file
fn get_file_contents(name: &Path) -> Result<String, Error> {
    let mut f = File::open(name)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    Ok(contents)
}

/// Get standard input
fn input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut result = String::new();
    io::stdin().read_line(&mut result).ok();
    result.trim().to_string()
}

/// Execution Mode
#[derive(Clone, Debug)]
enum Mode {
    Script, // Script execution
    Debug,  // Debug execution
}

/// Data type
#[derive(Clone, Debug)]
enum Type {
    Number(f64),
    String(String),
    Bool(bool),
    List(Vec<Type>),
    Object(String, HashMap<String, Type>),
    Error(String),
}

/// Implement methods
impl Type {
    /// Show data to display
    fn display(&self) -> String {
        match self {
            Type::Number(num) => num.to_string(),
            Type::String(s) => format!("({})", s),
            Type::Bool(b) => b.to_string(),
            Type::List(list) => {
                let result: Vec<String> = list.iter().map(|token| token.display()).collect();
                format!("[{}]", result.join(" "))
            }
            Type::Error(err) => format!("error:{err}"),
            Type::Object(name, _) => {
                format!("Object<{name}>")
            }
        }
    }

    /// Get string form data
    fn get_string(&mut self) -> String {
        match self {
            Type::String(s) => s.to_string(),
            Type::Number(i) => i.to_string(),
            Type::Bool(b) => b.to_string(),
            Type::List(l) => Type::List(l.to_owned()).display(),
            Type::Error(err) => format!("error:{err}"),
            Type::Object(name, _) => {
                format!("Object<{name}>")
            }
        }
    }

    /// Get number from data
    fn get_number(&mut self) -> f64 {
        match self {
            Type::String(s) => s.parse().unwrap_or(0.0),
            Type::Number(i) => *i,
            Type::Bool(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            Type::List(l) => l.len() as f64,
            Type::Error(e) => e.parse().unwrap_or(0f64),
            Type::Object(_, object) => object.len() as f64,
        }
    }

    /// Get bool from data
    fn get_bool(&mut self) -> bool {
        match self {
            Type::String(s) => !s.is_empty(),
            Type::Number(i) => *i != 0.0,
            Type::Bool(b) => *b,
            Type::List(l) => !l.is_empty(),
            Type::Error(e) => e.parse().unwrap_or(false),
            Type::Object(_, object) => object.is_empty(),
        }
    }

    /// Get list form data
    fn get_list(&mut self) -> Vec<Type> {
        match self {
            Type::String(s) => s
                .to_string()
                .chars()
                .map(|x| Type::String(x.to_string()))
                .collect::<Vec<Type>>(),
            Type::Number(i) => vec![Type::Number(*i)],
            Type::Bool(b) => vec![Type::Bool(*b)],
            Type::List(l) => l.to_vec(),
            Type::Error(e) => vec![Type::Error(e.to_string())],
            Type::Object(_, object) => object.values().map(|x| x.to_owned()).collect::<Vec<Type>>(),
        }
    }
}

/// Manage program execution
#[derive(Clone, Debug)]
struct Executor {
    stack: Vec<Type>,              // Data stack
    memory: HashMap<String, Type>, // Variable's memory
    mode: Mode,                    // Execution mode
}

impl Executor {
    /// Constructor
    fn new(mode: Mode) -> Executor {
        Executor {
            stack: Vec::new(),
            memory: HashMap::new(),
            mode,
        }
    }

    /// Output log
    fn log_print(&mut self, msg: String) {
        if let Mode::Debug = self.mode {
            print!("{msg}");
        }
    }

    /// Show variable inside memory
    fn show_variables(&mut self) {
        self.log_print("Variables {\n".to_string());
        let max = self.memory.keys().map(|s| s.len()).max().unwrap_or(0);
        for (name, value) in self.memory.clone() {
            self.log_print(format!(
                " {:>width$}: {}\n",
                name,
                value.display(),
                width = max
            ))
        }
        self.log_print("}\n".to_string())
    }

    /// Show inside the stack
    fn show_stack(&mut self) -> String {
        format!(
            "Stack〔 {} 〕",
            self.stack
                .iter()
                .map(|x| x.display())
                .collect::<Vec<_>>()
                .join(" | ")
        )
    }

    /// Parse token by analyzing syntax
    fn analyze_syntax(&mut self, code: String) -> Vec<String> {
        // Convert tabs, line breaks, and full-width spaces to half-width spaces
        let code = code.replace(['\n', '\t', '\r', '　'], " ");

        let mut syntax = Vec::new(); // Token string
        let mut buffer = String::new(); // Temporary storage
        let mut brackets = 0; // String's nest structure
        let mut parentheses = 0; // List's nest structure
        let mut hash = false; // Is it Comment
        let mut escape = false; // Flag to indicate next character is escaped

        for c in code.chars() {
            match c {
                '\\' if !escape => {
                    escape = true;
                }
                '(' if !hash && !escape => {
                    brackets += 1;
                    buffer.push('(');
                }
                ')' if !hash && !escape => {
                    brackets -= 1;
                    buffer.push(')');
                }
                '#' if !hash && !escape => {
                    hash = true;
                    buffer.push('#');
                }
                '#' if hash && !escape => {
                    hash = false;
                    buffer.push('#');
                }
                '[' if !hash && brackets == 0 && !escape => {
                    parentheses += 1;
                    buffer.push('[');
                }
                ']' if !hash && brackets == 0 && !escape => {
                    parentheses -= 1;
                    buffer.push(']');
                }
                ' ' if !hash && parentheses == 0 && brackets == 0 && !escape => {
                    if !buffer.is_empty() {
                        syntax.push(buffer.clone());
                        buffer.clear();
                    }
                }
                _ => {
                    if parentheses == 0 && brackets == 0 && !hash {
                        if escape {
                            match c {
                                'n' => buffer.push_str("\\n"),
                                't' => buffer.push_str("\\t"),
                                'r' => buffer.push_str("\\r"),
                                _ => buffer.push(c),
                            }
                        } else {
                            buffer.push(c);
                        }
                    } else {
                        if escape {
                            buffer.push('\\');
                        }
                        buffer.push(c);
                    }
                    escape = false; // Reset escape flag for non-escape characters
                }
            }
        }

        if !buffer.is_empty() {
            syntax.push(buffer);
        }
        syntax
    }

    /// evaluate string as program
    fn evaluate_program(&mut self, code: String) {
        // Parse into token string
        let syntax: Vec<String> = self.analyze_syntax(code);

        for token in syntax {
            // Show inside stack to debug
            let stack = self.show_stack();
            self.log_print(format!("{stack} ←  {token}\n"));

            // Character vector for token processing
            let chars: Vec<char> = token.chars().collect();

            // Judge what the token is
            if let Ok(i) = token.parse::<f64>() {
                // Push number value on the stack
                self.stack.push(Type::Number(i));
            } else if token == "true" || token == "false" {
                // Push bool value on the stack
                self.stack.push(Type::Bool(token.parse().unwrap_or(true)));
            } else if chars[0] == '(' && chars[chars.len() - 1] == ')' {
                // Processing string escape
                let string = {
                    let mut buffer = String::new(); // Temporary storage
                    let mut brackets = 0; // String's nest structure
                    let mut parentheses = 0; // List's nest structure
                    let mut hash = false; // Is it Comment
                    let mut escape = false; // Flag to indicate next character is escaped

                    for c in token[1..token.len() - 1].to_string().chars() {
                        match c {
                            '\\' if !escape => {
                                escape = true;
                            }
                            '(' if !hash && !escape => {
                                brackets += 1;
                                buffer.push('(');
                            }
                            ')' if !hash && !escape => {
                                brackets -= 1;
                                buffer.push(')');
                            }
                            '#' if !hash && !escape => {
                                hash = true;
                                buffer.push('#');
                            }
                            '#' if hash && !escape => {
                                hash = false;
                                buffer.push('#');
                            }
                            '[' if !hash && brackets == 0 && !escape => {
                                parentheses += 1;
                                buffer.push('[');
                            }
                            ']' if !hash && brackets == 0 && !escape => {
                                parentheses -= 1;
                                buffer.push(']');
                            }
                            _ => {
                                if parentheses == 0 && brackets == 0 && !hash {
                                    if escape {
                                        match c {
                                            'n' => buffer.push_str("\\n"),
                                            't' => buffer.push_str("\\t"),
                                            'r' => buffer.push_str("\\r"),
                                            _ => buffer.push(c),
                                        }
                                    } else {
                                        buffer.push(c);
                                    }
                                } else {
                                    if escape {
                                        buffer.push('\\');
                                    }
                                    buffer.push(c);
                                }
                                escape = false; // Reset escape flag for non-escape characters
                            }
                        }
                    }
                    buffer
                }; // Push string value on the stack
                self.stack.push(Type::String(string));
            } else if chars[0] == '[' && chars[chars.len() - 1] == ']' {
                // Push list value on the stack
                let old_len = self.stack.len(); // length of old stack
                let slice = &token[1..token.len() - 1];
                self.evaluate_program(slice.to_string());
                // Make increment of stack an element of list
                let mut list = Vec::new();
                for _ in old_len..self.stack.len() {
                    list.push(self.pop_stack());
                }
                list.reverse(); // reverse list
                self.stack.push(Type::List(list));
            } else if token.starts_with("error:") {
                // Push error value on the stack
                self.stack.push(Type::Error(token.replace("error:", "")))
            } else if let Some(i) = self.memory.get(&token) {
                // Push variable's data on stack
                self.stack.push(i.clone());
            } else if chars[0] == '#' && chars[chars.len() - 1] == '#' {
                // Processing comments
                self.log_print(format!("* Comment \"{}\"\n", token.replace('#', "")));
            } else {
                // Else, execute as command
                self.execute_command(token);
            }
        }

        // Show inside stack, after execution
        let stack = self.show_stack();
        self.log_print(format!("{stack}\n"));
    }

    /// execute string as commands
    fn execute_command(&mut self, command: String) {
        functions::execute_command(self, command);
    }

    /// Pop stack's top value
    fn pop_stack(&mut self) -> Type {
        if let Some(value) = self.stack.pop() {
            value
        } else {
            self.log_print(
                "Error! There are not enough values on the stack. returns default value\n"
                    .to_string(),
            );
            Type::String("".to_string())
        }
    }
}

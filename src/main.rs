use rand::seq::SliceRandom;
use regex::Regex;
use rodio::{OutputStream, Sink, Source};
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{self, Error, Read, Write};
use std::path::Path;
use std::thread::{self, sleep};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(test)]
mod test;

fn main() {
    // Reading command line arguments
    let args = env::args().collect::<Vec<_>>();
    if args.len() > 2 {
        // Open the script file
        match get_file_contents(args[1].clone()) {
            Ok(code) => {
                // Judge execution mode
                if args[2].contains("-d") {
                    let mut executor = Executor::new(Mode::Debug);
                    executor.evaluate_program(code); // Debug execution
                } else {
                    let mut executor = Executor::new(Mode::Script);
                    executor.evaluate_program(code); // Script execution
                }
            }
            Err(e) => println!("Error! {e}"),
        }
    } else if args.len() > 1 {
        // Open the script file
        match get_file_contents(args[1].clone()) {
            Ok(code) => {
                let mut executor = Executor::new(Mode::Script);
                executor.evaluate_program(code); // Default is script execution
            }
            Err(e) => println!("Error! {e}"),
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

/// Read string of the file
fn get_file_contents(name: String) -> Result<String, Error> {
    let mut f = File::open(name.trim())?;
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
                let syntax: Vec<String> = list.iter().map(|token| token.display()).collect();
                format!("[{}]", syntax.join(" "))
            }
            Type::Error(err) => format!("error:{err}"),
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
            Type::Error(_) => 0f64,
        }
    }

    /// Get bool from data
    fn get_bool(&mut self) -> bool {
        match self {
            Type::String(s) => !s.is_empty(),
            Type::Number(i) => *i != 0.0,
            Type::Bool(b) => *b,
            Type::List(l) => !l.is_empty(),
            Type::Error(_) => false,
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
    fn show_stack(&mut self) {
        self.log_print(format!(
            "Stack〔 {} 〕",
            self.stack
                .iter()
                .map(|x| x.display())
                .collect::<Vec<_>>()
                .join(" | ")
        ))
    }

    /// Parse token by analyzing syntax
    fn analyze_syntax(&mut self, code: String) -> Vec<String> {
        // Convert tabs, line breaks, and full-width spaces to half-width spaces
        let code = code.replace(['\n', '\t', '\r', '　'], " ");

        let mut syntax = Vec::new(); // Token string
        let mut buffer = String::new(); // Temporary storage
        let mut in_brackets = 0; // String's nest structure
        let mut in_parentheses = 0; // List's nest structure
        let mut in_hash = false; // Is it Comment

        for c in code.chars() {
            match c {
                '(' => {
                    in_brackets += 1;
                    buffer.push('(');
                }
                ')' => {
                    in_brackets -= 1;
                    buffer.push(')');
                }
                '#' if !in_hash => {
                    in_hash = true;
                    buffer.push('#');
                }
                '#' if in_hash => {
                    in_hash = false;
                    buffer.push('#');
                }
                '[' if in_brackets == 0 => {
                    in_parentheses += 1;
                    buffer.push('[');
                }
                ']' if in_brackets == 0 => {
                    in_parentheses -= 1;
                    buffer.push(']');
                }
                ' ' if !in_hash && in_parentheses == 0 && in_brackets == 0 => {
                    if !buffer.is_empty() {
                        syntax.push(buffer.clone());
                        buffer.clear();
                    }
                }
                _ => {
                    buffer.push(c);
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
            self.show_stack(); // Show inside stack to debug
            self.log_print(format!(" ←  {}\n", token));

            // Character vector for token processing
            let chars: Vec<char> = token.chars().collect();

            // Judge what the token is
            if let Ok(i) = token.parse::<f64>() {
                // Push number on stack
                self.stack.push(Type::Number(i));
            } else if token == "true" || token == "false" {
                // Push bool on stack
                self.stack.push(Type::Bool(token.parse().unwrap_or(true)));
            } else if chars[0] == '(' && chars[chars.len() - 1] == ')' {
                // Push string on stack
                self.stack
                    .push(Type::String(token[1..token.len() - 1].to_string()));
            } else if chars[0] == '[' && chars[chars.len() - 1] == ']' {
                // Push list on stack
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
            } else if let Some(i) = self.memory.get(&token) {
                // Push variable's data on stack
                self.stack.push(i.clone());
            } else if token.contains('#') {
                // Processing comments
                self.log_print(format!("* Comment \"{}\"\n", token.replace('#', "")));
            } else {
                // Else, execute as command
                self.execute_command(token);
            }
        }

        // Show inside stack, after execution
        self.show_stack();
        self.log_print("\n".to_string());
    }

    /// execute string as commands
    fn execute_command(&mut self, command: String) {
        match command.as_str() {
            // Commands of calculation

            // addition
            "add" => {
                let b = self.pop_stack().get_number();
                let a = self.pop_stack().get_number();
                self.stack.push(Type::Number(a + b));
            }

            // Subtraction
            "sub" => {
                let b = self.pop_stack().get_number();
                let a = self.pop_stack().get_number();
                self.stack.push(Type::Number(a - b));
            }

            // Multiplication
            "mul" => {
                let b = self.pop_stack().get_number();
                let a = self.pop_stack().get_number();
                self.stack.push(Type::Number(a * b));
            }

            // Division
            "div" => {
                let b = self.pop_stack().get_number();
                let a = self.pop_stack().get_number();
                self.stack.push(Type::Number(a / b));
            }

            // Remainder of division
            "mod" => {
                let b = self.pop_stack().get_number();
                let a = self.pop_stack().get_number();
                self.stack.push(Type::Number(a % b));
            }

            // Exponentiation
            "pow" => {
                let b = self.pop_stack().get_number();
                let a = self.pop_stack().get_number();
                self.stack.push(Type::Number(a.powf(b)));
            }

            // Rounding off
            "round" => {
                let a = self.pop_stack().get_number();
                self.stack.push(Type::Number(a.round()));
            }

            // Trigonometric sine
            "sin" => {
                let number = self.pop_stack().get_number();
                self.stack.push(Type::Number(number.sin()))
            }

            // Trigonometric cosine
            "cos" => {
                let number = self.pop_stack().get_number();
                self.stack.push(Type::Number(number.cos()))
            }

            // Trigonometric tangent
            "tan" => {
                let number = self.pop_stack().get_number();
                self.stack.push(Type::Number(number.tan()))
            }

            // Logical operations of AND
            "and" => {
                let b = self.pop_stack().get_bool();
                let a = self.pop_stack().get_bool();
                self.stack.push(Type::Bool(a && b));
            }

            // Logical operations of OR
            "or" => {
                let b = self.pop_stack().get_bool();
                let a = self.pop_stack().get_bool();
                self.stack.push(Type::Bool(a || b));
            }

            // Logical operations of NOT
            "not" => {
                let b = self.pop_stack().get_bool();
                self.stack.push(Type::Bool(!b));
            }

            // Is it equal
            "equal" => {
                let b = self.pop_stack().get_string();
                let a = self.pop_stack().get_string();
                self.stack.push(Type::Bool(a == b));
            }

            // Is it less
            "less" => {
                let b = self.pop_stack().get_number();
                let a = self.pop_stack().get_number();
                self.stack.push(Type::Bool(a < b));
            }

            // Get random value from list
            "rand" => {
                let list = self.pop_stack().get_list();
                let result = match list.choose(&mut rand::thread_rng()) {
                    Some(i) => i.to_owned(),
                    None => Type::List(list),
                };
                self.stack.push(result);
            }

            // Shuffle list by random
            "shuffle" => {
                let mut list = self.pop_stack().get_list();
                list.shuffle(&mut rand::thread_rng());
                self.stack.push(Type::List(list));
            }

            // Commands of string processing

            // Repeat string a number of times
            "repeat" => {
                let count = self.pop_stack().get_number(); // 回数
                let text = self.pop_stack().get_string(); // 文字列
                self.stack.push(Type::String(text.repeat(count as usize)));
            }

            // Get unicode character form number
            "decode" => {
                let code = self.pop_stack().get_number();
                let result = char::from_u32(code as u32);
                match result {
                    Some(c) => self.stack.push(Type::String(c.to_string())),
                    None => {
                        self.log_print("Error! failed of number decoding\n".to_string());
                        self.stack.push(Type::Error("number-decoding".to_string()));
                    }
                }
            }

            // Encode string by UTF-8
            "encode" => {
                let string = self.pop_stack().get_string();
                if let Some(first_char) = string.chars().next() {
                    self.stack.push(Type::Number((first_char as u32) as f64));
                } else {
                    self.log_print("Error! failed of string encoding\n".to_string());
                    self.stack.push(Type::Error("string-encoding".to_string()));
                }
            }

            // Concatenate the string
            "concat" => {
                let b = self.pop_stack().get_string();
                let a = self.pop_stack().get_string();
                self.stack.push(Type::String(a + &b));
            }

            // Replacing string
            "replace" => {
                let after = self.pop_stack().get_string();
                let before = self.pop_stack().get_string();
                let text = self.pop_stack().get_string();
                self.stack.push(Type::String(text.replace(&before, &after)))
            }

            // split string by key
            "split" => {
                let key = self.pop_stack().get_string();
                let text = self.pop_stack().get_string();
                self.stack.push(Type::List(
                    text.split(&key)
                        .map(|x| Type::String(x.to_string()))
                        .collect::<Vec<Type>>(),
                ));
            }

            // Generate a string by concat list
            "join" => {
                let key = self.pop_stack().get_string();
                let mut list = self.pop_stack().get_list();
                self.stack.push(Type::String(
                    list.iter_mut()
                        .map(|x| x.get_string())
                        .collect::<Vec<String>>()
                        .join(&key),
                ))
            }

            // Is it finding in string
            "find" => {
                let word = self.pop_stack().get_string();
                let text = self.pop_stack().get_string();
                self.stack.push(Type::Bool(text.contains(&word)))
            }

            // Search by regular expression
            "regex" => {
                let pattern = self.pop_stack().get_string();
                let text = self.pop_stack().get_string();

                let pattern: Regex = match Regex::new(pattern.as_str()) {
                    Ok(i) => i,
                    Err(e) => {
                        self.log_print(format!("Error! {}\n", e.to_string().replace("Error", "")));
                        self.stack.push(Type::Error("regex".to_string()));
                        return;
                    }
                };

                let mut list: Vec<Type> = Vec::new();
                for i in pattern.captures_iter(text.as_str()) {
                    list.push(Type::String(i[0].to_string()))
                }
                self.stack.push(Type::List(list));
            }

            // Commands of I/O

            // Write string in the file
            "write-file" => {
                let mut file = match File::create(self.pop_stack().get_string()) {
                    Ok(file) => file,
                    Err(e) => {
                        self.log_print(format!("Error! {e}\n"));
                        self.stack.push(Type::Error("create-file".to_string()));
                        return;
                    }
                };
                if let Err(e) = file.write_all(self.pop_stack().get_string().as_bytes()) {
                    self.log_print(format!("Error! {}\n", e));
                    self.stack.push(Type::Error("write-file".to_string()));
                }
            }

            // Read string in the file
            "read-file" => {
                let name = self.pop_stack().get_string();
                match get_file_contents(name) {
                    Ok(s) => self.stack.push(Type::String(s)),
                    Err(e) => {
                        self.log_print(format!("Error! {}\n", e));
                        self.stack.push(Type::Error("read-file".to_string()));
                    }
                };
            }

            // Standard input
            "input" => {
                let prompt = self.pop_stack().get_string();
                self.stack.push(Type::String(input(prompt.as_str())));
            }

            // Standard output
            "print" => {
                let a = self.pop_stack().get_string();
                if let Mode::Debug = self.mode {
                    println!("[Output]: {a}");
                } else {
                    println!("{a}");
                }
            }

            // Get command-line arguments
            "args-cmd" => self.stack.push(Type::List(
                env::args()
                    .collect::<Vec<_>>()
                    .iter()
                    .map(|x| Type::String(x.to_string()))
                    .collect::<Vec<Type>>(),
            )),

            // Play sound from frequency
            "play-sound" => {
                fn play_sine_wave(frequency: f64, duration_secs: f64) {
                    let sample_rate = 44100f64;

                    let num_samples = (duration_secs * sample_rate) as usize;
                    let samples: Vec<f32> = (0..num_samples)
                        .map(|t| {
                            let t = t as f64 / sample_rate;
                            (t * frequency * 2.0 * std::f64::consts::PI).sin() as f32
                        })
                        .collect();

                    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
                    let sink = Sink::try_new(&stream_handle).unwrap();

                    for _ in samples {
                        sink.append(
                            rodio::source::SineWave::new(frequency as f32)
                                .take_duration(Duration::from_secs_f64(duration_secs)),
                        );
                    }

                    sink.play();
                    std::thread::sleep(Duration::from_secs_f64(duration_secs));
                }

                let duration_secs = self.pop_stack().get_number();
                let frequency = self.pop_stack().get_number();

                play_sine_wave(frequency, duration_secs);
            }

            // Commands of control

            // evaluate string as program
            "eval" => {
                let code = self.pop_stack().get_string();
                self.evaluate_program(code)
            }

            // Conditional branch
            "if" => {
                let condition = self.pop_stack().get_bool(); // condition
                let code_else = self.pop_stack().get_string(); // else code
                let code_if = self.pop_stack().get_string(); // if code
                if condition {
                    self.evaluate_program(code_if)
                } else {
                    self.evaluate_program(code_else)
                };
            }

            // Loop while condition is true
            "while" => {
                let cond = self.pop_stack().get_string();
                let code = self.pop_stack().get_string();
                while {
                    self.evaluate_program(cond.clone());
                    self.pop_stack().get_bool()
                } {
                    self.evaluate_program(code.clone());
                }
            }

            // Generate a thread
            "thread" => {
                let code = self.pop_stack().get_string();
                let mut executor = self.clone();
                thread::spawn(move || executor.evaluate_program(code));
            }

            // exit a process
            "exit" => {
                let status = self.pop_stack().get_number();
                std::process::exit(status as i32);
            }

            // Commands of list processing

            // Get list value by index
            "get" => {
                let index = self.pop_stack().get_number() as usize;
                let list: Vec<Type> = self.pop_stack().get_list();
                if list.len() > index {
                    self.stack.push(list[index].clone());
                } else {
                    self.log_print("Error! Index specification is out of range\n".to_string());
                    self.stack.push(Type::Error("index-out-range".to_string()));
                }
            }

            // Set list value by index
            "set" => {
                let value = self.pop_stack();
                let index = self.pop_stack().get_number() as usize;
                let mut list: Vec<Type> = self.pop_stack().get_list();
                if list.len() > index {
                    list[index] = value;
                    self.stack.push(Type::List(list));
                } else {
                    self.log_print("Error! Index specification is out of range\n".to_string());
                    self.stack.push(Type::Error("index-out-range".to_string()));
                }
            }

            // Delete list value by index
            "del" => {
                let index = self.pop_stack().get_number() as usize;
                let mut list = self.pop_stack().get_list();
                if list.len() > index {
                    list.remove(index);
                    self.stack.push(Type::List(list));
                } else {
                    self.log_print("Error! Index specification is out of range\n".to_string());
                    self.stack.push(Type::Error("index-out-range".to_string()));
                }
            }

            // Append value in the list
            "append" => {
                let data = self.pop_stack();
                let mut list = self.pop_stack().get_list();
                list.push(data);
                self.stack.push(Type::List(list));
            }

            // Insert value in the list
            "insert" => {
                let data = self.pop_stack();
                let index = self.pop_stack().get_number();
                let mut list = self.pop_stack().get_list();
                list.insert(index as usize, data);
                self.stack.push(Type::List(list));
            }

            // Sorting in the list
            "sort" => {
                let mut list: Vec<String> = self
                    .pop_stack()
                    .get_list()
                    .iter()
                    .map(|x| x.to_owned().get_string())
                    .collect();
                list.sort();
                self.stack.push(Type::List(
                    list.iter()
                        .map(|x| Type::String(x.to_string()))
                        .collect::<Vec<_>>(),
                ));
            }

            // reverse in the list
            "reverse" => {
                let mut list = self.pop_stack().get_list();
                list.reverse();
                self.stack.push(Type::List(list));
            }

            // Iteration
            "for" => {
                let code = self.pop_stack().get_string();
                let vars = self.pop_stack().get_string();
                let list = self.pop_stack().get_list();

                list.iter().for_each(|x| {
                    self.memory
                        .entry(vars.clone())
                        .and_modify(|value| *value = x.clone())
                        .or_insert(x.clone());
                    self.evaluate_program(code.clone());
                });
            }

            // Mapping a list
            "map" => {
                let code = self.pop_stack().get_string();
                let vars = self.pop_stack().get_string();
                let list = self.pop_stack().get_list();

                let mut result_list = Vec::new();
                for x in list.iter() {
                    self.memory
                        .entry(vars.clone())
                        .and_modify(|value| *value = x.clone())
                        .or_insert(x.clone());

                    self.evaluate_program(code.clone());
                    result_list.push(self.pop_stack());
                }

                self.stack.push(Type::List(result_list));
            }

            // Filtering a list value
            "filter" => {
                let code = self.pop_stack().get_string();
                let vars = self.pop_stack().get_string();
                let list = self.pop_stack().get_list();

                let mut result_list = Vec::new();

                for x in list.iter() {
                    self.memory
                        .entry(vars.clone())
                        .and_modify(|value| *value = x.clone())
                        .or_insert(x.clone());

                    self.evaluate_program(code.clone());
                    if self.pop_stack().get_bool() {
                        result_list.push(x.clone());
                    }
                }

                self.stack.push(Type::List(result_list));
            }

            // Generate value from list
            "reduce" => {
                let code = self.pop_stack().get_string();
                let now = self.pop_stack().get_string();
                let acc = self.pop_stack().get_string();
                let list = self.pop_stack().get_list();

                self.memory
                    .entry(acc.clone())
                    .and_modify(|value| *value = Type::String("".to_string()))
                    .or_insert(Type::String("".to_string()));

                for x in list.iter() {
                    self.memory
                        .entry(now.clone())
                        .and_modify(|value| *value = x.clone())
                        .or_insert(x.clone());

                    self.evaluate_program(code.clone());
                    let result = self.pop_stack();

                    self.memory
                        .entry(acc.clone())
                        .and_modify(|value| *value = result.clone())
                        .or_insert(result);
                }

                let result = self.memory.get(&acc);
                self.stack
                    .push(result.unwrap_or(&Type::String("".to_string())).clone());

                self.memory
                    .entry(acc.clone())
                    .and_modify(|value| *value = Type::String("".to_string()))
                    .or_insert(Type::String("".to_string()));
            }

            // Generate a range
            "range" => {
                let step = self.pop_stack().get_number();
                let max = self.pop_stack().get_number();
                let min = self.pop_stack().get_number();

                let mut range: Vec<Type> = Vec::new();

                for i in (min as usize..max as usize).step_by(step as usize) {
                    range.push(Type::Number(i as f64));
                }

                self.stack.push(Type::List(range));
            }

            // Get length of list
            "len" => {
                let data = self.pop_stack().get_list();
                self.stack.push(Type::Number(data.len() as f64));
            }

            // Commands of memory manage

            // pop in the stack
            "pop" => {
                self.pop_stack();
            }

            // Get size of stack
            "size-stack" => {
                let len: f64 = self.stack.len() as f64;
                self.stack.push(Type::Number(len));
            }

            // Define variable at memory
            "var" => {
                let name = self.pop_stack().get_string();
                let data = self.pop_stack();
                self.memory
                    .entry(name)
                    .and_modify(|value| *value = data.clone())
                    .or_insert(data);
                self.show_variables()
            }

            // Get data type of value
            "type" => {
                let result = match self.pop_stack() {
                    Type::Number(_) => "number",
                    Type::String(_) => "string",
                    Type::Bool(_) => "bool",
                    Type::List(_) => "list",
                    Type::Error(_) => "error",
                }
                .to_string();
                self.stack.push(Type::String(result));
            }

            // Explicit data type casting
            "cast" => {
                let types = self.pop_stack().get_string();
                let mut value = self.pop_stack();
                match types.as_str() {
                    "number" => self.stack.push(Type::Number(value.get_number())),
                    "string" => self.stack.push(Type::String(value.get_string())),
                    "bool" => self.stack.push(Type::Bool(value.get_bool())),
                    "list" => self.stack.push(Type::List(value.get_list())),
                    _ => self.stack.push(value),
                }
            }

            // Is string include only number
            "only-number" => match self.pop_stack().get_string().trim().parse::<f64>() {
                Ok(_) => self.stack.push(Type::Bool(true)),
                Err(_) => self.stack.push(Type::Bool(false)),
            },

            // Get memory information
            "mem" => {
                let mut list: Vec<Type> = Vec::new();
                for (name, _) in self.memory.clone() {
                    list.push(Type::String(name))
                }
                self.stack.push(Type::List(list))
            }

            // Free up memory space of variable
            "free" => {
                let name = self.pop_stack().get_string();
                self.memory.remove(name.as_str());
                self.show_variables();
            }

            // Copy stack's top value
            "copy" => {
                let data = self.pop_stack();
                self.stack.push(data.clone());
                self.stack.push(data);
            }

            // Swap stack's top 2 value
            "swap" => {
                let b = self.pop_stack();
                let a = self.pop_stack();
                self.stack.push(b);
                self.stack.push(a);
            }

            // Commands of times

            // Get now time as unix epoch
            "now-time" => {
                self.stack.push(Type::Number(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64(),
                ));
            }

            // Sleep fixed time
            "sleep" => sleep(Duration::from_secs_f64(self.pop_stack().get_number())),

            // Command of external cooperation processing

            // Send the http request
            "request" => {
                let url = self.pop_stack().get_string();
                self.stack.push(Type::String(
                    reqwest::blocking::get(url).unwrap().text().unwrap(),
                ));
            }

            // Open the file or url
            "open" => {
                let name = self.pop_stack().get_string();
                if let Err(e) = opener::open(name.clone()) {
                    self.log_print(format!("Error! {e}\n"));
                    self.stack.push(Type::Error("open".to_string()));
                } else {
                    self.stack.push(Type::String(name))
                }
            }

            // Change current directory
            "cd" => {
                let name = self.pop_stack().get_string();
                if let Err(err) = std::env::set_current_dir(name.clone()) {
                    self.log_print(format!("Error! {}\n", err));
                    self.stack.push(Type::Error("cd".to_string()));
                } else {
                    self.stack.push(Type::String(name))
                }
            }

            // Get current directory
            "pwd" => {
                if let Ok(current_dir) = std::env::current_dir() {
                    if let Some(path) = current_dir.to_str() {
                        self.stack.push(Type::String(String::from(path)));
                    }
                }
            }

            // Make directory
            "mkdir" => {
                let name = self.pop_stack().get_string();
                if let Err(e) = fs::create_dir(name.clone()) {
                    self.log_print(format!("Error! {e}\n"));
                    self.stack.push(Type::Error("mkdir".to_string()));
                } else {
                    self.stack.push(Type::String(name))
                }
            }

            // Remove item
            "rm" => {
                let name = self.pop_stack().get_string();
                if Path::new(name.as_str()).is_dir() {
                    if let Err(e) = fs::remove_dir(name.clone()) {
                        self.log_print(format!("Error! {e}\n"));
                        self.stack.push(Type::Error("rm".to_string()));
                    } else {
                        self.stack.push(Type::String(name))
                    }
                } else if let Err(e) = fs::remove_file(name.clone()) {
                    self.log_print(format!("Error! {e}\n"));
                    self.stack.push(Type::Error("rm".to_string()));
                } else {
                    self.stack.push(Type::String(name))
                }
            }

            // Rename item
            "rename" => {
                let to = self.pop_stack().get_string();
                let from = self.pop_stack().get_string();
                if let Err(e) = fs::rename(from, to.clone()) {
                    self.log_print(format!("Error! {e}\n"));
                    self.stack.push(Type::Error("rename".to_string()));
                } else {
                    self.stack.push(Type::String(to))
                }
            }

            // Get list of files
            "ls" => {
                if let Ok(entries) = fs::read_dir(".") {
                    let value: Vec<Type> = entries
                        .filter_map(|entry| {
                            entry
                                .ok()
                                .and_then(|e| e.file_name().into_string().ok())
                                .map(Type::String)
                        })
                        .collect();
                    self.stack.push(Type::List(value));
                }
            }

            // Judge is it folder
            "folder" => {
                let path = self.pop_stack().get_string();
                let path = Path::new(path.as_str());
                self.stack.push(Type::Bool(path.is_dir()));
            }

            // If it is not recognized as a command, use it as a string.
            _ => self.stack.push(Type::String(command)),
        }
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

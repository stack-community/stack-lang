use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::io::{Error, Read};

/// ファイルを読み込む
fn get_file_contents(name: String) -> Result<String, Error> {
    let mut f = File::open(name.trim())?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn input(prompt: &str) -> String {
    print!("{}", prompt.to_string());
    io::stdout().flush().unwrap();
    let mut result = String::new();
    io::stdin().read_line(&mut result).ok();
    return result.trim().parse().ok().unwrap();
}

#[derive(Clone, Debug)]
enum Mode {
    Script,
    Debug,
}

#[derive(Clone, Debug)]
enum Type {
    Number(f64),
    String(String),
    Bool(bool),
}

impl Type {
    fn get_string(&mut self) -> String {
        match self {
            Type::String(s) => s.to_string(),
            Type::Number(i) => i.to_string(),
            Type::Bool(b) => b.to_string(),
        }
    }

    fn get_number(&mut self) -> f64 {
        match self {
            Type::String(s) => s.parse().expect("ふえぇ、変換できないよお"),
            Type::Number(i) => *i,
            Type::Bool(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }

    fn get_bool(&mut self) -> bool {
        match self {
            Type::String(s) => s.len() != 0,
            Type::Number(i) => *i != 0.0,
            Type::Bool(b) => *b,
        }
    }
}

struct Executor {
    stack: Vec<Type>,
    memory: HashMap<String, Type>,
    mode: Mode,
}

impl Executor {
    fn new(mode: Mode) -> Executor {
        Executor {
            stack: Vec::new(),
            memory: HashMap::new(),
            mode,
        }
    }

    fn log_print(&mut self, msg: String) {
        if let Mode::Debug = self.mode {
            println!("{msg}");
        }
    }

    fn execute(&mut self, code: String) {
        let token: Vec<String> = {
            let mut elements = Vec::new();
            let mut buffer = String::new();
            let mut in_brackets = 0;

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
                    ' ' | '　' if in_brackets == 0 => {
                        if !buffer.is_empty() {
                            elements.push(buffer.clone());
                            buffer.clear();
                        }
                    }
                    _ => {
                        buffer.push(c);
                    }
                }
            }

            if !buffer.is_empty() {
                elements.push(buffer);
            }
            elements
        };

        for item in token {
            self.log_print(format!("| Stack {:?} ←  {:?}", self.stack, item));

            if let Ok(i) = item.parse::<f64>() {
                self.stack.push(Type::Number(i));
                continue;
            }

            if item == "true" {
                self.stack.push(Type::Bool(true));
                continue;
            }

            if item == "false" {
                self.stack.push(Type::Bool(false));
                continue;
            }

            if item.contains("(") || item.contains(')') {
                self.stack
                    .push(Type::String(item[1..item.len() - 1].to_string()));
                continue;
            }

            if let Some(i) = self.memory.get(&item) {
                self.stack.push(i.clone());
                continue;
            }

            match item.as_str() {
                "add" => {
                    let b = self.pop().get_number();
                    let a = self.pop().get_number();
                    self.stack.push(Type::Number(a + b));
                }

                "repeat" => {
                    let count = self.pop().get_number();
                    let text = self.pop().get_string();
                    self.stack.push(Type::String(text.repeat(count as usize)));
                }

                "and" => {
                    let b = self.pop().get_bool();
                    let a = self.pop().get_bool();
                    self.stack.push(Type::Bool(a && b));
                }

                "or" => {
                    let b = self.pop().get_bool();
                    let a = self.pop().get_bool();
                    self.stack.push(Type::Bool(a || b));
                }

                "not" => {
                    let b = self.pop().get_bool();
                    self.stack.push(Type::Bool(!b));
                }

                "input" => {
                    let prompt = self.pop().get_string();
                    self.stack.push(Type::String(input(prompt.as_str())));
                }

                "equal" => {
                    let b = self.pop().get_string();
                    let a = self.pop().get_string();
                    self.stack.push(Type::Bool(a == b));
                }

                "less" => {
                    let b = self.pop().get_number();
                    let a = self.pop().get_number();
                    self.stack.push(Type::Bool(a < b));
                }

                "if" => {
                    let cond = self.pop().get_bool();
                    let code = self.pop().get_string();
                    if cond {
                        self.execute(code)
                    };
                }

                "eval" => {
                    let code = self.pop().get_string();
                    self.execute(code)
                }

                "while" => {
                    let cond = self.pop().get_string();
                    let code = self.pop().get_string();
                    loop {
                        if {
                            self.execute(cond.clone());
                            !self.pop().get_bool()
                        } {
                            break;
                        }
                        self.execute(code.clone());
                    }
                }

                "pop" => {
                    self.pop();
                }

                "concat" => {
                    let b = self.pop().get_string();
                    let a = self.pop().get_string();
                    self.stack.push(Type::String(a + &b));
                }

                "sub" => {
                    let b = self.pop().get_number();
                    let a = self.pop().get_number();
                    self.stack.push(Type::Number(a - b));
                }

                "var" => {
                    let name = self.pop().get_string();
                    let data = self.pop();
                    self.memory
                        .entry(name)
                        .and_modify(|value| *value = data.clone())
                        .or_insert(data);

                    self.log_print(format!("{:?}", self.memory))
                }
                "mul" => {
                    let b = self.pop().get_number();
                    let a = self.pop().get_number();
                    self.stack.push(Type::Number(a * b));
                }

                "div" => {
                    let b = self.pop().get_number();
                    let a = self.pop().get_number();
                    self.stack.push(Type::Number(a / b));
                }

                "mod" => {
                    let b = self.pop().get_number();
                    let a = self.pop().get_number();
                    self.stack.push(Type::Number(a % b));
                }

                "print" => {
                    let a = self.pop().get_string();
                    if let Mode::Debug = self.mode{
                    println!("出力: {a}");}else{println!("{a}");}
                }
                _ => self.stack.push(Type::String(item)),
            }
        }
        self.log_print(format!("| Stack〔{:?} 〕", self.stack));
    }

    fn pop(&mut self) -> Type {
        self.stack.pop().expect("Stack underflow")
    }
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() > 1 {
        if let Ok(code) = get_file_contents(args[1].clone()) {
            let mut executor = Executor::new(Mode::Script);
            executor.execute(code.replace("\n", " ").replace("\r", " "));
        }
    } else {
        println!("Stack Programing Language");
        let mut executor = Executor::new(Mode::Debug);
        loop {
            executor.execute(input("> "))
        }
    }
}

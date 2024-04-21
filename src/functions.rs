use crate::{get_file_contents, input, Executor, Mode, Type};
use clipboard::{ClipboardContext, ClipboardProvider};
use rand::seq::SliceRandom;
use regex::Regex;
use rodio::{OutputStream, Sink, Source};
use rusty_audio::Audio;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::thread::{self, sleep};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{env, fs};
use sys_info::{cpu_num, cpu_speed, hostname, mem_info, os_release, os_type};

pub fn execute_command(executor: &mut Executor, command: String) {
    match command.as_str() {
        // Commands of calculation

        // Addition
        "add" => {
            let b = executor.pop_stack().get_number();
            let a = executor.pop_stack().get_number();
            executor.stack.push(Type::Number(a + b));
        }

        // Subtraction
        "sub" => {
            let b = executor.pop_stack().get_number();
            let a = executor.pop_stack().get_number();
            executor.stack.push(Type::Number(a - b));
        }

        // Multiplication
        "mul" => {
            let b = executor.pop_stack().get_number();
            let a = executor.pop_stack().get_number();
            executor.stack.push(Type::Number(a * b));
        }

        // Division
        "div" => {
            let b = executor.pop_stack().get_number();
            let a = executor.pop_stack().get_number();
            executor.stack.push(Type::Number(a / b));
        }

        // Remainder of division
        "mod" => {
            let b = executor.pop_stack().get_number();
            let a = executor.pop_stack().get_number();
            executor.stack.push(Type::Number(a % b));
        }

        // Exponentiation
        "pow" => {
            let b = executor.pop_stack().get_number();
            let a = executor.pop_stack().get_number();
            executor.stack.push(Type::Number(a.powf(b)));
        }

        // Rounding off
        "round" => {
            let a = executor.pop_stack().get_number();
            executor.stack.push(Type::Number(a.round()));
        }

        // Trigonometric sine
        "sin" => {
            let number = executor.pop_stack().get_number();
            executor.stack.push(Type::Number(number.sin()))
        }

        // Trigonometric cosine
        "cos" => {
            let number = executor.pop_stack().get_number();
            executor.stack.push(Type::Number(number.cos()))
        }

        // Trigonometric tangent
        "tan" => {
            let number = executor.pop_stack().get_number();
            executor.stack.push(Type::Number(number.tan()))
        }

        // Logical operations of AND
        "and" => {
            let b = executor.pop_stack().get_bool();
            let a = executor.pop_stack().get_bool();
            executor.stack.push(Type::Bool(a && b));
        }

        // Logical operations of OR
        "or" => {
            let b = executor.pop_stack().get_bool();
            let a = executor.pop_stack().get_bool();
            executor.stack.push(Type::Bool(a || b));
        }

        // Logical operations of NOT
        "not" => {
            let b = executor.pop_stack().get_bool();
            executor.stack.push(Type::Bool(!b));
        }

        // Judge is it equal
        "equal" => {
            let b = executor.pop_stack().get_string();
            let a = executor.pop_stack().get_string();
            executor.stack.push(Type::Bool(a == b));
        }

        // Judge is it less
        "less" => {
            let b = executor.pop_stack().get_number();
            let a = executor.pop_stack().get_number();
            executor.stack.push(Type::Bool(a < b));
        }

        // Get random value from list
        "rand" => {
            let list = executor.pop_stack().get_list();
            let result = match list.choose(&mut rand::thread_rng()) {
                Some(i) => i.to_owned(),
                None => Type::List(list),
            };
            executor.stack.push(result);
        }

        // Shuffle list by random
        "shuffle" => {
            let mut list = executor.pop_stack().get_list();
            list.shuffle(&mut rand::thread_rng());
            executor.stack.push(Type::List(list));
        }

        // Commands of string processing

        // Repeat string a number of times
        "repeat" => {
            let count = executor.pop_stack().get_number(); // Count
            let text = executor.pop_stack().get_string(); // String
            executor
                .stack
                .push(Type::String(text.repeat(count as usize)));
        }

        // Get unicode character form number
        "decode" => {
            let code = executor.pop_stack().get_number();
            let result = char::from_u32(code as u32);
            match result {
                Some(c) => executor.stack.push(Type::String(c.to_string())),
                None => {
                    executor.log_print("Error! failed of number decoding\n".to_string());
                    executor
                        .stack
                        .push(Type::Error("number-decoding".to_string()));
                }
            }
        }

        // Encode string by UTF-8
        "encode" => {
            let string = executor.pop_stack().get_string();
            if let Some(first_char) = string.chars().next() {
                executor
                    .stack
                    .push(Type::Number((first_char as u32) as f64));
            } else {
                executor.log_print("Error! failed of string encoding\n".to_string());
                executor
                    .stack
                    .push(Type::Error("string-encoding".to_string()));
            }
        }

        // Concatenate the string
        "concat" => {
            let b = executor.pop_stack().get_string();
            let a = executor.pop_stack().get_string();
            executor.stack.push(Type::String(a + &b));
        }

        // Replacing string
        "replace" => {
            let after = executor.pop_stack().get_string();
            let before = executor.pop_stack().get_string();
            let text = executor.pop_stack().get_string();
            executor
                .stack
                .push(Type::String(text.replace(&before, &after)))
        }

        // Split string by the key
        "split" => {
            let key = executor.pop_stack().get_string();
            let text = executor.pop_stack().get_string();
            executor.stack.push(Type::List(
                text.split(&key)
                    .map(|x| Type::String(x.to_string()))
                    .collect::<Vec<Type>>(),
            ));
        }

        // Change string style case
        "case" => {
            let types = executor.pop_stack().get_string();
            let text = executor.pop_stack().get_string();

            executor.stack.push(Type::String(match types.as_str() {
                "lower" => text.to_lowercase(),
                "upper" => text.to_uppercase(),
                _ => text,
            }));
        }

        // Generate a string by concat list
        "join" => {
            let key = executor.pop_stack().get_string();
            let mut list = executor.pop_stack().get_list();
            executor.stack.push(Type::String(
                list.iter_mut()
                    .map(|x| x.get_string())
                    .collect::<Vec<String>>()
                    .join(&key),
            ))
        }

        // Judge is it find in string
        "find" => {
            let word = executor.pop_stack().get_string();
            let text = executor.pop_stack().get_string();
            executor.stack.push(Type::Bool(text.contains(&word)))
        }

        // Search by regular expression
        "regex" => {
            let pattern = executor.pop_stack().get_string();
            let text = executor.pop_stack().get_string();

            let pattern: Regex = match Regex::new(pattern.as_str()) {
                Ok(i) => i,
                Err(e) => {
                    executor.log_print(format!("Error! {}\n", e.to_string().replace("Error", "")));
                    executor.stack.push(Type::Error("regex".to_string()));
                    return;
                }
            };

            let mut list: Vec<Type> = Vec::new();
            for i in pattern.captures_iter(text.as_str()) {
                list.push(Type::String(i[0].to_string()))
            }
            executor.stack.push(Type::List(list));
        }

        // Commands of I/O

        // Write string in the file
        "write-file" => {
            let mut file = match File::create(Path::new(&executor.pop_stack().get_string())) {
                Ok(file) => file,
                Err(e) => {
                    executor.log_print(format!("Error! {e}\n"));
                    executor.stack.push(Type::Error("create-file".to_string()));
                    return;
                }
            };
            if let Err(e) = file.write_all(executor.pop_stack().get_string().as_bytes()) {
                executor.log_print(format!("Error! {}\n", e));
                executor.stack.push(Type::Error("write-file".to_string()));
            }
        }

        // Read string in the file
        "read-file" => {
            let name = Path::new(&executor.pop_stack().get_string()).to_owned();
            match get_file_contents(&name) {
                Ok(s) => executor.stack.push(Type::String(s)),
                Err(e) => {
                    executor.log_print(format!("Error! {}\n", e));
                    executor.stack.push(Type::Error("read-file".to_string()));
                }
            };
        }

        // Standard input
        "input" => {
            let prompt = executor.pop_stack().get_string();
            executor.stack.push(Type::String(input(prompt.as_str())));
        }

        // Standard output
        "print" => {
            let a = executor.pop_stack().get_string();

            let a = a.replace("\\n", "\n");
            let a = a.replace("\\t", "\t");
            let a = a.replace("\\r", "\r");

            if let Mode::Debug = executor.mode {
                println!("[Output]: {a}");
            } else {
                print!("{a}");
            }
        }

        // Standard output with new line
        "println" => {
            let a = executor.pop_stack().get_string();

            let a = a.replace("\\n", "\n");
            let a = a.replace("\\t", "\t");
            let a = a.replace("\\r", "\r");

            if let Mode::Debug = executor.mode {
                println!("[Output]: {a}");
            } else {
                println!("{a}");
            }
        }

        // Get command-line arguments
        "args-cmd" => executor.stack.push(Type::List(
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

            let duration_secs = executor.pop_stack().get_number();
            let frequency = executor.pop_stack().get_number();

            play_sine_wave(frequency, duration_secs);
        }

        // Play the music file
        "play-file" => {
            let path = executor.pop_stack().get_string();
            let sound_file_path = Path::new(&path);

            let res_sound_file = File::open(sound_file_path);

            if let Err(e) = res_sound_file {
                executor.log_print(format!("Error! {}\n", e));
                executor.stack.push(Type::Error("play-file".to_string()));
            } else {
                let mut audio_device = Audio::new();
                audio_device.add("sound", path.clone());
                audio_device.play("sound");
                audio_device.wait();

                executor.stack.push(Type::String(path));
            }
        }

        // Claer the console screen
        "cls" | "clear" => {
            let result = clearscreen::clear();
            if result.is_err() {
                println!("Error! Failed to clear screen");
                executor
                    .stack
                    .push(Type::Error(String::from("failed-to-clear-screen")));
            }
        }

        // Commands of control

        // Evaluate string as program
        "eval" => {
            let code = executor.pop_stack().get_string();
            executor.evaluate_program(code)
        }

        // Conditional branch
        "if" => {
            let condition = executor.pop_stack().get_bool(); // Condition
            let code_else = executor.pop_stack().get_string(); // Code of else
            let code_if = executor.pop_stack().get_string(); // Code of If
            if condition {
                executor.evaluate_program(code_if)
            } else {
                executor.evaluate_program(code_else)
            };
        }

        // Loop while condition is true
        "while" => {
            let cond = executor.pop_stack().get_string();
            let code = executor.pop_stack().get_string();
            while {
                executor.evaluate_program(cond.clone());
                executor.pop_stack().get_bool()
            } {
                executor.evaluate_program(code.clone());
            }
        }

        // Generate a thread
        "thread" => {
            let code = executor.pop_stack().get_string();
            let mut executor = executor.clone();
            thread::spawn(move || executor.evaluate_program(code));
        }

        // Exit a process
        "exit" => {
            let status = executor.pop_stack().get_number();
            std::process::exit(status as i32);
        }

        // Commands of list processing

        // Get list value by index
        "get" => {
            let index = executor.pop_stack().get_number() as usize;
            let list: Vec<Type> = executor.pop_stack().get_list();
            if list.len() > index {
                executor.stack.push(list[index].clone());
            } else {
                executor.log_print("Error! Index specification is out of range\n".to_string());
                executor
                    .stack
                    .push(Type::Error("index-out-range".to_string()));
            }
        }

        // Set list value by index
        "set" => {
            let value = executor.pop_stack();
            let index = executor.pop_stack().get_number() as usize;
            let mut list: Vec<Type> = executor.pop_stack().get_list();
            if list.len() > index {
                list[index] = value;
                executor.stack.push(Type::List(list));
            } else {
                executor.log_print("Error! Index specification is out of range\n".to_string());
                executor
                    .stack
                    .push(Type::Error("index-out-range".to_string()));
            }
        }

        // Delete list value by index
        "del" => {
            let index = executor.pop_stack().get_number() as usize;
            let mut list = executor.pop_stack().get_list();
            if list.len() > index {
                list.remove(index);
                executor.stack.push(Type::List(list));
            } else {
                executor.log_print("Error! Index specification is out of range\n".to_string());
                executor
                    .stack
                    .push(Type::Error("index-out-range".to_string()));
            }
        }

        // Append value in the list
        "append" => {
            let data = executor.pop_stack();
            let mut list = executor.pop_stack().get_list();
            list.push(data);
            executor.stack.push(Type::List(list));
        }

        // Insert value in the list
        "insert" => {
            let data = executor.pop_stack();
            let index = executor.pop_stack().get_number();
            let mut list = executor.pop_stack().get_list();
            list.insert(index as usize, data);
            executor.stack.push(Type::List(list));
        }

        // Get index of the list
        "index" => {
            let target = executor.pop_stack().get_string();
            let list = executor.pop_stack().get_list();

            for (index, item) in list.iter().enumerate() {
                if target == item.clone().get_string() {
                    executor.stack.push(Type::Number(index as f64));
                    return;
                }
            }
            executor.log_print(String::from("Error! item not found in the list\n"));
            executor
                .stack
                .push(Type::Error(String::from("item-not-found")));
        }

        // Sorting in the list
        "sort" => {
            let mut list: Vec<String> = executor
                .pop_stack()
                .get_list()
                .iter()
                .map(|x| x.to_owned().get_string())
                .collect();
            list.sort();
            executor.stack.push(Type::List(
                list.iter()
                    .map(|x| Type::String(x.to_string()))
                    .collect::<Vec<_>>(),
            ));
        }

        // reverse in the list
        "reverse" => {
            let mut list = executor.pop_stack().get_list();
            list.reverse();
            executor.stack.push(Type::List(list));
        }

        // Iteration for the list
        "for" => {
            let code = executor.pop_stack().get_string();
            let vars = executor.pop_stack().get_string();
            let list = executor.pop_stack().get_list();

            list.iter().for_each(|x| {
                executor
                    .memory
                    .entry(vars.clone())
                    .and_modify(|value| *value = x.clone())
                    .or_insert(x.clone());
                executor.evaluate_program(code.clone());
            });
        }

        // Generate a range
        "range" => {
            let step = executor.pop_stack().get_number();
            let max = executor.pop_stack().get_number();
            let min = executor.pop_stack().get_number();

            let mut range: Vec<Type> = Vec::new();

            for i in (min as usize..max as usize).step_by(step as usize) {
                range.push(Type::Number(i as f64));
            }

            executor.stack.push(Type::List(range));
        }

        // Get length of list
        "len" => {
            let data = executor.pop_stack().get_list();
            executor.stack.push(Type::Number(data.len() as f64));
        }

        // Commands of functional programming

        // Mapping a list
        "map" => {
            let code = executor.pop_stack().get_string();
            let vars = executor.pop_stack().get_string();
            let list = executor.pop_stack().get_list();

            let mut result_list = Vec::new();
            for x in list.iter() {
                executor
                    .memory
                    .entry(vars.clone())
                    .and_modify(|value| *value = x.clone())
                    .or_insert(x.clone());

                executor.evaluate_program(code.clone());
                result_list.push(executor.pop_stack());
            }

            executor.stack.push(Type::List(result_list));
        }

        // Filtering a list value
        "filter" => {
            let code = executor.pop_stack().get_string();
            let vars = executor.pop_stack().get_string();
            let list = executor.pop_stack().get_list();

            let mut result_list = Vec::new();

            for x in list.iter() {
                executor
                    .memory
                    .entry(vars.clone())
                    .and_modify(|value| *value = x.clone())
                    .or_insert(x.clone());

                executor.evaluate_program(code.clone());
                if executor.pop_stack().get_bool() {
                    result_list.push(x.clone());
                }
            }

            executor.stack.push(Type::List(result_list));
        }

        // Generate value from list
        "reduce" => {
            let code = executor.pop_stack().get_string();
            let now = executor.pop_stack().get_string();
            let init = executor.pop_stack();
            let acc = executor.pop_stack().get_string();
            let list = executor.pop_stack().get_list();

            executor
                .memory
                .entry(acc.clone())
                .and_modify(|value| *value = init.clone())
                .or_insert(init);

            for x in list.iter() {
                executor
                    .memory
                    .entry(now.clone())
                    .and_modify(|value| *value = x.clone())
                    .or_insert(x.clone());

                executor.evaluate_program(code.clone());
                let result = executor.pop_stack();

                executor
                    .memory
                    .entry(acc.clone())
                    .and_modify(|value| *value = result.clone())
                    .or_insert(result);
            }

            let result = executor.memory.get(&acc);
            executor
                .stack
                .push(result.unwrap_or(&Type::String("".to_string())).clone());

            executor
                .memory
                .entry(acc.clone())
                .and_modify(|value| *value = Type::String("".to_string()))
                .or_insert(Type::String("".to_string()));
        }

        // Commands of memory manage

        // Pop in the stack
        "pop" => {
            executor.pop_stack();
        }

        // Get size of stack
        "size-stack" => {
            let len: f64 = executor.stack.len() as f64;
            executor.stack.push(Type::Number(len));
        }

        // Get Stack as List
        "get-stack" => {
            executor.stack.push(Type::List(executor.stack.clone()));
        }

        // Define variable at memory
        "var" => {
            let name = executor.pop_stack().get_string();
            let data = executor.pop_stack();
            executor
                .memory
                .entry(name)
                .and_modify(|value| *value = data.clone())
                .or_insert(data);
            executor.show_variables()
        }

        // Get data type of value
        "type" => {
            let result = match executor.pop_stack() {
                Type::Number(_) => "number".to_string(),
                Type::String(_) => "string".to_string(),
                Type::Bool(_) => "bool".to_string(),
                Type::List(_) => "list".to_string(),
                Type::Error(_) => "error".to_string(),
                Type::Object(name, _) => name.to_string(),
            };

            executor.stack.push(Type::String(result));
        }

        // Explicit data type casting
        "cast" => {
            let types = executor.pop_stack().get_string();
            let mut value = executor.pop_stack();
            match types.as_str() {
                "number" => executor.stack.push(Type::Number(value.get_number())),
                "string" => executor.stack.push(Type::String(value.get_string())),
                "bool" => executor.stack.push(Type::Bool(value.get_bool())),
                "list" => executor.stack.push(Type::List(value.get_list())),
                "error" => executor.stack.push(Type::Error(value.get_string())),
                _ => executor.stack.push(value),
            }
        }

        // Get memory information
        "mem" => {
            let mut list: Vec<Type> = Vec::new();
            for (name, _) in executor.memory.clone() {
                list.push(Type::String(name))
            }
            executor.stack.push(Type::List(list))
        }

        // Free up memory space of variable
        "free" => {
            let name = executor.pop_stack().get_string();
            executor.memory.remove(name.as_str());
            executor.show_variables();
        }

        // Copy stack's top value
        "copy" => {
            let data = executor.pop_stack();
            executor.stack.push(data.clone());
            executor.stack.push(data);
        }

        // Swap stack's top 2 value
        "swap" => {
            let b = executor.pop_stack();
            let a = executor.pop_stack();
            executor.stack.push(b);
            executor.stack.push(a);
        }

        // Commands of times

        // Get now time as unix epoch
        "now-time" => {
            executor.stack.push(Type::Number(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64(),
            ));
        }

        // Sleep fixed time
        "sleep" => sleep(Duration::from_secs_f64(executor.pop_stack().get_number())),

        // Commands of object oriented system

        // Generate a instance of object
        "instance" => {
            let data = executor.pop_stack().get_list();
            let mut class = executor.pop_stack().get_list();
            let mut object: HashMap<String, Type> = HashMap::new();

            let name = if !class.is_empty() {
                class[0].get_string()
            } else {
                executor.log_print("Error! the type name is not found.".to_string());
                executor
                    .stack
                    .push(Type::Error("instance-name".to_string()));
                return;
            };

            let mut index = 0;
            for item in &mut class.to_owned()[1..class.len()].iter() {
                let mut item = item.to_owned();
                if item.get_list().len() == 1 {
                    let element = match data.get(index) {
                        Some(value) => value,
                        None => {
                            executor.log_print("Error! initial data is shortage\n".to_string());
                            executor
                                .stack
                                .push(Type::Error("instance-shortage".to_string()));
                            return;
                        }
                    };
                    object.insert(
                        item.get_list()[0].to_owned().get_string(),
                        element.to_owned(),
                    );
                    index += 1;
                } else if item.get_list().len() >= 2 {
                    let item = item.get_list();
                    object.insert(item[0].clone().get_string(), item[1].clone());
                } else {
                    executor.log_print("Error! the class data structure is wrong.".to_string());
                    executor
                        .stack
                        .push(Type::Error("instance-default".to_string()));
                }
            }

            executor.stack.push(Type::Object(name, object))
        }

        // Get property of object
        "property" => {
            let name = executor.pop_stack().get_string();
            match executor.pop_stack() {
                Type::Object(_, data) => executor.stack.push(
                    data.get(name.as_str())
                        .unwrap_or(&Type::Error("property".to_string()))
                        .clone(),
                ),
                _ => executor.stack.push(Type::Error("not-object".to_string())),
            }
        }

        // Call the method of object
        "method" => {
            let method = executor.pop_stack().get_string();
            match executor.pop_stack() {
                Type::Object(name, value) => {
                    let data = Type::Object(name, value.clone());
                    executor
                        .memory
                        .entry("executor".to_string())
                        .and_modify(|value| *value = data.clone())
                        .or_insert(data);

                    let program: String = match value.get(&method) {
                        Some(i) => i.to_owned().get_string().to_string(),
                        None => "".to_string(),
                    };

                    executor.evaluate_program(program)
                }
                _ => executor.stack.push(Type::Error("not-object".to_string())),
            }
        }

        // Modify the property of object
        "modify" => {
            let data = executor.pop_stack();
            let property = executor.pop_stack().get_string();
            match executor.pop_stack() {
                Type::Object(name, mut value) => {
                    value
                        .entry(property)
                        .and_modify(|value| *value = data.clone())
                        .or_insert(data.clone());

                    executor.stack.push(Type::Object(name, value))
                }
                _ => executor.stack.push(Type::Error("not-object".to_string())),
            }
        }

        // Get all of properties
        "all" => match executor.pop_stack() {
            Type::Object(_, data) => executor.stack.push(Type::List(
                data.keys()
                    .map(|x| Type::String(x.to_owned()))
                    .collect::<Vec<Type>>(),
            )),
            _ => executor.stack.push(Type::Error("not-object".to_string())),
        },

        // Commands of external cooperation processing

        // Send the http request
        "request" => {
            let url = executor.pop_stack().get_string();
            match reqwest::blocking::get(url) {
                Ok(i) => executor
                    .stack
                    .push(Type::String(i.text().unwrap_or("".to_string()))),
                Err(e) => {
                    executor.log_print(format!("Error! {e}\n"));
                    executor.stack.push(Type::Error("request".to_string()))
                }
            }
        }

        // Open the file or url
        "open" => {
            let name = executor.pop_stack().get_string();
            if let Err(e) = opener::open(name.clone()) {
                executor.log_print(format!("Error! {e}\n"));
                executor.stack.push(Type::Error("open".to_string()));
            } else {
                executor.stack.push(Type::String(name))
            }
        }

        // Change current directory
        "cd" => {
            let name = executor.pop_stack().get_string();
            if let Err(err) = std::env::set_current_dir(name.clone()) {
                executor.log_print(format!("Error! {}\n", err));
                executor.stack.push(Type::Error("cd".to_string()));
            } else {
                executor.stack.push(Type::String(name))
            }
        }

        // Get current directory
        "pwd" => {
            if let Ok(current_dir) = std::env::current_dir() {
                if let Some(path) = current_dir.to_str() {
                    executor.stack.push(Type::String(String::from(path)));
                }
            }
        }

        // Make directory
        "mkdir" => {
            let name = executor.pop_stack().get_string();
            if let Err(e) = fs::create_dir(name.clone()) {
                executor.log_print(format!("Error! {e}\n"));
                executor.stack.push(Type::Error("mkdir".to_string()));
            } else {
                executor.stack.push(Type::String(name))
            }
        }

        // Remove item
        "rm" => {
            let name = executor.pop_stack().get_string();
            if Path::new(name.as_str()).is_dir() {
                if let Err(e) = fs::remove_dir(name.clone()) {
                    executor.log_print(format!("Error! {e}\n"));
                    executor.stack.push(Type::Error("rm".to_string()));
                } else {
                    executor.stack.push(Type::String(name))
                }
            } else if let Err(e) = fs::remove_file(name.clone()) {
                executor.log_print(format!("Error! {e}\n"));
                executor.stack.push(Type::Error("rm".to_string()));
            } else {
                executor.stack.push(Type::String(name))
            }
        }

        // Rename item
        "rename" => {
            let to = executor.pop_stack().get_string();
            let from = executor.pop_stack().get_string();
            if let Err(e) = fs::rename(from, to.clone()) {
                executor.log_print(format!("Error! {e}\n"));
                executor.stack.push(Type::Error("rename".to_string()));
            } else {
                executor.stack.push(Type::String(to))
            }
        }

        // Copy the item
        "cp" => {
            let to = executor.pop_stack().get_string();
            let from = executor.pop_stack().get_string();

            match fs::copy(from, to) {
                Ok(i) => executor.stack.push(Type::Number(i as f64)),
                Err(e) => {
                    executor.log_print(format!("Error! {e}\n"));
                    executor.stack.push(Type::Error("cp".to_string()))
                }
            }
        }

        // Get size of the file
        "size-file" => match fs::metadata(executor.pop_stack().get_string()) {
            Ok(i) => executor.stack.push(Type::Number(i.len() as f64)),
            Err(e) => {
                executor.log_print(format!("Error! {e}\n"));
                executor.stack.push(Type::Error("size-file".to_string()))
            }
        },

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
                executor.stack.push(Type::List(value));
            }
        }

        // Judge is it folder
        "folder" => {
            let path = executor.pop_stack().get_string();
            let path = Path::new(path.as_str());
            executor.stack.push(Type::Bool(path.is_dir()));
        }

        // Get system information
        "sys-info" => {
            let option = executor.pop_stack().get_string();
            executor.stack.push(match option.as_str() {
                "os-release" => Type::String(os_release().unwrap_or("".to_string())),
                "os-type" => Type::String(os_type().unwrap_or("".to_string())),
                "cpu-num" => Type::Number(cpu_num().unwrap_or(0) as f64),
                "cpu-speed" => Type::Number(cpu_speed().unwrap_or(0) as f64),
                "host-name" => Type::String(hostname().unwrap_or("".to_string())),
                "mem-size" => match mem_info() {
                    Ok(info) => Type::Number(info.total as f64),
                    Err(_) => Type::Error("sys-info".to_string()),
                },
                "mem-used" => match mem_info() {
                    Ok(info) => Type::Number((info.total - info.free) as f64),
                    Err(_) => Type::Error("sys-info".to_string()),
                },
                _ => Type::Error("sys-info".to_string()),
            })
        }

        // Set value in the clipboard
        "set-clipboard" => {
            let mut ctx: ClipboardContext;
            if let Ok(i) = ClipboardProvider::new() {
                ctx = i
            } else {
                executor
                    .stack
                    .push(Type::Error("set-clipboard".to_string()));
                return;
            };

            let value = executor.pop_stack().get_string();
            if ctx.set_contents(value.clone()).is_ok() {
                executor.stack.push(Type::String(value));
            } else {
                executor
                    .stack
                    .push(Type::Error("set-clipboard".to_string()))
            };
        }

        // Get value in the clipboard
        "get-clipboard" => {
            let mut ctx: ClipboardContext;
            if let Ok(i) = ClipboardProvider::new() {
                ctx = i
            } else {
                executor
                    .stack
                    .push(Type::Error("get-clipboard".to_string()));
                return;
            };

            if let Ok(contents) = ctx.get_contents() {
                executor.stack.push(Type::String(contents));
            } else {
                executor
                    .stack
                    .push(Type::Error("get-clipboard".to_string()))
            }
        }

        // If it is not recognized as a command, use it as a string.
        _ => executor.stack.push(Type::String(command)),
    }
}

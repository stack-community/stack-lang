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

// 標準入力
pub fn input(prompt: &str) -> String {
    print!("{}", prompt.to_string());
    io::stdout().flush().unwrap();
    let mut result = String::new();
    io::stdin().read_line(&mut result).ok();
    return result.trim().parse().ok().unwrap();
}

// 実行モード
#[derive(Clone, Debug)]
enum Mode {
    Script, // スクリプト実行
    Debug,  // デバッグ実行
}

// データ型
#[derive(Clone, Debug)]
enum Type {
    Number(f64),    //数値
    String(String), //文字列
    Bool(bool),     //論理
}

impl Type {
    // 文字列を取得
    fn get_string(&mut self) -> String {
        match self {
            Type::String(s) => s.to_string(),
            Type::Number(i) => i.to_string(),
            Type::Bool(b) => b.to_string(),
        }
    }

    // 数値を取得
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

    // 論理値を取得
    fn get_bool(&mut self) -> bool {
        match self {
            Type::String(s) => s.len() != 0,
            Type::Number(i) => *i != 0.0,
            Type::Bool(b) => *b,
        }
    }
}

struct Executor {
    stack: Vec<Type>,              // スタック
    memory: HashMap<String, Type>, // 変数のメモリ領域
    mode: Mode,                    // 実行モード
}

impl Executor {
    // コンストラクタ
    fn new(mode: Mode) -> Executor {
        Executor {
            stack: Vec::new(),
            memory: HashMap::new(),
            mode,
        }
    }

    // ログ表示
    fn log_print(&mut self, msg: String) {
        if let Mode::Debug = self.mode {
            println!("{msg}");
        }
    }

    // プログラムを評価する
    fn execute(&mut self, code: String) {
        // トークンを整える
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

            // 煤地に変換できたらスタックに積む
            if let Ok(i) = item.parse::<f64>() {
                self.stack.push(Type::Number(i));
                continue;
            }

            // 論理値trueをスタックに積む
            if item == "true" {
                self.stack.push(Type::Bool(true));
                continue;
            }

            // 論理値falseをスタックに積む
            if item == "false" {
                self.stack.push(Type::Bool(false));
                continue;
            }

            // 文字列
            if item.contains("(") || item.contains(')') {
                self.stack
                    .push(Type::String(item[1..item.len() - 1].to_string()));
                continue;
            }

            // 変数を読み込む
            if let Some(i) = self.memory.get(&item) {
                self.stack.push(i.clone());
                continue;
            }

            // コマンドを実行
            match item.as_str() {
                // 足し算(数値,数値)->数値
                "add" => {
                    let b = self.pop().get_number();
                    let a = self.pop().get_number();
                    self.stack.push(Type::Number(a + b));
                }

                // 文字列を回数分リピート(文字列,数値)->文字列
                "repeat" => {
                    let count = self.pop().get_number(); // 回数
                    let text = self.pop().get_string(); // 文字列
                    self.stack.push(Type::String(text.repeat(count as usize)));
                }

                // AND演算(論理,論理)->論理
                "and" => {
                    let b = self.pop().get_bool();
                    let a = self.pop().get_bool();
                    self.stack.push(Type::Bool(a && b));
                }

                // OR演算(論理,論理)->論理
                "or" => {
                    let b = self.pop().get_bool();
                    let a = self.pop().get_bool();
                    self.stack.push(Type::Bool(a || b));
                }

                // NOT演算(論理)->論理
                "not" => {
                    let b = self.pop().get_bool();
                    self.stack.push(Type::Bool(!b));
                }

                // 標準入力(文字列)->文字列
                "input" => {
                    let prompt = self.pop().get_string(); //プロンプト
                    self.stack.push(Type::String(input(prompt.as_str())));
                }

                // 等しいか(文字列,文字列)->論理
                "equal" => {
                    let b = self.pop().get_string();
                    let a = self.pop().get_string();
                    self.stack.push(Type::Bool(a == b));
                }

                // 未満か(数値,数値)->論理
                "less" => {
                    let b = self.pop().get_number();
                    let a = self.pop().get_number();
                    self.stack.push(Type::Bool(a < b));
                }

                // 条件分岐(コード:文字列,論理)
                "if" => {
                    let cond = self.pop().get_bool(); // 条件式
                    let code = self.pop().get_string(); // コード
                    if cond {
                        self.execute(code)
                    };
                }

                // 文字列を式として評価(コード:文字列)
                "eval" => {
                    let code = self.pop().get_string();
                    self.execute(code)
                }

                // 条件が一致してる間ループ(コード:文字列, 条件式:文字列)
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

                // スタックの値をポップ()
                "pop" => {
                    self.pop();
                }

                // 文字列を結合(文字列, 文字列)->文字列
                "concat" => {
                    let b = self.pop().get_string();
                    let a = self.pop().get_string();
                    self.stack.push(Type::String(a + &b));
                }

                // 引き算(数値,数値)->数値
                "sub" => {
                    let b = self.pop().get_number();
                    let a = self.pop().get_number();
                    self.stack.push(Type::Number(a - b));
                }

                // 変数定義(データ, 変数名:文字列)
                "var" => {
                    let name = self.pop().get_string(); // 変数名
                    let data = self.pop(); // 値
                    self.memory
                        .entry(name)
                        .and_modify(|value| *value = data.clone())
                        .or_insert(data);

                    self.log_print(format!("{:?}", self.memory))
                }

                // 掛け算(数値,数値)->数値
                "mul" => {
                    let b = self.pop().get_number();
                    let a = self.pop().get_number();
                    self.stack.push(Type::Number(a * b));
                }

                // 割り算(数値,数値)->数値
                "div" => {
                    let b = self.pop().get_number();
                    let a = self.pop().get_number();
                    self.stack.push(Type::Number(a / b));
                }

                // 商の余り(数値,数値)->数値
                "mod" => {
                    let b = self.pop().get_number();
                    let a = self.pop().get_number();
                    self.stack.push(Type::Number(a % b));
                }

                // 表示出力(文字列)
                "print" => {
                    let a = self.pop().get_string();
                    if let Mode::Debug = self.mode {
                        println!("出力: {a}");
                    } else {
                        println!("{a}");
                    }
                }
                _ => self.stack.push(Type::String(item)),
            }
        }
        self.log_print(format!("| Stack〔{:?} 〕", self.stack));
    }

    // スタックの値をポップ
    fn pop(&mut self) -> Type {
        self.stack.pop().expect("Stack underflow")
    }
}

fn main() {
    // コマンドライン引数を読み込む
    let args = env::args().collect::<Vec<_>>();
    if args.len() > 2 {
        // ファイルを開く
        if let Ok(code) = get_file_contents(args[2].clone()) {
            let mut executor = Executor::new(if args[1].contains("d") {
                Mode::Debug
            } else {
                Mode::Script
            });
            executor.execute(code.replace("\n", " ").replace("\r", " "));
        }
    } else if args.len() > 1 {
        // ファイルを開く
        if let Ok(code) = get_file_contents(args[1].clone()) {
            let mut executor = Executor::new(Mode::Script);
            executor.execute(code.replace("\n", " ").replace("\r", " "));
        }
    } else {
        println!("Stack Programing Language");
        let mut executor = Executor::new(Mode::Debug);
        // REPL実行
        loop {
            executor.execute(input("> "))
        }
    }
}

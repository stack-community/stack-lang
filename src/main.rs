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
    Number(f64),     //数値
    String(String),  //文字列
    Bool(bool),      //論理
    List(Vec<Type>), //リスト
}

impl Type {
    // 文字列を取得
    fn get_string(&mut self) -> String {
        match self {
            Type::String(s) => s.to_string(),
            Type::Number(i) => i.to_string(),
            Type::Bool(b) => b.to_string(),
            Type::List(l) => format!("{l:?}"),
        }
    }

    // 数値を取得
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
        }
    }

    // 論理値を取得
    fn get_bool(&mut self) -> bool {
        match self {
            Type::String(s) => s.len() != 0,
            Type::Number(i) => *i != 0.0,
            Type::Bool(b) => *b,
            Type::List(l) => l.len() != 0,
        }
    }

    fn get_list(&mut self) -> Vec<Type> {
        match self {
            Type::String(s) => vec![Type::String(s.to_string())],
            Type::Number(i) => vec![Type::Number(*i)],
            Type::Bool(b) => vec![Type::Bool(*b)],
            Type::List(l) => l.to_vec(),
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
            let mut in_parentheses = 0;
            let mut in_hash = false;

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
                    ' ' | '　' if !in_hash && in_parentheses == 0 && in_brackets == 0 => {
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
            self.log_print(format!("| Stack {:?} ←  {}", self.stack, item));

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

            let chars: Vec<char> = item.chars().collect();

            // 文字列
            if chars[0] == '(' || chars[chars.len() - 1] == ')' {
                self.stack
                    .push(Type::String(item[1..item.len() - 1].to_string()));
                continue;
            }

            // リスト
            if chars[0] == '[' || chars[chars.len() - 1] == ']' {
                let old_len = self.stack.len();
                let slice = &item[1..item.len() - 1];
                let token: Vec<_> = slice.split_whitespace().map(|x| x.to_string()).collect();
                self.execute(
                    token
                        .into_iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                );
                let mut list = Vec::new();
                for _ in old_len..self.stack.len() {
                    list.push(self.pop());
                }
                list.reverse();
                self.stack.push(Type::List(list));
                continue;
            }

            // 変数を読み込む
            if let Some(i) = self.memory.get(&item) {
                self.stack.push(i.clone());
                continue;
            }

            if item.contains("#") {
                self.log_print(format!("※ コメント「{}」", item.replace("#", "")));
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

                "map" => {
                    let code = self.pop().get_string();
                    let vars = self.pop().get_string();
                    let list = self.pop().get_list();

                    list.iter().for_each(|x| {
                        self.memory
                            .entry(vars.clone())
                            .and_modify(|value| *value = x.clone())
                            .or_insert(x.clone());
                        self.execute(code.clone());
                    });
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

                "split" => {
                    let key = self.pop().get_string();
                    let text = self.pop().get_string();
                    self.stack.push(Type::List(
                        text.split(&key)
                            .map(|x| Type::String(x.to_string()))
                            .collect::<Vec<Type>>(),
                    ));
                }

                "join" => {
                    let key = self.pop().get_string();
                    let mut list = self.pop().get_list();
                    self.stack.push(Type::String(
                        list.iter_mut()
                            .map(|x| x.get_string())
                            .collect::<Vec<String>>()
                            .join(&key),
                    ))
                }

                "replace" => {
                    let after = self.pop().get_string();
                    let before = self.pop().get_string();
                    let text = self.pop().get_string();
                    self.stack.push(Type::String(text.replace(&before, &after)))
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

                // 四捨五入(数値)->数値
                "round" => {
                    let a = self.pop().get_number();
                    self.stack.push(Type::Number(a.round()));
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

                //リストの値を取得(リスト, 数値)->データ
                "get" => {
                    let index = self.pop().get_number();
                    let list = self.pop().get_list();
                    self.stack.push(list[index as usize].clone());
                }

                //リストの値を削除(リスト, 数値)
                "del" => {
                    let index = self.pop().get_number();
                    let mut list = self.pop().get_list();
                    list.remove(index as usize);
                    self.stack.push(Type::List(list));
                }

                //リストの値を設定(リスト, 数値, データ)
                "set" => {
                    let index = self.pop().get_number();
                    let mut list = self.pop().get_list();
                    let data = self.pop();
                    list[index as usize] = data;
                    self.stack.push(Type::List(list));
                }

                //リストに追加(リスト, 追加するデータ)
                "append" => {
                    let data = self.pop();
                    let mut list = self.pop().get_list();
                    list.push(data);
                    self.stack.push(Type::List(list));
                }

                "insert" => {
                    let data = self.pop();
                    let index = self.pop().get_number();
                    let mut list = self.pop().get_list();
                    list.insert(index as usize, data);
                    self.stack.push(Type::List(list));
                }

                "find" => {
                    let word = self.pop().get_string();
                    let text = self.pop().get_string();
                    self.stack.push(Type::Bool(text.contains(&word)))
                }

                "len" => {
                    let data = self.pop();
                    self.stack.push(Type::Number(match data {
                        Type::List(l) => l.len() as f64,
                        Type::String(s) => s.chars().count() as f64,
                        _ => 1f64,
                    }));
                }

                "copy" => {
                    let data = self.pop();
                    self.stack.push(data.clone());
                    self.stack.push(data);
                }

                _ => self.stack.push(Type::String(item)),
            }
        }
        self.log_print(format!("| Stack {:?}", self.stack));
    }

    // スタックの値をポップ
    fn pop(&mut self) -> Type {
        if let Some(value) = self.stack.pop() {
            value
        } else {
            println!("エラー! スタックの値が足りません。デフォルト値を返します");
            Type::String("".to_string())
        }
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
        println!("Stack プログラミング言語");
        println!("(c) 2023 梶塚太智. All rights reserved");
        let mut executor = Executor::new(Mode::Debug);
        // REPL実行
        loop {
            executor.execute(input("> "))
        }
    }
}

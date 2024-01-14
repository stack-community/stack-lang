use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, Error, Read, Write};

fn main() {
    // コマンドライン引数を読み込む
    let args = env::args().collect::<Vec<_>>();
    if args.len() > 2 {
        // ファイルを開く
        if let Ok(code) = get_file_contents(args[2].clone()) {
            let code = code.replace("\n", " ").replace("\r", " ");
            // 実行モードを判定する
            if args[1].contains("d") {
                let mut executor = Executor::new(Mode::Debug);
                executor.evaluate_program(code); //デバッグ実行
            } else {
                let mut executor = Executor::new(Mode::Script);
                executor.evaluate_program(code); // スクリプト実行
            }
        }
    } else if args.len() > 1 {
        // ファイルを開く
        if let Ok(code) = get_file_contents(args[1].clone()) {
            let mut executor = Executor::new(Mode::Script); //デフォルメ値はスクリプト実行
            executor.evaluate_program(code.replace("\n", " ").replace("\r", " "));
        }
    } else {
        // タイトルを表示する
        println!("Stack プログラミング言語");
        println!("(c) 2023 梶塚太智. All rights reserved");
        let mut executor = Executor::new(Mode::Debug);
        // REPL実行
        loop {
            executor.evaluate_program(input("> "))
        }
    }
}

/// ファイルを読み込む
fn get_file_contents(name: String) -> Result<String, Error> {
    let mut f = File::open(name.trim())?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    Ok(contents)
}

/// 標準入力を受け取る
fn input(prompt: &str) -> String {
    print!("{}", prompt.to_string());
    io::stdout().flush().unwrap();
    let mut result = String::new();
    io::stdin().read_line(&mut result).ok();
    return result.trim().parse().ok().unwrap();
}

/// 実行モード
#[derive(Clone, Debug)]
enum Mode {
    Script, // スクリプト実行
    Debug,  // デバッグ実行
}

/// データ型
#[derive(Clone, Debug)]
enum Type {
    Number(f64),     //数値
    String(String),  //文字列
    Bool(bool),      //論理
    List(Vec<Type>), //リスト
}

/// メソッド実装
impl Type {
    /// ディスプレイに表示
    fn display(&self) -> String {
        match self {
            Type::Number(num) => num.to_string(),
            Type::String(s) => format!("({})", s),
            Type::Bool(b) => b.to_string(),
            Type::List(list) => {
                let syntax: Vec<String> = list.iter().map(|token| token.display()).collect();
                format!("[{}]", syntax.join(" "))
            }
        }
    }

    /// 文字列を取得
    fn get_string(&mut self) -> String {
        match self {
            Type::String(s) => s.to_string(),
            Type::Number(i) => i.to_string(),
            Type::Bool(b) => b.to_string(),
            Type::List(l) => Type::List(l.to_owned()).display(),
        }
    }

    /// 数値を取得
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

    /// 論理値を取得
    fn get_bool(&mut self) -> bool {
        match self {
            Type::String(s) => s.len() != 0,
            Type::Number(i) => *i != 0.0,
            Type::Bool(b) => *b,
            Type::List(l) => l.len() != 0,
        }
    }

    ///　リストを取得
    fn get_list(&mut self) -> Vec<Type> {
        match self {
            Type::String(s) => vec![Type::String(s.to_string())],
            Type::Number(i) => vec![Type::Number(*i)],
            Type::Bool(b) => vec![Type::Bool(*b)],
            Type::List(l) => l.to_vec(),
        }
    }
}

/// プログラム実行を管理
struct Executor {
    stack: Vec<Type>,              // スタック
    memory: HashMap<String, Type>, // 変数のメモリ領域
    mode: Mode,                    // 実行モード
}

impl Executor {
    /// コンストラクタ
    fn new(mode: Mode) -> Executor {
        Executor {
            stack: Vec::new(),
            memory: HashMap::new(),
            mode,
        }
    }

    /// ログ表示
    fn log_print(&mut self, msg: String) {
        if let Mode::Debug = self.mode {
            println!("{msg}");
        }
    }

    /// メモリを表示
    fn show_variables(&mut self) {
        self.log_print(format!(
            "メモリ内部の変数 {{ {} }}",
            self.memory
                .clone()
                .iter()
                .map(|(name, value)| { format!("'{name}': {}", value.display()) })
                .collect::<Vec<String>>()
                .join(", ")
        ));
    }

    /// 構文解析
    fn analyze_syntax(&mut self, code: String) -> Vec<String> {
        let mut syntax = Vec::new();
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

    /// プログラムを評価する
    fn evaluate_program(&mut self, code: String) {
        // トークンを整える
        let syntax: Vec<String> = self.analyze_syntax(code);

        for token in syntax {
            // スタック内部を表示する
            self.log_print(format!(
                "Stack〔 {} 〕 ←  {}",
                self.stack
                    .iter()
                    .map(|x| x.display())
                    .collect::<Vec<_>>()
                    .join(" | "),
                token
            ));

            // 数値に変換できたらスタックに積む
            if let Ok(i) = token.parse::<f64>() {
                self.stack.push(Type::Number(i));
                continue;
            }

            // 論理値をスタックに積む
            if token == "true" || token == "false" {
                self.stack.push(Type::Bool(token.parse().unwrap_or(true)));
                continue;
            }

            // 文字列をスタックに積む
            let chars: Vec<char> = token.chars().collect();
            if chars[0] == '(' || chars[chars.len() - 1] == ')' {
                self.stack
                    .push(Type::String(token[1..token.len() - 1].to_string()));
                continue;
            }

            // リストを処理
            if chars[0] == '[' || chars[chars.len() - 1] == ']' {
                let old_len = self.stack.len();
                let slice = &token[1..token.len() - 1];
                let token: Vec<_> = slice.split_whitespace().map(|x| x.to_string()).collect();
                self.evaluate_program(
                    token
                        .into_iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                );
                let mut list = Vec::new();
                for _ in old_len..self.stack.len() {
                    list.push(self.pop_stack());
                }
                list.reverse();
                self.stack.push(Type::List(list));
                continue;
            }

            // 変数を読み込む
            if let Some(i) = self.memory.get(&token) {
                self.stack.push(i.clone());
                continue;
            }

            // コメントを処理
            if token.contains("#") {
                self.log_print(format!("※ コメント「{}」", token.replace("#", "")));
                continue;
            }

            // コマンドを実行する
            self.execute_command(token);
        }

        // 実行後のスタックを表示
        self.log_print(format!(
            "Stack〔 {} 〕",
            self.stack
                .iter()
                .map(|x| x.display())
                .collect::<Vec<_>>()
                .join(" | "),
        ));
    }

    /// コマンドを実行する
    fn execute_command(&mut self, command: String) {
        match command.as_str() {
            // 足し算
            "add" => {
                let b = self.pop_stack().get_number();
                let a = self.pop_stack().get_number();
                self.stack.push(Type::Number(a + b));
            }

            // 文字列を回数分リピート
            "repeat" => {
                let count = self.pop_stack().get_number(); // 回数
                let text = self.pop_stack().get_string(); // 文字列
                self.stack.push(Type::String(text.repeat(count as usize)));
            }

            // AND論理演算
            "and" => {
                let b = self.pop_stack().get_bool();
                let a = self.pop_stack().get_bool();
                self.stack.push(Type::Bool(a && b));
            }

            // OR論理演算
            "or" => {
                let b = self.pop_stack().get_bool();
                let a = self.pop_stack().get_bool();
                self.stack.push(Type::Bool(a || b));
            }

            // NOT論理演算
            "not" => {
                let b = self.pop_stack().get_bool();
                self.stack.push(Type::Bool(!b));
            }

            // 標準入力
            "input" => {
                let prompt = self.pop_stack().get_string(); //プロンプト
                self.stack.push(Type::String(input(prompt.as_str())));
            }

            // 等しいか
            "equal" => {
                let b = self.pop_stack().get_string();
                let a = self.pop_stack().get_string();
                self.stack.push(Type::Bool(a == b));
            }

            // ファイル書き込み
            "write-file" => {
                let mut file =
                    File::create(self.pop_stack().get_string()).expect("Failed to create file");
                file.write_all(self.pop_stack().get_string().as_bytes())
                    .expect("Failed to write file");
            }

            // ファイル読み込み
            "read-file" => {
                let name = self.pop_stack().get_string();
                self.stack
                    .push(Type::String(get_file_contents(name).unwrap()));
            }

            // 未満か
            "less" => {
                let b = self.pop_stack().get_number();
                let a = self.pop_stack().get_number();
                self.stack.push(Type::Bool(a < b));
            }

            // プロセスを終了
            "exit" => {
                let status = self.pop_stack().get_number();
                std::process::exit(status as i32);
            }

            // 条件分岐
            "if" => {
                let condition = self.pop_stack().get_bool(); // 条件式
                let code_else = self.pop_stack().get_string(); // elseコード
                let code_if = self.pop_stack().get_string(); // ifコード
                if condition {
                    self.evaluate_program(code_if)
                } else {
                    self.evaluate_program(code_else)
                };
            }

            // 文字列を式として評価
            "eval" => {
                let code = self.pop_stack().get_string();
                self.evaluate_program(code)
            }

            // 条件が一致してる間ループ
            "while" => {
                let cond = self.pop_stack().get_string();
                let code = self.pop_stack().get_string();
                loop {
                    if {
                        self.evaluate_program(cond.clone());
                        !self.pop_stack().get_bool()
                    } {
                        break;
                    }
                    self.evaluate_program(code.clone());
                }
            }

            // イテレート
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

            // マッピング処理
            "map" => {
                let code = self.pop_stack().get_string();
                let vars = self.pop_stack().get_string();
                let list = self.pop_stack().get_list();

                let mut result_list = Vec::new(); // Create a new vector to store the results

                for x in list.iter() {
                    self.memory
                        .entry(vars.clone())
                        .and_modify(|value| *value = x.clone())
                        .or_insert(x.clone());

                    self.evaluate_program(code.clone());
                    result_list.push(self.pop_stack()); // Store the result in the new vector
                }

                self.stack.push(Type::List(result_list)); // Push the final result back onto the stack
            }

            // スタックの値をポップ
            "pop_stack" => {
                self.pop_stack();
            }

            // 文字列を結合
            "concat" => {
                let b = self.pop_stack().get_string();
                let a = self.pop_stack().get_string();
                self.stack.push(Type::String(a + &b));
            }

            // 文字列を分割
            "split" => {
                let key = self.pop_stack().get_string();
                let text = self.pop_stack().get_string();
                self.stack.push(Type::List(
                    text.split(&key)
                        .map(|x| Type::String(x.to_string()))
                        .collect::<Vec<Type>>(),
                ));
            }

            // リストを結合した文字列を生成
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

            // 文字列の置換
            "replace" => {
                let after = self.pop_stack().get_string();
                let before = self.pop_stack().get_string();
                let text = self.pop_stack().get_string();
                self.stack.push(Type::String(text.replace(&before, &after)))
            }

            // 引き算
            "sub" => {
                let b = self.pop_stack().get_number();
                let a = self.pop_stack().get_number();
                self.stack.push(Type::Number(a - b));
            }

            // 変数の定義
            "var" => {
                let name = self.pop_stack().get_string(); // 変数名
                let data = self.pop_stack(); // 値
                self.memory
                    .entry(name)
                    .and_modify(|value| *value = data.clone())
                    .or_insert(data);

                self.show_variables()
            }

            // 掛け算
            "mul" => {
                let b = self.pop_stack().get_number();
                let a = self.pop_stack().get_number();
                self.stack.push(Type::Number(a * b));
            }

            // 割り算
            "div" => {
                let b = self.pop_stack().get_number();
                let a = self.pop_stack().get_number();
                self.stack.push(Type::Number(a / b));
            }

            // 商の余り
            "mod" => {
                let b = self.pop_stack().get_number();
                let a = self.pop_stack().get_number();
                self.stack.push(Type::Number(a % b));
            }

            // べき乗
            "pow" => {
                let b = self.pop_stack().get_number();
                let a = self.pop_stack().get_number();
                self.stack.push(Type::Number(a.powf(b)));
            }

            // 四捨五入
            "round" => {
                let a = self.pop_stack().get_number();
                self.stack.push(Type::Number(a.round()));
            }

            // 表示出力
            "print" => {
                let a = self.pop_stack().get_string();
                if let Mode::Debug = self.mode {
                    println!("出力: {a}");
                } else {
                    println!("{a}");
                }
            }

            // 並び替え
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

            // データ型の取得
            "type" => {
                let result = match self.pop_stack() {
                    Type::Number(_) => "number",
                    Type::String(_) => "string",
                    Type::Bool(_) => "bool",
                    Type::List(_) => "list",
                }
                .to_string();
                self.stack.push(Type::String(result));
            }

            // シーケンス値を設定
            "set" => {
                let mut value = self.pop_stack();
                let index = self.pop_stack().get_number();
                let result: Type = match self.pop_stack() {
                    Type::String(s) => {
                        let mut text: Vec<_> = s.chars().collect();
                        text[index as usize] = value.get_string().chars().collect::<Vec<_>>()[0];
                        Type::String(
                            text.iter()
                                .map(|x| x.to_string())
                                .collect::<Vec<String>>()
                                .join(""),
                        )
                    }
                    Type::List(l) => {
                        let mut list = l;
                        list[index as usize] = value;
                        Type::List(list)
                    }
                    _ => {
                        self.log_print(format!("エラー! シーケンス型でのみ有効です"));
                        Type::List(vec![])
                    }
                };
                self.stack.push(result);
            }

            // メモリ開放
            "free" => {
                let name = self.pop_stack().get_string();
                self.memory.remove(name.as_str());
                self.show_variables();
            }

            // シーケンス値を削除
            "del" => {
                let index = self.pop_stack().get_number();
                let result = match self.pop_stack() {
                    Type::String(s) => {
                        let mut text = s;
                        text.remove(index as usize);
                        Type::String(text)
                    }
                    Type::List(l) => {
                        let mut list = l;
                        list.remove(index as usize);
                        Type::List(list)
                    }
                    _ => {
                        self.log_print(format!("エラー! シーケンス型でのみ有効です"));
                        Type::List(vec![])
                    }
                };
                self.stack.push(result);
            }

            // シーケンス値を取得
            "get" => {
                let index = self.pop_stack().get_number();
                let result: Type = match self.pop_stack() {
                    Type::String(s) => {
                        let text: Vec<_> = s.chars().collect();
                        Type::String(text[index as usize].to_string())
                    }
                    Type::List(l) => {
                        let list = l;
                        list[index as usize].clone()
                    }
                    _ => {
                        self.log_print(format!("エラー! シーケンス型でのみ有効です"));
                        Type::List(vec![])
                    }
                };
                self.stack.push(result);
            }

            //リストに追加
            "append" => {
                let mut data = self.pop_stack();
                let result: Type = match self.pop_stack() {
                    Type::List(l) => {
                        let mut l = l;
                        l.push(data);
                        Type::List(l)
                    }

                    Type::String(s) => {
                        let mut s = s;
                        s.push_str(data.get_string().as_str());
                        Type::String(s)
                    }
                    _ => {
                        self.log_print(format!("エラー! シーケンス型でのみ有効です"));
                        Type::List(vec![])
                    }
                };
                self.stack.push(result);
            }

            // シーケンスに挿入
            "insert" => {
                let mut data = self.pop_stack();
                let index = self.pop_stack().get_number();
                let result: Type = match self.pop_stack() {
                    Type::String(s) => {
                        let mut text = s;
                        text.insert_str(index as usize, data.get_string().as_str());
                        Type::String(text)
                    }
                    Type::List(l) => {
                        let mut list = l;
                        list.insert(index as usize, data);
                        Type::List(list)
                    }
                    _ => {
                        self.log_print(format!("エラー! シーケンス型でのみ有効です"));
                        Type::List(vec![])
                    }
                };
                self.stack.push(result);
            }

            // 含まれているか
            "find" => {
                let word = self.pop_stack().get_string();
                let text = self.pop_stack().get_string();
                self.stack.push(Type::Bool(text.contains(&word)))
            }

            // 範囲を生成
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

            // シーケンスの長さ
            "len" => {
                let data = self.pop_stack();
                self.stack.push(Type::Number(match data {
                    Type::List(l) => l.len() as f64,
                    Type::String(s) => s.chars().count() as f64,
                    _ => 1f64,
                }));
            }

            // 値のコピー
            "copy" => {
                let data = self.pop_stack();
                self.stack.push(data.clone());
                self.stack.push(data);
            }

            // コマンドとして認識されない場合は文字列とする
            _ => self.stack.push(Type::String(command)),
        }
    }

    /// スタックの値をポップする
    fn pop_stack(&mut self) -> Type {
        if let Some(value) = self.stack.pop() {
            value
        } else {
            self.log_print("エラー! スタックの値が足りません。デフォルト値を返します".to_string());
            Type::String("".to_string())
        }
    }
}

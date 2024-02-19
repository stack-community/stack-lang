use rand::seq::SliceRandom;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{self, Error, Read, Write};
use std::path::Path;
use std::thread;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn main() {
    // コマンドライン引数を読み込む
    let args = env::args().collect::<Vec<_>>();
    if args.len() > 2 {
        // ファイルを開く
        if let Ok(code) = get_file_contents(args[1].clone()) {
            // 実行モードを判定する
            if args[2].contains("-d") {
                let mut executor = Executor::new(Mode::Debug);
                executor.evaluate_program(code); //デバッグ実行
            } else {
                let mut executor = Executor::new(Mode::Script);
                executor.evaluate_program(code); // スクリプト実行
            }
        } else {
            println!("エラー! ファイルが見つかりません")
        }
    } else if args.len() > 1 {
        // ファイルを開く
        if let Ok(code) = get_file_contents(args[1].clone()) {
            let mut executor = Executor::new(Mode::Script); //デフォルト値はスクリプト実行
            executor.evaluate_program(code);
        } else {
            println!("エラー! ファイルが見つかりません")
        }
    } else {
        // タイトルを表示する
        println!("Stack プログラミング言語");
        let mut executor = Executor::new(Mode::Debug);
        // REPL実行
        loop {
            let mut code = String::new();
            loop {
                let inputed = input("> ");
                code += &format!("{inputed}\n");
                if inputed.is_empty() {
                    break;
                }
            }

            executor.evaluate_program(code)
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
    print!("{}", prompt);
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
            Type::String(s) => !s.is_empty(),
            Type::Number(i) => *i != 0.0,
            Type::Bool(b) => *b,
            Type::List(l) => !l.is_empty(),
        }
    }

    ///　リストを取得
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
        }
    }
}

/// プログラム実行を管理
#[derive(Clone, Debug)]
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
            print!("{msg}");
        }
    }

    /// メモリの変数を表示
    fn show_variables(&mut self) {
        self.log_print(format!(
            "メモリ内部の変数 {{ {} }}\n",
            self.memory
                .clone()
                .iter()
                .map(|(name, value)| { format!("'{name}': {}", value.display()) })
                .collect::<Vec<String>>()
                .join(", ")
        ));
    }

    // スタック内部を表示
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

    /// 構文解析
    fn analyze_syntax(&mut self, code: String) -> Vec<String> {
        // タブや改行・全角スペースを半角スペースに変換する
        let code = code.replace(['\n', '\t', '\r', '　'], " ");

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

    /// プログラムを評価する
    fn evaluate_program(&mut self, code: String) {
        // 構文解析してトークン列にする
        let syntax: Vec<String> = self.analyze_syntax(code);

        for token in syntax {
            self.show_stack(); // スタック内部を表示する
            self.log_print(format!(" ←  {}\n", token));

            // 加工用の文字ベクタ
            let chars: Vec<char> = token.chars().collect();

            if let Ok(i) = token.parse::<f64>() {
                // 数値に変換できたらスタックに積む
                self.stack.push(Type::Number(i));
            } else if token == "true" || token == "false" {
                // 論理値をスタックに積む
                self.stack.push(Type::Bool(token.parse().unwrap_or(true)));
            } else if chars[0] == '(' && chars[chars.len() - 1] == ')' {
                // 文字列をスタックに積む
                self.stack
                    .push(Type::String(token[1..token.len() - 1].to_string()));
            } else if chars[0] == '[' && chars[chars.len() - 1] == ']' {
                // リストをスタックに積む
                let old_len = self.stack.len(); // 本来のスタックの大きさ
                let slice = &token[1..token.len() - 1];
                let token: Vec<_> = slice.split_whitespace().map(|x| x.to_string()).collect();
                self.evaluate_program(
                    token
                        .into_iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                );
                // スタックの増加分をリストの要素にする
                let mut list = Vec::new();
                for _ in old_len..self.stack.len() {
                    list.push(self.pop_stack());
                }
                list.reverse(); // スタックから取り出したのは逆なので反転させる
                self.stack.push(Type::List(list));
            } else if let Some(i) = self.memory.get(&token) {
                // 変数を読み込む
                self.stack.push(i.clone());
            } else if token.contains('#') {
                // コメントを処理
                self.log_print(format!("※ コメント「{}」\n", token.replace('#', "")));
            } else {
                // コマンドを実行する
                self.execute_command(token);
            }
        }

        // 実行後のスタックを表示
        self.show_stack();
        self.log_print("\n".to_string());
    }

    /// コマンドを実行する
    fn execute_command(&mut self, command: String) {
        match command.as_str() {
            // 演算コマンド

            // 足し算
            "add" => {
                let b = self.pop_stack().get_number();
                let a = self.pop_stack().get_number();
                self.stack.push(Type::Number(a + b));
            }

            // 引き算
            "sub" => {
                let b = self.pop_stack().get_number();
                let a = self.pop_stack().get_number();
                self.stack.push(Type::Number(a - b));
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

            // 等しいか
            "equal" => {
                let b = self.pop_stack().get_string();
                let a = self.pop_stack().get_string();
                self.stack.push(Type::Bool(a == b));
            }

            // 未満か
            "less" => {
                let b = self.pop_stack().get_number();
                let a = self.pop_stack().get_number();
                self.stack.push(Type::Bool(a < b));
            }

            // ランダム値を取得
            "rand" => {
                let list = self.pop_stack().get_list();
                let result = match list.choose(&mut rand::thread_rng()) {
                    Some(i) => i.to_owned(),
                    None => Type::List(list),
                };
                self.stack.push(result);
            }

            // シーケンス値をシャッフル
            "shuffle" => {
                let mut list = self.pop_stack().get_list();
                list.shuffle(&mut rand::thread_rng());
                self.stack.push(Type::List(list));
            }

            // 文字列操作コマンド

            // 文字列を回数分リピート
            "repeat" => {
                let count = self.pop_stack().get_number(); // 回数
                let text = self.pop_stack().get_string(); // 文字列
                self.stack.push(Type::String(text.repeat(count as usize)));
            }

            // 数値からユニコード文字列を取得
            "decode" => {
                let code = self.pop_stack().get_number();
                let result = char::from_u32(code as u32);
                match result {
                    Some(c) => self.stack.push(Type::String(c.to_string())),
                    None => {
                        self.log_print("エラー! 数値デコードに失敗しました\n".to_string());
                        self.stack.push(Type::Number(code));
                    }
                }
            }

            // 文字列をUTF-8でエンコード
            "encode" => {
                let string = self.pop_stack().get_string();
                if let Some(first_char) = string.chars().next() {
                    self.stack.push(Type::Number((first_char as u32) as f64));
                } else {
                    self.log_print("エラー! 文字列のエンコードに失敗しました\n".to_string());
                    self.stack.push(Type::String(string))
                }
            }

            // 文字列を結合
            "concat" => {
                let b = self.pop_stack().get_string();
                let a = self.pop_stack().get_string();
                self.stack.push(Type::String(a + &b));
            }

            // 文字列の置換
            "replace" => {
                let after = self.pop_stack().get_string();
                let before = self.pop_stack().get_string();
                let text = self.pop_stack().get_string();
                self.stack.push(Type::String(text.replace(&before, &after)))
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

            // 含まれているか
            "find" => {
                let word = self.pop_stack().get_string();
                let text = self.pop_stack().get_string();
                self.stack.push(Type::Bool(text.contains(&word)))
            }

            // 正規表現で検索
            "regex" => {
                let pt = self.pop_stack().get_string();
                let patern: Regex = match Regex::new(pt.as_str()) {
                    Ok(i) => i,
                    Err(_) => {
                        self.log_print("エラー! 正規表現が不正です\n".to_string());
                        return;
                    }
                };

                let text = self.pop_stack().get_string();

                let mut list: Vec<Type> = Vec::new();
                for i in patern.captures_iter(text.as_str()) {
                    list.push(Type::String(i[0].to_string()))
                }
                self.stack.push(Type::List(list));
            }

            // 入出力コマンド

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
                match get_file_contents(name) {
                    Ok(s) => self.stack.push(Type::String(s)),
                    Err(e) => self.log_print(format!("エラー! {}\n", e)),
                };
            }

            // 標準入力
            "input" => {
                let prompt = self.pop_stack().get_string(); //プロンプト
                self.stack.push(Type::String(input(prompt.as_str())));
            }

            // 標準出力
            "print" => {
                let a = self.pop_stack().get_string();
                if let Mode::Debug = self.mode {
                    println!("[出力]: {a}");
                } else {
                    println!("{a}");
                }
            }

            // 制御コマンド

            // 文字列を式として評価
            "eval" => {
                let code = self.pop_stack().get_string();
                self.evaluate_program(code)
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

            // 条件が一致してる間ループ
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

            // スレッドを作成
            "thread" => {
                let code = self.pop_stack().get_string();
                let mut executor = self.clone();
                thread::spawn(move || executor.evaluate_program(code));
            }

            // プロセスを終了
            "exit" => {
                let status = self.pop_stack().get_number();
                std::process::exit(status as i32);
            }

            // リスト操作コマンド

            // リストの値を取得
            "get" => {
                let index = self.pop_stack().get_number() as usize;
                let list: Vec<Type> = self.pop_stack().get_list();
                if list.len() > index {
                    self.stack.push(list[index].clone());
                } else {
                    self.log_print("エラー! インデックス指定が範囲外です\n".to_string());
                    self.stack.push(Type::List(list));
                }
            }

            // リストの値を設定
            "set" => {
                let value = self.pop_stack();
                let index = self.pop_stack().get_number() as usize;
                let mut list: Vec<Type> = self.pop_stack().get_list();
                if list.len() > index {
                    list[index] = value;
                    self.stack.push(Type::List(list));
                } else {
                    self.log_print("エラー! インデックス指定が範囲外です\n".to_string());
                    self.stack.push(Type::List(list));
                }
            }

            // リストの値を削除
            "del" => {
                let index = self.pop_stack().get_number() as usize;
                let mut list = self.pop_stack().get_list();
                if list.len() > index {
                    list.remove(index);
                    self.stack.push(Type::List(list));
                } else {
                    self.log_print("エラー! インデックス指定が範囲外です\n".to_string());
                    self.stack.push(Type::List(list));
                }
            }

            // リストに値を追加
            "append" => {
                let data = self.pop_stack();
                let mut list = self.pop_stack().get_list();
                list.push(data);
                self.stack.push(Type::List(list));
            }

            // リストに挿入
            "insert" => {
                let data = self.pop_stack();
                let index = self.pop_stack().get_number();
                let mut list = self.pop_stack().get_list();
                list.insert(index as usize, data);
                self.stack.push(Type::List(list));
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

            // 反転
            "reverse" => {
                let mut list = self.pop_stack().get_list();
                list.reverse();
                self.stack.push(Type::List(list));
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

            // フィルタ処理
            "filter" => {
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
                    if self.pop_stack().get_bool() {
                        result_list.push(x.clone()); // Store the result in the new vector
                    }
                }

                self.stack.push(Type::List(result_list)); // Push the final result back onto the stack
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

            // リストの長さ
            "len" => {
                let data = self.pop_stack();
                self.stack.push(Type::Number(match data {
                    Type::List(l) => l.len() as f64,
                    Type::String(s) => s.chars().count() as f64,
                    _ => 1f64,
                }));
            }

            // メモリ管理コマンド

            // スタックの値をポップ
            "pop" => {
                self.pop_stack();
            }

            // スタックのサイズを取得
            "size-stack" => {
                let len: f64 = self.stack.len() as f64;
                self.stack.push(Type::Number(len));
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

            // 明示的なデータ型変換
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

            // メモリ情報を取得
            "mem" => {
                let mut list: Vec<Type> = Vec::new();
                for (name, _) in self.memory.clone() {
                    list.push(Type::String(name))
                }
                self.stack.push(Type::List(list))
            }

            // メモリ開放
            "free" => {
                let name = self.pop_stack().get_string();
                self.memory.remove(name.as_str());
                self.show_variables();
            }

            // 値のコピー
            "copy" => {
                let data = self.pop_stack();
                self.stack.push(data.clone());
                self.stack.push(data);
            }

            // 値の交換
            "swap" => {
                let b = self.pop_stack();
                let a = self.pop_stack();
                self.stack.push(b);
                self.stack.push(a);
            }

            // 時間処理

            // 現在時刻を取得
            "now-time" => {
                self.stack.push(Type::Number(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64(),
                ));
            }

            // 一定時間スリープ
            "sleep" => sleep(Duration::from_secs_f64(self.pop_stack().get_number())),

            // 外部連携処理

            //HTTPリクエストを送る
            "request" => {
                let url = self.pop_stack().get_string();
                self.stack.push(Type::String(
                    reqwest::blocking::get(url).unwrap().text().unwrap(),
                ));
            }

            // ファイルを開く
            "open" => {
                let _result = opener::open(self.pop_stack().get_string());
            }

            //カレントディレクトリを変更
            "cd" => {
                if let Err(err) = std::env::set_current_dir(self.pop_stack().get_string()) {
                    self.log_print(format!("エラー! {}\n", err));
                }
            }

            // カレントディレクトリを表示
            "pwd" => {
                if let Ok(current_dir) = std::env::current_dir() {
                    if let Some(path) = current_dir.to_str() {
                        self.stack.push(Type::String(String::from(path)));
                    }
                }
            }

            // ディレクトリを作成
            "mkdir" => {
                let name = self.pop_stack().get_string();
                if let Err(e) = fs::create_dir(name) {
                    self.log_print(format!("エラー! {e}\n"))
                }
            }

            // アイテムを削除
            "rm" => {
                let name = self.pop_stack().get_string();
                if Path::new(name.as_str()).is_dir() {
                    if let Err(e) = fs::remove_dir(name) {
                        self.log_print(format!("エラー! {e}\n"))
                    }
                } else if let Err(e) = fs::remove_file(name) {
                    self.log_print(format!("エラー! {e}\n"))
                }
            }

            // アイテム名を変更
            "rename" => {
                let to = self.pop_stack().get_string();
                let from = self.pop_stack().get_string();
                let _result = fs::rename(from, to);
            }

            // ファイル一覧のリスト
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

            // フォルダかどうか判定
            "folder" => {
                let path = self.pop_stack().get_string();
                let path = Path::new(path.as_str());
                self.stack.push(Type::Bool(path.is_dir()));
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
            self.log_print(
                "エラー! スタックの値が足りません。デフォルト値を返します\n".to_string(),
            );
            Type::String("".to_string())
        }
    }
}

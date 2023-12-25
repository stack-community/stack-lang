# Stack プログラミング言語

Stackはその名の通りスタック指向のプログラミング言語で、使いやすさと効率的な実行を目指して設計されています。

## 特徴

- スタックベースの実行モデル
- スペースで区切られたトークンを使ったシンプルな構文
- 加算、減算、乗算、除算などの基本的な算術演算をサポート
- 論理演算: AND・OR・NOT
- 文字列操作: 結合、繰り返し
- 条件文: If、while
- 変数の代入と利用

## サンプルコード

以下はStackでFizzBuzzを実装した例です:

```stack
1 (i) var
(
    ((FizzBuzz) print )
    i 15 mod 0 equal if

    (
        ((Fizz) print )
        i 3 mod 0 equal if 

        (
            ((Buzz) print )
            i 5 mod 0 equal if 

            (i print )
            i 5 mod 0 equal not if
        )
        i 3 mod 0 equal not if
    )
    i 15 mod 0 equal not if 
    i 1 add (i) var
)
100 repeat eval
```
Stackプログラミング言語のさらなるサンプルや機能についてはソースコードを自由にご覧ください！

## 貢献

貢献は大歓迎です！バグ報告や機能リクエストのためのissueを開いたり、Stackプログラミング言語の改善のためのプルリクエストを提出して頂けると幸いです。

## ライセンス

このプロジェクトは MITライセンスの下オープンソースで公開されています - 詳細は[LICENSE](LICENSE)ファイルをご覧ください。

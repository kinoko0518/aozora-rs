# 🦀aozora-rs
ツールとしてのブランドであるayameに対し、ライブラリとしてのブランドです。WASMとRustクレートをご用意しております。

## WASM
[こちら](https://github.com/kinoko0518/aozora-rs/releases/latest/download/aozora-rs-wasm-pkg.zip)から最新のビルドをダウンロードしてください。

## Rustクレート
青空文庫書式をRustで扱うためのCrateです。以下のクレート群から成り、これらの責任を持っています。
```
aozora-rs
┣ aozora-rs       ... いわゆるファザードクレートです
┣ aozora-rs-gaiji ... 外字をUTF-8に変換します
┣ aozora-rs-core  ... 青空文庫書式を中間表現に変換します
┣ aozora-rs-zip   ... 青空文庫で配布されている.zipを扱いやすい形に変換します
┣ aozora-rs-xhtml ... 中間表現をXHTMLに変換します
┣ aozora-rs-epub  ... XHTMLとメタデータからEPUBファイルを構築します
┗ aozora-rs-qa    ... aozora-rsの品質保証プログラムです
```
ご利用の際には、crate.ioにはアップロードされていないので、GitHubから追加してください。
```bash
cargo add aozora-rs --git https://github.com/kinoko0518/aozora-rs/
```
# 使い方
基本的な使い方としては文字列やzipなどから`AozoraDocument`を生成し、`epub()`や`xhtml()`などのメソッドを呼び出す形になります。

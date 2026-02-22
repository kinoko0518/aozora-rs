# 📖 文目プロジェクト

[![GitHub release (latest by date)](https://img.shields.io/github/v/release/kinoko0518/aozora-rs)](https://github.com/kinoko0518/aozora-rs/releases/latest)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

文目プロジェクトは、青空文庫書式の周辺ソフトウェアの総合的な近代化を目指すプロジェクトです。

用途に合わせて以下の3つの窓口をご用意しています。

* **📦 ライブラリ**: Rustクレート`aozora-rs`および、WASM向け`aozora-rs-wasm`
* **🖥️ アプリケーション**: CLIツール`ayame-cli`およびGUIアプリ`文目`
* **📚 ドキュメント**: プロジェクト公式ホームページ兼青空文庫書式ドキュメント、[あやめぐさ](https://kinoko0518.github.io/ayamegusa/)

WASMバイナリ、各種アプリケーションの配布、各バージョンの品質保証レポートは[最新のリリース](https://github.com/kinoko0518/aozora-rs/releases/latest)をご覧ください。

> [!WARNING]
> 文目プロジェクトの各プログラムが発生させた損害について、開発者は一切の責任を負うことができません。
> また、まだしばらくは頻繫に破壊的な変更が行われることが予想されます。
> 簡潔に言えば、まだ実用レベルまで枯れていないという段階です。

## 📂 プロジェクト構成

```
文目プロジェクト
├── ライブラリ
│   ├── aozora-rs
│   ├── aozora-rs-wasm
│   ├── aozora-rs-core
│   ├── aozora-rs-gaiji
│   ├── aozora-rs-xhtml
│   ├── aozora-rs-epub
│   └── aozora-rs-qa
├── アプリケーション
│   ├── ayame-core
│   ├── ayame-cli
│   └── 文目
└── ドキュメント
    └── あやめぐさ
```

## 📥 インストール方法

### アプリケーションとしての利用

[最新のリリース](https://www.google.com/url?sa=E&source=gmail&q=https://github.com/kinoko0518/aozora-rs/releases/latest)からOSに合わせたバイナリ（ayame-cli または 文目）をダウンロードしてください。
Rust環境がある場合は、リポジトリから直接ビルドしてインストールすることも可能です。

```bash
cargo install --git [https://github.com/kinoko0518/aozora-rs](https://github.com/kinoko0518/aozora-rs) ayame-cli
```

### ライブラリとしての利用

Rustプロジェクトで利用する場合は、`Cargo.toml` に以下を追加してください。

```toml
[dependencies]
aozora-rs = { git = "[https://github.com/kinoko0518/aozora-rs](https://github.com/kinoko0518/aozora-rs)" }
```

## 🦀 ライブラリ / aozora.rs

`aozora.rs` は既存のソリューションと比較して高い移植性、ネイティブ動作、軽量さを目指しており、コア部分（`aozora-rs-core`）においてはゼロコピー、純粋関数、Best Effortを実現しています。

パースは **トークン化 → 注記影響範囲決定 → 再トークン化** の順番で進行し、各段階は純粋関数を用いて変換されるため、任意の段階でご利用いただけます。
再トークン化された表現の直和（Retokenized）は中間表現として振る舞います。

* **外字の処理**: `aozora-rs-gaiji`
* **XHTMLの構築**: `aozora-rs-xhtml`
* **EPUBの構築**: `aozora-rs-epub`

ファサードクレートとして`aozora-rs`と`aozora-rs-wasm`を提供しており、品質保証レポートの自動生成は`aozora-rs-qa`が行います。

## ✨ アプリケーション / ayame-cli & 文目

### ayame-cli

青空文庫書式のテキストファイル（`.txt`）またはZIPファイル（`.zip`）からXHTML/EPUBを生成するCLIツールです。

```bash
ayame <COMMAND>
```

#### `xhtml` — XHTMLを生成

```bash
ayame xhtml [OPTIONS] <SOURCE>
```

| 引数 / オプション | 説明 |
| --- | --- |
| `<SOURCE>` | 入力ファイル（`.txt` または `.zip`） |
| `--utf8` | 入力がUTF-8の場合に指定（デフォルト: Shift-JIS） |
| `--merge` | 複数のXHTMLを `<hr>` 区切りで1ファイルに結合 |
| `-o`, `--output <DIR>` | 出力先ディレクトリ（デフォルト: カレントディレクトリ） |

#### `epub` — EPUBを生成

```bash
ayame epub [OPTIONS] <SOURCE>
```

| 引数 / オプション | 説明 |
| --- | --- |
| `<SOURCE>` | 入力ファイル（`.txt` または `.zip`） |
| `--utf8` | 入力がUTF-8の場合に指定（デフォルト: Shift-JIS） |
| `--horizontal` | 横書きで生成（デフォルト: 縦書き） |
| `--no-prelude` | 組み込みCSS `prelude` を適用しない（デフォルト: 適用） |
| `--no-miyabi` | 組み込みCSS `miyabi` を適用しない（デフォルト: 適用） |
| `--css <FILE>` | 追加で適用するCSSファイルのパス（複数指定可） |
| `-o`, `--output <DIR>` | 出力先ディレクトリ（デフォルト: カレントディレクトリ） |

### 文目

`ayame-cli` のGUI版です。CLIと同等の機能を、直感的なインターフェースでご利用いただけます。

## 📖 ドキュメンテーション / あやめぐさ

プロジェクトの詳細や青空文庫書式の仕様については、公式ドキュメントをご覧ください。
👉 [あやめぐさ にアクセス](https://www.google.com/url?sa=E&source=gmail&q=https://kinoko0518.github.io/ayamegusa/)

## 📄 ライセンス

本プロジェクトは [Apache License 2.0](https://www.google.com/search?q=LICENSE) のもとで公開されています。

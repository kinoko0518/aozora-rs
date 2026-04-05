# 🦀 aozora.rs

[![GitHub release (latest by date)](https://img.shields.io/github/v/release/kinoko0518/aozora-rs)](https://github.com/kinoko0518/aozora-rs/releases/latest)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

Rust実装の青空文庫書式パーサーです。組み込みが容易・ネイティブ動作なAozoraEpub3を目指して開発しています。GUI、CLI、Web版を公開しており、Web版は[こちらのリンク](https://kinoko0518.github.io/aozora-rs/)から今すぐ試せます！ライブラリとしてはRustクレートとしてはもちろん、WASMパッケージとしてもご利用いただけます。低レベルAPIはaozora-rsを、高レベルAPIはayame-coreをご利用ください。

## GUI版

<img src="./docs/images/screenshots/gui.png" width="800px">

## インストール

**cargoからインストール**

```bash
cargo install --git https://github.com/kinoko0518/aozora-rs ayame-app
```

**コンパイル済みバイナリをダウンロード**

<div align="center">
    <a href="https://github.com/kinoko0518/aozora-rs/releases/latest/download/ayame-app-windows-x86_64.exe">
        <picture>
            <source media="(prefers-color-scheme: dark)" srcset="./docs/images/icons/windows-in-dark.svg">
            <source media="(prefers-color-scheme: light)" srcset="./docs/images/icons/windows-in-light.svg">
            <img alt="Windows" src="./docs/images/icons/windows-in-dark.svg">
        </picture>
    </a>
    <a href="https://github.com/kinoko0518/aozora-rs/releases/latest/download/ayame-app-linux-x86_64">
        <picture>
            <source media="(prefers-color-scheme: dark)" srcset="./docs/images/icons/linux-in-dark.svg">
            <source media="(prefers-color-scheme: light)" srcset="./docs/images/icons/linux-in-light.svg">
            <img alt="Linux" src="./docs/images/icons/linux-in-dark.svg">
        </picture>
    </a>
    <a href="https://github.com/kinoko0518/aozora-rs/releases/latest/download/ayame-app-macos-aarch64">
        <picture>
            <source media="(prefers-color-scheme: dark)" srcset="./docs/images/icons/mac-in-dark.svg">
            <source media="(prefers-color-scheme: light)" srcset="./docs/images/icons/mac-in-light.svg">
            <img alt="macOS" src="./docs/images/icons/mac-in-dark.svg">
        </picture>
    </a>
</div>

## CLI版

<img src="./docs/images/screenshots/cli.png" width="800px">

### インストール

**cargoからインストール**

```bash
cargo install --git https://github.com/kinoko0518/aozora-rs ayame-cli
```

**コンパイル済みバイナリをダウンロード**

<div align="center">
    <a href="https://github.com/kinoko0518/aozora-rs/releases/latest/download/ayame-cli-windows-x86_64.exe">
        <picture>
            <source media="(prefers-color-scheme: dark)" srcset="./docs/images/icons/windows-in-dark.svg">
            <source media="(prefers-color-scheme: light)" srcset="./docs/images/icons/windows-in-light.svg">
            <img alt="Windows" src="./docs/images/icons/windows-in-dark.svg">
        </picture>
    </a>
    <a href="https://github.com/kinoko0518/aozora-rs/releases/latest/download/ayame-cli-linux-x86_64">
        <picture>
            <source media="(prefers-color-scheme: dark)" srcset="./docs/images/icons/linux-in-dark.svg">
            <source media="(prefers-color-scheme: light)" srcset="./docs/images/icons/linux-in-light.svg">
            <img alt="Linux" src="./docs/images/icons/linux-in-dark.svg">
        </picture>
    </a>
    <a href="https://github.com/kinoko0518/aozora-rs/releases/latest/download/ayame-cli-macos-aarch64">
        <picture>
            <source media="(prefers-color-scheme: dark)" srcset="./docs/images/icons/mac-in-dark.svg">
            <source media="(prefers-color-scheme: light)" srcset="./docs/images/icons/mac-in-light.svg">
            <img alt="macOS" src="./docs/images/icons/mac-in-dark.svg">
        </picture>
    </a>
</div>

### 使い方

```bash
ayame <COMMAND> <SOURCE> [OPTIONS]
```
`<SOURCE>`には.zip、または.txtファイルのパスを受け付けます。

| `<Command>` | 出力フォーマット |
| --- | --- |
| epub | EPUB 3 |
| xhtml | XHTML |

| `[OPTIONS]` | 効果 |
| --- | --- |
| --utf8 | UTF-8としてファイルを解釈します。特に指定しない限りはShift-JISです。 |
| --horizontal | 横書きになります。特に指定しない限りは縦書きです。 | 
| --no-prelude | 要素を正しく表示するための組み込みCSSを無効化します。 |
| --no-miyabi | 美しく表示するための組み込みCSSを無効化します。 |
| --css <FILE_PATH> | 追加のカスタムCSSを適用します。複数回使用できます。 |
| -o, --output <DIR_PATH> | 出力先のディレクトリを指定します。 |

## ライブラリ利用

> [!WARNING]
> クレートとしてのaozora-rsは近日中のAPIの整理が予定されています。

**aozora-rs**
```bash
cargo add aozora-rs --git https://github.com/kinoko0518/aozora-rs/
```

**ayame-rs**
```bash
cargo add ayame-core --git https://github.com/kinoko0518/aozora-rs/
```

**WASM**

[こちら](https://github.com/kinoko0518/aozora-rs/releases/latest/download/aozora-rs-wasm-pkg.zip)から最新のビルドをダウンロードしてください。

## 利用について
本プロジェクトは[Apache License 2.0]("./LICENCE")を採用しています。使用報告などは特に必要ありませんが、作者の[Twitter](https://x.com/6osciola/)までご一報いただけると喜びます。

## Star履歴
[![Star History Chart](https://api.star-history.com/svg?repos=kinoko0518/aozora-rs&type=date&legend=top-left)](https://www.star-history.com/?spm=a2c6h.12873639.article-detail.7.7b9d7fabjNxTRk#kinoko0518/aozora-rs&type=date&legend=top-left)

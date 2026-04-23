#import "./prelude.typ"
#import "./doorpage.typ"
#import "./util.typ": *

#show: document => prelude.prelude(document)

#doorpage.doorpage(
  [Rustを用いた高速青空文庫書式パーサーの開発],
  (
    ([Kinokov#footnote[https://github.com/kinoko0518/]], [Main Contributor]),
    ([Gramme#footnote[https://github.com/gramme-linkcom]], [Contributor]),
  ),
  [
    既存のパーサーはインタプリタ言語やVMを必要とする言語で書かれていたり、一つのアプリケーションとして閉じているなどライブラリとしての利用に適さないものが多かった。
    そこで、コンパイル言語であるRustを用いて独自の三層構造からなる青空文庫書式パーサーを構築した結果、ドストエフスキー著『罪と罰』を16.1ミリ秒で中間表現へ変換することができた。
  ],
  ([青空文庫], [パーサー], [ゼロコピー設計]),
)

#h(20mm)

#show: document => prelude.main(document)

#show: columns.with(2, gutter: 20pt)

= はじめに
== 背景
青空文庫書式は、主に日本で著作権が切れた文学作品を公開しているサイト、青空文庫（ https://www.aozora.gr.jp/ ）で公開されている作品に用いられているマークアップ言語であるが、その和文小説に特化した性質から利用はそれだけに留まらず、国内の小説投稿サイトなどでも青空文庫書式や青空文庫書式ライクなマークアップ言語が使用されている。

また、青空文庫は近年ではWikipediaと並んで国産LLMのための高品質なコーパスの作成の材料としてもよく用いられている。

== 課題
現状、もっとも主要な青空文庫書式パーサーはhmdevによるAozoraEpub3#footnote[https://github.com/hmdev/AozoraEpub3/]というパーサーだが、最後のコミットが2016年6月17日であり、実質的に開発が停止している。

フォークとして急急如律令による改造版AozoraEpub3#footnote[https://github.com/kyukyunyorituryo/AozoraEpub3]、Harusame64によるJRE 21対応版のAozoraEpub3-JDK21#footnote[https://github.com/AozoraEpub3-JDK21/AozoraEpub3-JDK21]などが存在しているが、いずれも共通してアプリケーションとして完結しておりライブラリとしての利用が困難であること、GPLライセンスでありクローズドソースのソフトウェアでの使用が困難であること、Java製であるためVMのオーバーヘッドやポータビリティの問題などがある。

aozora2html#footnote[https://github.com/aozorahack/aozora2html]はHTMLへの変換に特化しており、Ruby製であるため、JITは存在するもののやはりVMのオーバーヘッドがある。なお、aozora2htmlはCC0で公開されている。

aozora-parser.js#footnote[https://github.com/aozorahack/aozora-parser.js]は抽象構文木を構築することに特化しており、JavaScript製であるため、ネイティブアプリへの組み込みには向かない。

Go言語ではMasahiro Yamadaによる実装のAozoraConvert#footnote[https://github.com/adamay909/AozoraConvert]）が存在するが、青空文庫に収録されている全作品を入力するテストを行ったところ、出力されたEPUBの数が12698に対し入力の数が17808と、28.695%のケースで変換が失敗している。また、こちらもAGPLライセンスを採用しており、クローズドソースのソフトウェアでの使用が困難である。

以上の先行アプローチから、既存の青空文庫書式パーサーによく見られる問題として下記の課題を指摘する。

- ライセンスが寛容ではない。
- コードが老朽化している。
- インタプリタやVMを要求し、ポータビリティが低い。
- アプリケーションとして閉じており、ライブラリとしての利用が想定されていない。
- 出力がASTに留まるため、実用には別ツールの導入が必要になる。

== 目的と貢献
本研究では上記の課題を解決し、現代のソフトウェア開発において実用的かつ組み込みやすい青空文庫書式パーサーを提供することを目的とする。そのために、以下の要件を満たすパーサーの設計および実装を行った。

- Pure-Rustで動作し、メモリアロケーションを極限まで削減する設計でリアルタイムプレビュー水準の速度を実現する。
- .txtと.zipの両方の入力から出力フォーマットが限定されない中間表現を生成でき、また、それ単体でXHTML、EPUBへの変換を行うことができる。
- アプリケーションとして閉じていないことに加え、ライブラリとしての抽象度を高くし、中身を知らなくても利用することができる。
- CLI、GUI、WASMなど幅広いインターフェースを採用し、高いユーザビリティを実現する。
- Apache 2.0ライセンスを採用し、クローズドソースのソフトウェアでも青空文庫書式を利用しやすい状況を作る。

= 設計と実装
本パーサーの各処理段階における入出力の型シグネチャを定式化すると、以下の通りに表すことができる。Tはテキスト、Dは依存関係にあるすべての画像データとその名前のHashMapの直積、Mはメタデータ、Iは中間表現、Oは失敗の可能性を表すモナドである。

$ f: "*.zip" arrow.r O(M × T × D) $
$ g: "*.txt" arrow.r O(M) × T $
$ h: T arrow.r O(I) $
$ j: M × I arrow.r "*.xhtml" $
$ k: M × I × D arrow.r O("*.epub") $

このとき、関数hは以下の通りにさらに分解することができる。このとき、Sは装飾の影響範囲を表す意味付きバイトスライスである。

なお、以下では$h_"tokenize"$をTokenize層、$h_"scopenize"$をScopenize層、$h_"retokenize"$をRetokenize層と呼称し、それぞれトークン化、装飾範囲の確定、HTMLライクな中間表現への変換の責務を持つと定める。

$ h: h_"retokenize" compose h_"scopenize" compose h_"tokenize" $
$ h_"tokenize": T arrow.r O(T_"tokenized") $
$ h_"scopenize": T_"tokenized" arrow.r O(T_"scopenized" × S) $
$ h_"retokenize": T_"scopenized" × S arrow.r O(I) $

== Tokenize層
まず、本パーサーにおけるトークンの種類を定義する。注記のパースについては、詳細を後述する。
- Note: ［＃……］で表現される注記に対応。
- Ruby: 《……》で表現されるルビに対応。
- RubyDelimiter: ルビの範囲指定に用いる記号、'｜'（U+FF5C）に対応。
- Odoriji: 踊り字を表現する記号（／＼、または／″＼）に対応
- Br: 改行に対応。
- Text: テキストのスライスに対応。
Text以外にマッチするパーサーをまとめてSpecialとし、Specialにマッチしうる記号が出現するまで読み飛ばし、Specialにマッチしたらそこに到達するまでの文字列スライスをTextトークンとして追加してからSpecialを追加、最後に長さ0の文字列スライスを無視する方式でTokenize層を実装した。特筆すべき点として、Textトークンはオリジナルのテキストのスライスのみを持つため、どんなに長い文字列でも128バイト（32ビット環境では64バイト）で表すことができる。

たとえば、宮沢賢治著『屈折率』を例にすると、このようにトークン化が行われる。

*トークン化前:*

#let work(content) = block(fill: luma(230), inset: 7mm)[
  #set par(first-line-indent: 0em)
  #set text(size: 0.72em)

  #content
]

#let kusseturitu_original = work[
  ［＃２字下げ］屈折率［＃「屈折率」は中見出し］


  七つ森のこつちのひとつが

  水の中よりもつと明るく

  そしてたいへん巨きいのに

  わたくしはでこぼこ凍つたみちをふみ

  このでこぼこの雪をふみ

  向ふの縮れた亜鉛《あえん》の雲へ

  陰気な郵便｜脚夫《きやくふ》のやうに

  　　（またアラツデイン　洋燈《ラムプ》とり）

  急がなければならないのか

  ［＃地付き］（一九二二、一、六）
]
#kusseturitu_original

*トークン化後:*

#let kusseturitu_tokenized = work[
  （行頭型注記・字下げ（２））（テキスト・"屈折率"）（前方参照型・中見出し（"屈折率"））（テキスト・"七つ森のこつちのひとつが"）（改行）（テキスト・"水の中よりもつと明るく"）（改行）（テキスト・"そしてたいへん巨きいのに"）（改行）（テキスト・"わたくしはでこぼこ凍つたみちをふみ"）（改行）（テキスト・"このでこぼこの雪をふみ"）（改行）（テキスト・"向ふの縮れた亜鉛"）（ルビ・"あえん"）（テキスト・"の雲へ"）（改行）（テキスト・"陰気な郵便"）（ルビデリミタ）（テキスト・"脚夫"）（ルビ・"きやくふ"）（テキスト・"のやうに"）（テキスト・"　　（またアラツデイン　洋燈"）（ルビ・"ラムプ"）（テキスト・"とり）"）（改行）（テキスト・"急がなければならないのか"）（改行）（行頭型注記・地付き）（テキスト・"（一九二二、一、六）"）
]
#kusseturitu_tokenized

なお、本パーサーにおいて、トークン化にはパーサーコンビネータであるwinnowを利用している。

=== 注記の分類とパース
青空文庫の注記について、青空文庫は公式には分類を行っていないが、独自に挙動に基づいて分類を行った。
1. 前方参照型: ［＃「XXXX」はYYYY］の形式でYYYYが影響する文字列をXXXXで表現する。XXXXは注記の直前になければならない。
2. 行内挟み込み型: ［＃XXXX］［＃XXXX終わり］のペアによって挟みこんんだ範囲で注記の影響範囲を指定する。行をまたいで注記を適用することはできない。
3. 複数行挟み込み型: 独立行として表記された［＃ここからXXXX］［＃ここでXXXX終わり］で複数行を挟みこみ、挟みこんだ行全体に注記を適用させる。
4. 行頭型: 行頭に配置され、その行の終わりまで注記を影響させる。
5. 単一表現型: 画像、訓点など、それ単体が見た目を持つ要素に変換される。

== Scopenize層
この層では、Tokenize層で切り出した注記、およびルビの影響範囲を確定し、ここではスコープを表現する注記の影響範囲を表す意味付きバイトスライスと、以下のように定義される平坦トークンの直積に変換する。

- Text: テキストに対応。
- Break: 改行のほか、Tokenize層の段階では注記として扱われていた改ページや改段などを含む。
- Odoriji: 踊り字に対応。
- Kunten: 訓点に対応。
- Okurigana: 漢文における送り仮名に対応。
- Figure: 画像の挿入指示に対応。

たとえば、宮沢賢治著『屈折率』を例にすると、このようにスコープ化が行われる。

*スコープ化前:*
#kusseturitu_tokenized

*スコープ化後:*
#let scope(colour, info, content) = box(fill: colour, inset: 0.3em)[
  #content#text(size: 5pt)[（#info）]
]
#let scope_colours = (rgb(255, 255, 0, 255), rgb(0, 255, 255, 255))
#work[
  #scope(scope_colours.at(0), [字下げ（2）])[#scope(scope_colours.at(1), [中見出し])[屈折率]]

  七つ森のこつちのひとつが

  水の中よりもつと明るく

  そしてたいへん巨きいのに

  わたくしはでこぼこ凍つたみちをふみ

  このでこぼこの雪をふみ

  向ふの縮れた#scope(scope_colours.at(0), [ルビ・”あえん”])[亜鉛]の雲へ

  陰気な郵便#scope(scope_colours.at(0), [ルビ・”きやくふ”])[脚夫]のやうに

  　　（またアラツデイン　#scope(scope_colours.at(0), [ルビ・”ラムプ”])[洋燈]とり）

  急がなければならないのか

  #scope(scope_colours.at(0), [地付き])[（一九二二、一、六）]
]

=== アルゴリズム
- *ルビ:* Textトークンが次のトークンをpeekし、ルビであった場合、自身の末尾から漢字であるかを判定する関数が真を返し続ける範囲を影響範囲として確定する。また、ルビデリミタが出現した場合、次のトークンがTextトークン、次の次がルビであることを期待し、次の次のルビを次のトークンの範囲に適用する。
- *前方参照型注記:* Textトークンが次のトークンをpeekし、前方参照型であった場合、前方参照を行っている文字列と自身の末尾の一致を確認し、影響範囲を確定する。末尾が一致しなかった場合、前方参照型注記は消費され、エラーを蓄積する。
- *行内挟み込み型:* 行内挟み込み型の開始注記が出現したらスタックに積み、対応する終了注記が出現したら影響範囲を確定する。改行が出現したらスタックに積まれた注記をすべて閉じ、エラーを蓄積する。
- *複数行挟み込み型:* 複数行挟み込み型の開始注記が出現したらスタックに積み、対応する終了注記が出現したら影響範囲を確定する。Scopenizeが終了したとき、スタックにまだ要素が残っていればすべて閉じ、エラーを蓄積する。
- *行頭型:* スタックに蓄積し、改行が出たら注記の影響範囲を確定する。
- *単一表現型:* 平坦トークンに変換を行う。

== Retokenize層
Retokenize層では、受け取ったスコープと平坦トークンからHTMLライクな開始タグ・要素・終了タグからなる中間表現に変換する責務を負う。Retokenizerにおいて、考慮しなければならない状態はトークンを主語に考えると以下の8つである。

1. トークンの開始地点より前から開始されている注記が存在している。
2. トークンの開始地点で開始される注記が存在している。
3. トークンの途中で開始される注記が存在している。
4. トークンとトークンの隙間で注記が開始されている。
5. トークンの終了時点で終了されない注記が存在している。
6. トークンの途中で終了する注記が存在している。
7. トークンの開始地点で終了される注記が存在している。
8. トークンとトークンの隙間で注記が終了されている。

目的の達成のため、以下のアルゴリズムで目的を達成する。

1. トークンの開始地点と終了地点、注記の開始地点と終了地点を抽出する。
2. トークンと注記それぞれについて発生位置と開始・終了イベントの直積へと変換し、発生位置が早い順にソートする。このとき、開始イベントは開始されたトークン、スコープの情報を持っている。
3. スコープ開始・終了を受け取ったとき、閉じられていないトークンを途中で切り、開始・終了タグを積み、閉じられていないトークンを再開する。
4. トークン開始を受け取ったとき、スタックに閉じられていないトークンとして積む。
5. トークン終了を受け取ったとき、スタックの一番上のトークンを確定して積む。

= 評価
== 実験環境
#{
  let env = json("../../aozora-rs/aozora-rs-qa/result/enviroment.json")

  figure(
    caption: [実験に用いたコンピュータの性能],
    table(
      columns: (auto, 1fr),
      [OS情報], [#env.os_name #env.os_version],
      [カーネル情報], [#env.kernel],
      [CPUアーキテクチャ], [#env.architecture],
      [CPU], [#env.cpu_name],
      [主記憶装置], [#sign-digits(env.memory_size / calc.pow(1024, 3), 4)GiB],
      [rustcバージョン], [#env.rustc_version]
    )
  )
}

== 実験結果
性能を評価するため、以下の項目を計測した。なお、ここで示した結果はGitのコミットID、#read("../../.git/refs/heads/MAIN")に対応している。

以下、特段断りがない限り全量は青空文庫に収録されている全作品を指す。なお、解析はマルチスレッドで実行しているため、スレッド内で計測した処理時間の合計は実際の全体の処理時間よりも長くなる。

よって、合計処理時間はすべてのスレッドでの処理時間の計測結果を結合したもの、総合処理時間は解析開始から全量のEPUBへの変換と書き出しが終わるまでの時間を指すと区別する。

#let qa_summary = json("../../aozora-rs/aozora-rs-qa/result/summary.json");
#let total_megabyte = qa_summary.total_bytes / 1024 / 1024
#{
  figure(
    caption: [実験結果],
    table(
      columns: (auto, 1fr),
      [合計処理時間], [#sign-digits(rustd(qa_summary.duration_every_thread_total), 5)秒],
      [総合処理時間], [#sign-digits(rustd(qa_summary.duration_total), 5)秒],
      [合計入力サイズ], [#sign-digits(total_megabyte, 4)MB],
      [合計入力文字数], [#qa_summary.total_wordcount;文字],
      [作品数], [#qa_summary.total_works;点],
      [変換成功], [#qa_summary.total_succeed;点],
      [変換失敗], [#qa_summary.total_failed;点],
      [スコープ化エラー], [#qa_summary.scopenize_warning_total;件],
      [再トークン化エラー], [#qa_summary.retokenize_warning_total;件],
    )
  )
}

結果から、秒間スループットは合計処理時間対ファイルサイズでは#sign-digits(total_megabyte / rustd(qa_summary.duration_every_thread_total), 4)MB/s、総合処理時間対ファイルサイズでは#sign-digits(total_megabyte / rustd(qa_summary.duration_total), 4)MB/sとなった。また、致命的なエラーを発生させずパースを完遂したパターンは全体の#sign-digits(qa_summary.total_succeed / qa_summary.total_works * 100, 4)%となった。

また、Y軸処理時間に対し、それぞれX軸文字数、トークン数、トークンのうち注記とルビの登場回数の合計（以降、装飾数）としたプロット図を描画したとき、処理時間はこのようにスケールした。

#figure(
  caption: [文字数に対する処理時間のスケール],
  image("../../aozora-rs/aozora-rs-qa/result/wordcount_vs_duration.png")
)

#figure(
  caption: [トークン数に対する処理時間のスケール],
  image("../../aozora-rs/aozora-rs-qa/result/tokencount_vs_duration.png")
)

#figure(
  caption: [装飾数に対する処理時間のスケール],
  image("../../aozora-rs/aozora-rs-qa/result/notecount_vs_duration.png")
)

= 考察

# aozora-rs-core
[`String`]を受け取り、HTMLライクな中間表現である[`Retokenized`]の[`Vec`]を返します。

低レイヤなぶん`aozora-rs`よりも細かな制御が可能ですが、`aozora-rs-core`のAPIは予告なく変更される可能性があるため、可能な限り`aozora-rs`を用いて実装することを推奨します。

外字処理、および踊り字の変換は`aozora-rs-gaiji`に委託しているので、たとえばShift-JISの外字入りテキストを変換したい際は`encoding_rs`などでShift-JISをUTF-8に変換し、[`utf8tify_all_gaiji`]で外字と踊り字を変換してから[`aozora-rs-core`]にテキストを渡してください。

## アーキテクチャ
- [Tokenize](./docs/tokenize.md) … 青空文庫書式で記述されたテキストをトークン化します。
- [Scopenize](./docs/scopenize.md) … トークン列を受け取り、装飾範囲の影響範囲をスコープとして確定、非装飾トークンを平坦トークンとして次の層に渡します。
- [Retokenize](./docs/retokenize.md) … スコープと平坦トークンの直積を再びHTMLライクなフラット表現に変換します。

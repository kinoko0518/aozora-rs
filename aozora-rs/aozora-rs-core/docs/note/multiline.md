## 複数行挟み込み型
以下のような形式で記述し、挟み込んだ行に装飾を施します。
```aozorabunko
［＃ここから……］
一行目
二行目
……
［＃ここまで……］
```
### 開始
| パターン | 参照 | 表記ゆれ | 効果 |
| --- | --- | --- | --- |
| ここからN字下げ | [青空文庫公式ドキュメント](https://www.aozora.gr.jp/annotation/layout_2.html#jisage) | --- | パラグラフ単位でN文字分下げます。 |
| ここからN字下げ、折り返してM字下げ | [青空文庫公式ドキュメント](https://www.aozora.gr.jp/annotation/layout_2.html#ototsu) | --- | パラグラフ単位でN文字下げ、改行したあとM字下げします。 |
| 改行天付き、折り返してM字下げ | [青空文庫公式ドキュメント](https://www.aozora.gr.jp/annotation/layout_2.html#ototsu) | --- | ここからN字下げ、折り返してM字下げのNを0として省略したパターンです。 |
| ここから地付き | [青空文庫公式ドキュメント](https://www.aozora.gr.jp/annotation/layout_2.html#chitsuki) | --- | パラグラフを縦書きでは下寄せ、横書きでは左寄せします。 |
| ここから地からN字上げ | [青空文庫公式ドキュメント](https://www.aozora.gr.jp/annotation/layout_2.html#chiyose) | --- | パラグラフを縦書きでは下寄せ、横書きでは左寄せし、N字上げます。 |
| ここからN段階小さな文字 | [青空文庫公式ドキュメント](https://www.aozora.gr.jp/annotation/etc.html#moji_size) | --- | 文字を小さく描画します。 |
| ここからN段階大きな文字 | [青空文庫公式ドキュメント](https://www.aozora.gr.jp/annotation/etc.html#moji_size) | --- | 文字を大きく描画します。 |
| ここからN字詰め | [青空文庫公式ドキュメント](https://www.aozora.gr.jp/annotation/etc.html#jizume) | --- | 一行の長さをN字にします。 |

### 終了
| パターン | 参照 | 表記ゆれ | 効果 |
| --- | --- | --- | --- |
| ここで字下げ終わり | [青空文庫公式ドキュメント](https://www.aozora.gr.jp/annotation/layout_2.html#jisage) | ここで字下げおわり | 「ここからN字下げ」「ここからN字下げ、折り返してM字下げ」「改行天付き、折り返してM字下げ」を閉じます。 |
| ここで字上げ終わり | [青空文庫公式ドキュメント](https://www.aozora.gr.jp/annotation/layout_2.html#chiyose) | ここで字上げおわり | 「ここから地からN字上げ」を閉じます。 |
| ここで地付き終わり | [青空文庫公式ドキュメント](https://www.aozora.gr.jp/annotation/layout_2.html#chitsuki) | ここで地付きおわり | 「ここから地付き」を閉じます。 | 
| ここで小さな文字終わり | [青空文庫公式ドキュメント](https://www.aozora.gr.jp/annotation/etc.html#moji_size) | ここで小さな文字おわり | 「ここからN段階小さな文字」を閉じます。 |
| ここで大きな文字終わり | [青空文庫公式ドキュメント](https://www.aozora.gr.jp/annotation/etc.html#moji_size) | ここで大きな文字おわり | 「ここからN段階大きな文字」を閉じます。 |
| ここで字詰め終わり | [青空文庫公式ドキュメント](https://www.aozora.gr.jp/annotation/etc.html#jizume) | ここで字詰めおわり | 「ここからN字詰め」を閉じます。 |

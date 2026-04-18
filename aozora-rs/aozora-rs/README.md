# aozora-rs
aozora-rs-*のファザードクレートです。`String`または`impl Read + Seek`を入力すると`AozoraHyle`が生成され、そこからXHTMLとEPUB、メタデータなどを生成可能になります。

外字の考慮が必要な場合は与える文字列に先に`utf8ify_all_gaiji`を実行してください。

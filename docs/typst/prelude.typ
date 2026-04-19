#let gothic = ("Harano Aji Gothic")
#let mincho = ("Harano Aji Mincho")
#let english = ("CMU Serif")
#let mathf = ("Latin Modern Math")
#let rawf = ("Noto Mono for Powerline")

#let prelude(body) = {
  // 言語を設定
  set text(
    font: mincho,
    lang: "ja"
  )
  // それぞれのheadingにスタイルを適用
  set heading(
    numbering: "1.1.",
  )
  body
}

#let main(body) = {
  // 字下げを行う
  set par(
    first-line-indent: (
      all: true,
      amount: 1em,
    )
  )
  // 表，ソースコードならキャプションを上に表示
  show figure.where(
    kind: table
  ): set figure.caption(position: top)
  show figure.where(
    kind: raw
  ): set figure.caption(position: top)
  show figure.where(kind: raw): set figure(supplement: "ソースコード")
  body
}
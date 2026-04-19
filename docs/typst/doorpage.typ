#let doorpage(
  title, authors,
  abstract, keywords
) = {
  align(center, {
    text(size: 20pt)[
      *#{title}*
    ]
    table(
      columns: 5,
      stroke: none,
      align: bottom,
      ..for (author, role) in authors {
        (author, text(size: 7pt)[#role,], h(7mm))
      }
    )
    align(center)[
      #block(
        width: 90%,
        align(left)[
          *Abstract:* #abstract
          
          *Keywords:* #keywords.join(", ")
        ]
      )
    ]
  })
}
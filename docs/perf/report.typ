#set text(font: "Cantarell")
#show link: underline
#let figure = figure.with(
  kind: "image",
  supplement: none,
) // disable prefix in captions

#set page(
  margin: 30pt,
  numbering: "1",
  footer: align(
    center, 
    context(counter(page).display())
  )
)

#align(center)[
#text(size: 20pt)[= Optimizing DME]
#image("imgs/logo.svg", height: 4em)
  = HPC - Projet final

  Aubry Mangold and Samuel Roland
]

#pagebreak()

#outline(
 title: "Table of Contents",
)

== Introduction

#figure(
  image("imgs/code-snippet.png", width: 80%),
  caption: [Code snippet in C highlighted with Tree-Sitter and colored with Catpuccin Latte theme],
) <fig-code-snippet>

== Baseline

== Optimisation de la colorisation syntaxique

=== Comment ça fonctionne actuellement ?
TODO Aubry
=== imp 1
=== imp 2

== Optimisation de la recherche
TODO Sam

== Optimizing grammar installation

If we have time

== Conclusion

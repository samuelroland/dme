// Presentation about the paradigm choice and language

// Sous la forme d'une présentation de maximum 10 minutes, vous présentez à la classe votre choix :
//     Quel paradigme, ainsi que le langage associé ;
//     Pourquoi (particularité du paradigme et/ou du langage) ;
//     Comment prévoyez-vous de l'utiliser (courte description du projet qui mettra en avant les particularités du paradigme et/ou langage).

#import "@preview/typslides:1.2.3": * // https://github.com/manjavacas/typslides
#import "@preview/tablex:0.0.8": tablex

// Project configuration
#show: typslides.with(
  ratio: "16-9",
  theme: "yelly",
)

#text(font: "Cantarell", [

#blank-slide[
  #align(center, [
  #yelly[#text(weight: "bold", size: 2.2em, fill: gradient.linear(rgb("#fc1100"),rgb("#ffb000")))[Delightful Markdown Experience ?]]
  = That's possible !
  ])
]

#slide(title: "Meta experience")[
== Current experience
 - Preview not always pleasant
 - Code highlighting too much basic
 - PDF export hard and broken
 - Single file preview

== Dream experience
- Jumping easily through any Markdown file on disk
- Full text search on Markdown content
- Fast preview load and refresh, even for very big documents
]
    // TODO continue this

])

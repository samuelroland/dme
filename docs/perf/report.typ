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
#text(size: 20pt)[= Optimisation de DME]
#image("imgs/logo.svg", height: 4em)
  = HPC - Projet final
  2025-06-10

  Aubry Mangold and Samuel Roland
]

#pagebreak()

#outline(
 title: "Table of Contents",
)

== Introduction

DME (Delightful Markdown Preview) a pour but de faciliter l'expérience autour de l'édition du Markdown, notamment sur la colorisation des snippets de code et la recherche de fichiers Markdown sur le disque pour faciliter switcher de fichiers directement depuis l'interface. Le preview se fait en convertissant le Markdown en un output HTML + CSS (pour le thème).

DME se découpe en une librairie `dme-core` dans le sous dossier `app/core` qui stocke toute la logique coeur de l'application. Nous allons nous concentrer uniquement sur cette partie pour ce projet d'optimisation. Cette librairie est ensuite utilisée par une application de bureau (avec le framework Tauri) pour bénéficier des fonctionnalités depuis une interface graphique. Tauri nous permet de faire des applications web dans des applications desktop en mettant ensemble VueJS (framework frontend) et Rust en backend.

#figure(
  image("imgs/simple-preview.png", width: 60%),
  caption: [Aperçu de DME ouvert sur un document Markdown, l'aperçu étant simplement une page HTML],
) <fig-simple-preview>

#figure(
  image("imgs/code-snippet.png", height: 50%),
  caption: [Un snippet de code C colorisé par Tree-Sitter et le thème Catpuccin Latte],
) <fig-code-snippet>


Le projet DME utilise Tree-Sitter pour la colorisation syntaxique. Cette librairie qui est un générateur de parseur permet de créer un CST (Concrete Syntax Tree) qui décrit la structure d'un morceau de code sous forme d'un arbre de tokens avec un type associé. Ces tokens permettent ensuite notamment de faire de la colorisation syntaxique (syntax highlighting) dans les IDE (tel que Neovim, Zed, Helix et bientôt VSCode) basée sur des parseurs puissants plutôt que des systèmes de regexes. Cette colorisation avancée peut aussi être utilisée ailleurs, sur le web notamment mais demande l'intégration n'est pas triviale.

DME intègre un système de téléchargement des grammaires pour les languages à coloriser souhaités et charge ensuite dynamiquement une librairie partagée par grammaire depuis la librairie `tree-sitter` (crate Rust) embarquée.

Cette colorisation est généralement plus poussées et plaisante visuellement, c'est pour cette raison que cette complexité a été intégrée dans DME.

Un système de recherche permet ensuite de chercher un fichier Markdown sur son disque dur, en cherchant via son chemin ou n'importe quel titre d'un des documents. Le but est de pouvoir facilement switcher entre des fichiers Markdown sans devoir constamment ouvrir des dossiers à droite à gauche pour trouver le bon document. Cela ressemble au `Ctrl+P` dans VSCode qui donne un sélecteur fuzzy sur la liste des fichiers. Mais cela est étendu au contenu des titres et ne se restreint pas au dossier ouvert comme sur VSCode.

#figure(
  image("imgs/search-overview.png", width: 60%),
  caption: [Aperçu de l'interface de recherche, montrant des résultats de recherches avec des titres qui ont matchés.],
) <fig-search-overview>

// TODO systeme de priority ??

// TODO: description de la topo des systèmes de tests
// TODO: description de la stratégie de benchmarking (mentionner essais ratés)

== Tests mis en place

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

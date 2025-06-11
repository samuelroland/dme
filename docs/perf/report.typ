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

== Tests de stabilité
Pour éviter les regresssions et s'assurer de la stabilité une fois le code refactorisé, nous avons créé en plus des tests unitaires existants, une suite de tests d'intégration qui nous permettent de vérifier le comportement général du preview et de la recherche.

Nous avons développés ça dans `app/core/tests`, ils peuvent être lancés spécifiquement depuis ce dossier avec `cargo test`, une fois les tests unitaires passés ils se lancent. Voir `tests/large_preview.rs` et `tests/large_search.rs` si utile.

Voici un exemple de test qui fait TODO

```rust
#[test]
fn test_large_search_on_mdn_content_can_find_a_single_heading() {
    let repos = clone_mdn_content();
    let mut disk_search = DiskResearcher::new(repos.to_str().unwrap().to_string());
    disk_search.start();
    let search = "Array constructor with a single parameter";
    let results = disk_search.search(search, 20, None);
    assert_eq!(
        results.len(),
        1,
        "Results should have only one result, contains\n{results:?}"
    );
    assert_eq!(results[0].title, Some(search.to_string()));
    assert!(results[0].path.ends_with(
        "content/files/en-us/web/javascript/reference/global_objects/array/array/index.md"
    ));
}
```

== Système de benchmark

Comme à chaque laboratoire de HPC, nous avons commencé par 

Tout le code est toujours compilés en `--release` ce qui active les optimisations du compilateur Rust. On ne risque pas d'oublier de le faire car nous avons mis un check qui panic seulement en mode debug.

```rust
debug_assert!(false, "This benchmark system MUST be compiled and run in --release mode only to have best performances.");
```

Nous avons ensuite du configurer le profile de compilation pour `--release` pour qu'il garde les symboles de debug (équivalent de `-g`)

En rajoutant cette section au `Cargo.toml` de `bench`.
```toml
[profile.release]
debug = true
```

Pour lister nos benchmark il suffit de lancer le programme `bench` normalement.

// TODO: complete list of benchmark there with latest names and desc

```sh
cd app/core/bench
> cargo run --release

Listing available benchmarks
- preview_code : Different code snippets numbers in various languages
- preview_md : Large Markdown file without code snippets

To execute a benchmark run: cargo run --release -- bench <id>
```


Nous avons choisi de définir les paramètres de mon benchmark comme suit.
- Benchmark du preview via la fonction `markdown_to_highlighted_html(path: &str) -> Result<Html, String>`
  - `preview_code`: en utilisant une fonction utilitaire `generate_large_markdown_with_codes(30, 15);` nous pouvons définir un maximum de 30 code par language et un maximum de 15 languages. Ce ne sont que des limites maximum mais dans tous les cas le fichiers est bien grand. Le fichier généré dans `target/large-30.md` contient ainsi *117* morceaux de code dans 8 languages: `c go haskell java javascript lua rust scala`. Nous n'avons pas réussi à installer toutes les grammairs des languages proposés ou certaines n'ont pas de snippet disponible dans le repository utilisés.
  - `preview_md`: nous avons choisi un grand fichier `files/en-us/mdn/writing_guidelines/writing_style_guide/index.md` sans aucun morceau de code. Comme la génération du HTML par Comrak est très rapide, nous avons fait un dupliqué de 30 fois son contenu et mis son résultat dans `target/big_markdown.md` ce qui fait 1.8M.

== Baseline

== Optimisation de la colorisation syntaxique
=== Comment ça fonctionne actuellement ?

Tree-Sitter est une technologie de loin simple à appréhender, de nombreux d'éléments le composent. Si besoin d'avoir une vue générale du fonctionnement des étapes, voir le documents #link("https://github.com/samuelroland/dme/blob/main/app/core/docs.md#preview")[docs.md] section Preview sur le repository.

En très résumé, nous avons des grammaires qui définissent comment tokeniser un language. Ces grammaires sont publiés sur des repository Git tel que #link("https://github.com/tree-sitter/tree-sitter-css")[celui pour le css `tree-sitter-css`].

Elle contiennent un parseur généré en C (depuis la définition javascript ou JSON)
```
src> tree
├── grammar.json
├── node-types.json
├── parser.c
├── scanner.c
└── tree_sitter
    ├── alloc.h
    ├── array.h
    └── parser.h
```

On peut ensuite demander de le compiler en librairie partagée qui va être dynamiquement chargée au runtime au moment où on a besoin de coloriser un code d'un langage donné.

Voici les étapes par lesquels nous devons passer pour coloriser via la crate `tree-sitter` et `tree-sitter-highlight`, une fois une grammaire clonée et compilée.
- Créer un `Loader` qui définit le chemin final des librairies partagées
- TODO continue

=== Version 1, analyse du code de départ

Nous avons pu utiliser perf, en lancant notre fonction via notre système de benchmark, l'overhead pour arriver sur la fonction est très petit comme on y va directement le code de préparation est fait séparement.
```sh
cd app/core/bench
cargo build --release
sudo perf record --call-graph dwarf -e cpu-cycles target/release/bench fn preview_md target/large-15.md
hotspot perf.data
```

On voit clairement que tout le temps est passé dans l'initialisation de la `HighlightConfig`, cette partie qui va charger les fichiers de query (de requête sur l'arbre de tokens) qui permettent ensuite d'attribuer les noms de surlignage.
#figure(
  image("imgs/flamegraph-v1.png", width: 80%),
  caption: [Flameshot de la V1],
) <fig-flamegraph-v1>

On reconnait les noms de nos fonctions soulignés, le départ `markdown_to_highlighted_html` jusqu'à la dernière fonction visible dans notre code `HighlightConfig::new`, qui prend la grande majeure partie du temps.

// todo inclure exemple de query de ma docs un des readme du repos, chaque query c'est une ligne

Nous n'allons pas chercher à optimiser ce parsing de query étant dans une librairie séparée écrite en C. Mais nous pouvons essayer d'éviter un maximum de recréer ces `HighlightConfig` pour rien, c'est à dire d'éviter de créer des `TreeSitterHighlighter` qui en stocke.

Cette situation n'est pas surprenante comme dans notre usage du parseur Markdown (la librairie `comrak`), nous définissons un morceau de code qui sera lancé pour transformer chaque code snippet rencontré dans sa forme HTML à l'aise de `TreeSitterHighlighter`. A chaque snippet, on recrée cette configuration comme on le voit dans le snippet suivant.
```rust
impl SyntaxHighlighterAdapter for ComrakParser {
    fn write_highlighted(
        &self,
        output: &mut dyn Write,
        lang: Option<&str>,
        code: &str,
    ) -> io::Result<()> {
    // ...
      let mut loader =
          Loader::new().map_err(|e| std::io::Error::new(io::ErrorKind::Other, e))?;
      let highlighter =
          TreeSitterHighlighter::new(&mut loader, lang.unwrap_or_default(), &self.manager);
      // If lang might be supported or not
      output.write_all(highlighter.highlight(code).as_string().as_bytes())
    // ...
    }
```

Comme mentionné, notre `TreeSitterHighlighter` contient cette configuration `HighlightConfiguration` qu'il crée dans son constructeur `new()`.
```rust
pub struct TreeSitterHighlighter<'a> {
    lang: &'a str,
    highlight_config: &'a HighlightConfiguration,
}
```

// code de départ sur commit 92c94261 si besoin

Résultat du benchmark
- `preview_code`: 5.1575s
- `preview_md`: 0.0459s

// TODO: en faire un CSV avec les résutats si ca fait sens ? maybe pas en fait

=== V2, mise en cache des grammaires une fois chargée

Résultat du benchmark
- `preview_code`: 0.3093s
- `preview_md`:  0.0466s


TODO Aubry
=== imp 1
=== imp 2

== Optimisation de la recherche


TODO Sam

== Optimizing grammar installation

If we have time

== Conclusion

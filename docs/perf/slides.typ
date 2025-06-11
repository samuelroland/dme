#import "@preview/polylux:0.4.0": *

#set page(paper: "presentation-16-9")
#set text(lang: "fr", size: 25pt, discretionary-ligatures: true)

#slide[
  #set align(horizon)
  = HPC -- Projet final
  == Optimisation de DME (Rust)

  Aubry Mangold et Samuel Roland

  11 juin 2025
]

#slide[
  == Sommaire

  + Contexte et architecture
  + Rust et Tree-Sitter
  + Tests et infrastructure de benchmark
  + Problème initial de colorisation
  + Optimisation de la colorisation
  + Optimisation de l'installation des grammaires
  + Optimisation de la recherche
  + Conclusion et perspectives
]

#slide[
  == Contexte et architecture

  - DME : "Delightful Markdown Experience" (projet scolaire)
  - Conversion de Markdown vers HTML/CSS via Comrak
  - Recherche de fichiers Markdown dans le système
  - Architecture :
    - `dme-core` (Rust) + front VueJS/Tauri
]

#slide[
  == Rust et Tree-Sitter

  - Rust : modèle mémoire strict, ownership & lifetimes
  - Comrak pour parser Markdown
  - Tree-Sitter pour syntax highlighting
    - CST (Concrete Syntax Tree)
    - Queries & HighlightConfiguration
    - Difficultés avec le modèle mémoire
]

#slide[
  == Tests et infrastructure de benchmark

  - Tests unitaires et d'intégration
  - Benchmarks intégrés avec le binaire `bench`
  - Exécution systématique en `--release` (+ debug symbols pour Perf)
]

#slide[
  == Problème initial de colorisation

  - `HighlightConfig` recréé pour chaque snippet
  - 117 snippets mènent à 117 initialisations coûteuses
  - Perf montre un pic dans `HighlightConfig::new`

    Résultats: TODO graph v1
]

#slide[
  == Optimisation de la colorisation
  - Cache global `TSH_CACHE: Lazy<RwLock<HashMap>>`
  - Lecture rapide des highlighters existants
  - Écriture (creation) uniquement au premier usage par langue

    Résultats: TODO grpah v2
]

#slide[

  == Optimisation de l’installation de grammaires
  - Poids élevé dû à l’historique Git (.git ~49 M)
  - Passage à `git clone --depth 1 --single-branch`
    - Ajout de paramètres `only_latest_commits` et `single_branch`

  TODO graph v3
]

#slide[
  == Optimisation de la recherche

  // TODO: a reprendre SAM

  - Indexation rapide du dépôt MDN
  - Fuzzy matching sur titres et chemins
  - Benchmark général_keyword : 159 ms (index + recherche)
  - Streaming des résultats pour réactivité
  - Pas eu le temps d'optimiser
]

#slide[
  == Conclusion et perspectives

  - Transposition C → Rust présente des défis autour du modèle mémoire
  - Importance des tests et bench intégrés
  - Gains majeurs avec mise en cache et clone léger
  - Pistes futures :
    - Parallélisation du cache (lecture concurrente)
    - Optimisation de la recherche fuzzy
]

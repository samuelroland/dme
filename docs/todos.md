gestion état partagé et communication inter threads

todo analyse static, remove numerous checks at compile time, no type at runtime
todo expliquer où ça a été inventé et pourquoi

ajouter précision de ce que c'est vraiment uen lifteime comment ca finit

comment se passe une allocation dynamique, linteraction entre owner et lifetime. -> appel du drop
```sh
rustc -Z unpretty="mir" src/main.rs | bat -l rust
```


error du value doesnt live long enough

annoter pour préciser les lifetimes des variables

add deadlock mention in limit

passer tout à languagetool !

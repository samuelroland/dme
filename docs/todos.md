gestion état partagé et communication inter threads


todo analyse static, remove numerous checks at compile time, no type at runtime
todo expliquer où ça a été inventé et pourquoi

ajouter précision de ce que c'est vraiment uen lifteime comment ca finit

comment se passe une allocation dynamique, linteraction entre owner et lifetime. -> appel du drop
```sh
rustc -Z unpretty="mir" src/main.rs | bat -l rust
```

schéma du sapin

error du value doesnt live long enough

annoter pour préciser les lifetimes des variables

exemple de en quoi les lifetimes sont transcrites dans le système de type, exemple avec code c fonction on sait pas quand free

```c
// library.h
void save_file(float* buffer, char* filename);

// main.c
float* buffer = malloc(SIZE * sizeof(float));
char* filename = "test.txt";
save_file(buffer, filename);
free(buffer)
```

```rust
fn save_file(buffer: &[f32], filename: &str) {}

fn main() {
    let buffer = [10.2, 3.2, 5.2];
    let filename = "test.txt";
    save_file(&buffer, filename);
}
```


checklist de règles enforced du borrowchecker



add deadlock mention in limit

just a pointer

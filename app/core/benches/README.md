# Benchmark system based on Criterion.rs

start by reading parts of the docs

- https://bheisler.github.io/criterion.rs/book/user_guide/command_line_output.html
- https://bheisler.github.io/criterion.rs/book/user_guide/command_line_options.html
- https://bheisler.github.io/criterion.rs/book/user_guide/benchmarking_with_inputs.html
- https://bheisler.github.io/criterion.rs/book/user_guide/advanced_configuration.html

## Write benchmarks under benches/


To run it use --bench to specify the name of the file under `benches` without the extension.
```sh
cargo bench --quiet --bench preview
```

Tweak params to make sure it doesnt consider some improvement made by noise around.

## Start with a baseline
Go on start code `git checkout main`.

Save the baseline with `--save-baseline` under a name like `start`
```sh
cargo bench --quiet --bench preview -- --save-baseline start
```

## Do some change

## Benchmark again by comparing to named baseline

```sh
cargo bench --quiet --bench preview -- --baseline start
```

Note: these warnings are normal...
```
Warning: you should add a `highlights` entry pointing to the highlights path in the `tree-sitter` object in the grammar's tree-sitter.json file.
See more here: https://tree-sitter.github.io/tree-sitter/3-syntax-highlighting#query-paths
```

When there is no change.
```sh
preview_codes/preview {i}
    time:   [258.93 ms 260.50 ms 262.24 ms]
    change: [−3.6723% −2.6499% −1.6663%] (p = 0.00 < 0.05)
    Change within noise threshold.
```

## Profiling
see docs https://bheisler.github.io/criterion.rs/book/user_guide/profiling.html
to see how to reuse the benchmarks for profiling

## see cargo flamegraph


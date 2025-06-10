use criterion::{criterion_group, criterion_main, Criterion};
use dme_core::markdown_to_highlighted_html;
use dme_core::util::setup::{
    clone_mdn_content, generate_large_markdown_with_codes,
    install_all_grammars_in_local_target_folder,
};
use std::hint::black_box;
use std::time::Duration;

pub fn preview_nocode_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("preview_nocode");
    // TODO: tweak that if needed
    group.warm_up_time(Duration::from_secs(1));
    group.sample_size(10);
    let path = clone_mdn_content();
    // That's a file without any code snippet and of 59627 chars.
    let path = path.join("files/en-us/mdn/writing_guidelines/writing_style_guide/index.md");
    // TODO: duplicate this file a few times to make it slower to parse ?
    group.bench_function("preview basic", |b| {
        b.iter(|| markdown_to_highlighted_html(black_box(path.to_str().unwrap())))
    });
    group.finish();
}

pub fn preview_codes_benchmark(c: &mut Criterion) {
    install_all_grammars_in_local_target_folder(); // can be take 2-5 minutes the first time...

    let mut group = c.benchmark_group("preview_codes");
    // TODO: tweak these params to have a not too slow benchmark
    group.warm_up_time(Duration::from_millis(300));
    group.sample_size(10);
    group.noise_threshold(0.02);
    // for i in [1, 2, 5] {
    let i = 1;
    let path = generate_large_markdown_with_codes(i);
    group.bench_function("preview {i}", |b| {
        b.iter(|| markdown_to_highlighted_html(black_box(&path)))
    });
    // }
    group.finish();
}

// Ignore first bench for now
// criterion_group!(benches, preview_nocode_benchmark, preview_codes_benchmark);
criterion_group!(benches, preview_codes_benchmark);
criterion_main!(benches);

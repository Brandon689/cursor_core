use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput,
};

use cursor_core::Cursor;

fn make_repeated(mut pat: &[u8], size: usize) -> Vec<u8> {
    let mut buf = Vec::with_capacity(size);
    while buf.len() < size {
        let take = (size - buf.len()).min(pat.len());
        buf.extend_from_slice(&pat[..take]);
        if take < pat.len() {
            pat = &pat[take..];
        }
    }
    buf
}

fn bench_next_byte(c: &mut Criterion) {
    let mut group = c.benchmark_group("next_byte");
    for &size in &[1_024usize, 16_384, 262_144, 1_048_576] {
        let buf = make_repeated(b"abcdefghijklmnopqrstuvwxyz0123456789", size);
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format!("consume_all/{}B", size), |b| {
            b.iter_batched(
                || Cursor::new(black_box(&buf)),
                |mut cur| {
                    while cur.next_byte().is_some() {}
                    black_box(cur.pos())
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

fn bench_iterator_next(c: &mut Criterion) {
    let mut group = c.benchmark_group("iterator_next");
    for &size in &[1_024usize, 16_384, 262_144, 1_048_576] {
        let buf = make_repeated(b"The quick brown fox jumps over the lazy dog.", size);
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format!("for_next/{}B", size), |b| {
            b.iter_batched(
                || Cursor::new(black_box(&buf)),
                |mut cur| {
                    let mut sum = 0u64;
                    while let Some(b) = cur.next() {
                        sum = sum.wrapping_add(b as u64);
                    }
                    black_box(sum)
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

fn bench_skip_until(c: &mut Criterion) {
    let mut group = c.benchmark_group("skip_until");
    for &size in &[4_096usize, 32_768, 262_144, 1_048_576] {
        // Insert a newline ~every 64 bytes.
        let mut chunk = vec![b'a'; 63];
        chunk.push(b'\n');
        let buf = make_repeated(&chunk, size);

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format!("newline/{}B", size), |b| {
            b.iter_batched(
                || Cursor::new(black_box(&buf)),
                |mut cur| {
                    let mut lines = 0usize;
                    loop {
                        let _ = cur.skip_until(b'\n');
                        if cur.peek() == Some(b'\n') {
                            let _ = cur.next_byte();
                            lines += 1;
                        }
                        if cur.eof() {
                            break;
                        }
                    }
                    black_box(lines)
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

fn bench_match_bytes(c: &mut Criterion) {
    let mut group = c.benchmark_group("match_bytes");
    let pat = b"hello";
    for &size in &[4_096usize, 32_768, 262_144, 1_048_576] {
        // Ensure frequent matches, e.g., "helloXhelloY..."
        let buf = make_repeated(b"helloX", size);

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format!("pat_hello/{}B", size), |b| {
            b.iter_batched(
                || Cursor::new(black_box(&buf)),
                |mut cur| {
                    let mut hits = 0usize;
                    while !cur.eof() {
                        if cur.match_bytes(pat).is_some() {
                            hits += 1;
                        } else {
                            let _ = cur.next_byte();
                        }
                    }
                    black_box(hits)
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

fn bench_take_space_and_idents(c: &mut Criterion) {
    let mut group = c.benchmark_group("take_space_ident");
    for &size in &[4_096usize, 32_768, 262_144, 1_048_576] {
        // Pattern: whitespace + identifier tokens separated by spaces.
        let buf = make_repeated(b"   \t\n\rident_123 other_ident XYZ_9  ", size);

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format!("scan_idents/{}B", size), |b| {
            b.iter_batched(
                || Cursor::new(black_box(&buf)),
                |mut cur| {
                    let mut idents = 0usize;
                    while !cur.eof() {
                        let _ = cur.take_space();
                        if cur.take_ident_ascii().is_some() {
                            idents += 1;
                        } else {
                            // Progress by one byte if no token matched.
                            let _ = cur.next_byte();
                        }
                    }
                    black_box(idents)
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

fn bench_take_int_ascii(c: &mut Criterion) {
    let mut group = c.benchmark_group("take_int_ascii");
    for &size in &[4_096usize, 32_768, 262_144, 1_048_576] {
        // Numbers separated by mixed whitespace and punctuation.
        let buf = make_repeated(b"12345 6789\t42,\n1001; 0 99999 xyz ", size);

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format!("scan_ints/{}B", size), |b| {
            b.iter_batched(
                || Cursor::new(black_box(&buf)),
                |mut cur| {
                    let mut ints = 0usize;
                    while !cur.eof() {
                        if cur.take_int_ascii().is_some() {
                            ints += 1;
                        } else {
                            let _ = cur.next_byte();
                        }
                    }
                    black_box(ints)
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

fn bench_peek_and_peek_slice(c: &mut Criterion) {
    let mut group = c.benchmark_group("peek_peek_slice");
    for &size in &[4_096usize, 32_768, 262_144, 1_048_576] {
        let buf = make_repeated(b"abcdefg12345_", size);

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format!("peek/{}B", size), |b| {
            b.iter_batched(
                || Cursor::new(black_box(&buf)),
                |mut cur| {
                    let mut sum = 0usize;
                    while !cur.eof() {
                        if let Some(b) = cur.peek() {
                            sum = sum.wrapping_add(b as usize);
                        }
                        let _ = cur.next_byte();
                    }
                    black_box(sum)
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_function(format!("peek_slice_8/{}B", size), |b| {
            b.iter_batched(
                || Cursor::new(black_box(&buf)),
                |mut cur| {
                    let mut sum = 0u64;
                    while !cur.eof() {
                        if let Some(s) = cur.peek_slice(8) {
                            sum = sum.wrapping_add(s.len() as u64);
                        }
                        let _ = cur.next_byte();
                    }
                    black_box(sum)
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

fn bench_take_ident_starting_alpha(c: &mut Criterion) {
    let mut group = c.benchmark_group("take_ident_starting_alpha");
    for &size in &[4_096usize, 32_768, 262_144, 1_048_576] {
        let buf = make_repeated(b"_not alpha_start valid idA_1 next _skip A1_ok ", size);

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format!("scan_alpha_idents/{}B", size), |b| {
            b.iter_batched(
                || Cursor::new(black_box(&buf)),
                |mut cur| {
                    let mut count = 0usize;
                    while !cur.eof() {
                        if cur.take_ident_starting_alpha().is_some() {
                            count += 1;
                        } else {
                            let _ = cur.next_byte();
                        }
                    }
                    black_box(count)
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

fn bench_expectations(c: &mut Criterion) {
    let mut group = c.benchmark_group("expectations");
    for &size in &[4_096usize, 32_768, 262_144, 1_048_576] {
        let buf = make_repeated(b"XabcYabcZ", size);

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format!("expect_byte/{}B", size), |b| {
            b.iter_batched(
                || Cursor::new(black_box(&buf)),
                |mut cur| {
                    let mut hits = 0usize;
                    while !cur.eof() {
                        if cur.expect_byte(b'X').is_some() {
                            hits += 1;
                        } else {
                            let _ = cur.next_byte();
                        }
                    }
                    black_box(hits)
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_function(format!("expect_bytes_abc/{}B", size), |b| {
            b.iter_batched(
                || Cursor::new(black_box(&buf)),
                |mut cur| {
                    let mut hits = 0usize;
                    while !cur.eof() {
                        if cur.expect_bytes(b"abc").is_some() {
                            hits += 1;
                        } else {
                            let _ = cur.next_byte();
                        }
                    }
                    black_box(hits)
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

fn bench_starts_with(c: &mut Criterion) {
    let mut group = c.benchmark_group("starts_with");
    for &size in &[4_096usize, 32_768, 262_144, 1_048_576] {
        let buf = make_repeated(b"foobar foobaz fooqux barfoo ", size);
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format!("starts_with_foo/{}B", size), |b| {
            b.iter_batched(
                || Cursor::new(black_box(&buf)),
                |mut cur| {
                    let mut hits = 0usize;
                    while !cur.eof() {
                        if cur.starts_with(b"foo") {
                            hits += 1;
                        }
                        let _ = cur.next_byte();
                    }
                    black_box(hits)
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

fn bench_take_space(c: &mut Criterion) {
    let mut group = c.benchmark_group("take_space");
    for &size in &[4_096usize, 32_768, 262_144, 1_048_576] {
        let buf = make_repeated(b"   \t\t\n\r\x0C\x0Bword", size);
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format!("consume_spaces/{}B", size), |b| {
            b.iter_batched(
                || Cursor::new(black_box(&buf)),
                |mut cur| {
                    let mut count = 0usize;
                    while !cur.eof() {
                        if cur.take_space().is_some() {
                            count += 1;
                        } else {
                            let _ = cur.next_byte();
                        }
                    }
                    black_box(count)
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

fn bench_skip_while(c: &mut Criterion) {
    let mut group = c.benchmark_group("skip_while");
    for &size in &[4_096usize, 32_768, 262_144, 1_048_576] {
        let buf = make_repeated(b"aaaaaBBBBBcccccDDDDD", size);
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format!("lower_then_upper/{}B", size), |b| {
            b.iter_batched(
                || Cursor::new(black_box(&buf)),
                |mut cur| {
                    let mut segments = 0usize;
                    while !cur.eof() {
                        let _ = Cursor::skip_while(&mut cur, |b| b.is_ascii_lowercase());
                        let _ = Cursor::skip_while(&mut cur, |b| b.is_ascii_uppercase());
                        segments += 1;
                    }
                    black_box(segments)
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

pub fn criterion_benchmark(c: &mut Criterion) {
    bench_next_byte(c);
    bench_iterator_next(c);
    bench_skip_until(c);
    bench_match_bytes(c);
    bench_take_space_and_idents(c);
    bench_take_int_ascii(c);
    bench_peek_and_peek_slice(c);
    bench_take_ident_starting_alpha(c);
    bench_expectations(c);
    bench_starts_with(c);
    bench_take_space(c);
    bench_skip_while(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

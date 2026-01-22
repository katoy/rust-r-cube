use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rubiks_cube_3x3::cube::{Cube, Move};
use rubiks_cube_3x3::solver;

/// ソルバーの基本性能ベンチマーク
fn benchmark_solver_easy(c: &mut Criterion) {
    c.bench_function("solver_scramble_3", |b| {
        b.iter(|| {
            let mut cube = Cube::new();
            cube.scramble(3);
            let result = solver::solve(black_box(&cube), 11, false);
            assert!(result.found);
        });
    });
}

fn benchmark_solver_medium(c: &mut Criterion) {
    c.bench_function("solver_scramble_5", |b| {
        b.iter(|| {
            let mut cube = Cube::new();
            cube.scramble(5);
            let result = solver::solve(black_box(&cube), 11, false);
            assert!(result.found);
        });
    });
}

fn benchmark_solver_hard(c: &mut Criterion) {
    c.bench_function("solver_scramble_8", |b| {
        b.iter(|| {
            let mut cube = Cube::new();
            cube.scramble(8);
            let result = solver::solve(black_box(&cube), 11, false);
            assert!(result.found);
        });
    });
}

/// 深いスクランブルのベンチマーク（10手）
fn benchmark_solver_god_number(c: &mut Criterion) {
    c.bench_function("solver_scramble_10", |b| {
        b.iter(|| {
            // 10手のスクランブル（god numberに近い）
            let mut cube = Cube::new();
            cube.scramble(10);
            let result = solver::solve(black_box(&cube), 11, false);
            assert!(result.found);
        });
    });
}

/// 向きを考慮したソルバーのベンチマーク
fn benchmark_solver_with_orientation(c: &mut Criterion) {
    c.bench_function("solver_with_orientation", |b| {
        b.iter(|| {
            let mut cube = Cube::new();
            cube.scramble(5);
            let result = solver::solve(black_box(&cube), 11, false);
            assert!(result.found);
        });
    });
}

/// 向きを無視したソルバーのベンチマーク
fn benchmark_solver_ignore_orientation(c: &mut Criterion) {
    c.bench_function("solver_ignore_orientation", |b| {
        b.iter(|| {
            let mut cube = Cube::new();
            cube.scramble(5);
            let result = solver::solve(black_box(&cube), 11, true);
            assert!(result.found);
        });
    });
}

/// 基本的なキューブ操作のベンチマーク
fn benchmark_cube_operations(c: &mut Criterion) {
    c.bench_function("cube_apply_move", |b| {
        let mut cube = Cube::new();
        b.iter(|| {
            cube.apply_move(black_box(Move::R));
        });
    });

    c.bench_function("cube_scramble_100", |b| {
        b.iter(|| {
            let mut cube = Cube::new();
            cube.scramble(black_box(100));
        });
    });

    c.bench_function("cube_clone", |b| {
        let cube = Cube::new();
        b.iter(|| {
            black_box(cube.clone());
        });
    });
}

/// ハッシュ計算のベンチマーク
fn benchmark_cube_hash(c: &mut Criterion) {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    c.bench_function("cube_hash", |b| {
        let cube = Cube::new();
        b.iter(|| {
            let mut hasher = DefaultHasher::new();
            black_box(&cube).hash(&mut hasher);
            hasher.finish()
        });
    });
}

/// 正規化処理のベンチマーク
fn benchmark_cube_normalization(c: &mut Criterion) {
    c.bench_function("cube_normalized", |b| {
        let cube = Cube::new();
        b.iter(|| {
            let _ = black_box(&cube).normalized();
        });
    });

    c.bench_function("cube_with_clockwise_orientations", |b| {
        let cube = Cube::new();
        b.iter(|| {
            let _ = black_box(&cube).with_clockwise_orientations();
        });
    });
}

/// ファイルI/Oのベンチマーク
fn benchmark_file_io(c: &mut Criterion) {
    c.bench_function("cube_to_file_format", |b| {
        let mut cube = Cube::new();
        cube.scramble(10);
        b.iter(|| {
            black_box(&cube).to_file_format();
        });
    });
}

criterion_group!(
    benches,
    benchmark_solver_easy,
    benchmark_solver_medium,
    benchmark_solver_hard,
    benchmark_solver_god_number,
    benchmark_solver_with_orientation,
    benchmark_solver_ignore_orientation,
    benchmark_cube_operations,
    benchmark_cube_hash,
    benchmark_cube_normalization,
    benchmark_file_io,
);
criterion_main!(benches);

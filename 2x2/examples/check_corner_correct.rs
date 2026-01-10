use rubiks_cube_2x2::cube::{Cube, Move};
use std::collections::HashSet;

fn main() {
    let cube = Cube::new();

    println!("=== 初期状態 ===");
    check_corners(&cube);

    // B操作
    let mut cube_b = cube.clone();
    cube_b.apply_move(Move::B);
    println!("\n=== B操作後 ===");
    check_corners(&cube_b);

    // 全操作チェック
    println!("\n=== 全操作耐久テスト ===");
    let moves = vec![
        Move::U,
        Move::D,
        Move::R,
        Move::L,
        Move::F,
        Move::B,
        Move::Up,
        Move::Dp,
        Move::Rp,
        Move::Lp,
        Move::Fp,
        Move::Bp,
    ];

    for &mv in &moves {
        let mut c = Cube::new();
        c.apply_move(mv);
        let valid = check_corners_silent(&c);
        if !valid {
            println!("❌ {:?} 操作でコーナー整合性が壊れました！", mv);
        } else {
            // println!("✅ {:?} OK", mv);
        }
    }

    // ユーザー報告の現象
    println!("\n=== ユーザー報告の手順 ===");
    let sequence = vec![
        Move::Bp,
        Move::Lp,
        Move::Bp,
        Move::Lp,
        Move::Fp,
        Move::D,
        Move::F,
        Move::U,
        Move::F,
        Move::R,
        Move::B,
        Move::Up,
    ];
    let mut c_seq = Cube::new();
    for (i, &mv) in sequence.iter().enumerate() {
        c_seq.apply_move(mv);
        if !check_corners_silent(&c_seq) {
            println!("ステップ {} ({:?}) で整合性が壊れました", i + 1, mv);
            check_corners(&c_seq); // 詳細表示
            break;
        }
    }
    if check_corners_silent(&c_seq) {
        println!("✅ 手順完了後も整合性は保たれています");
    }

    // 色の分布チェック
    check_color_distribution(&c_seq);

    // ランダムスクランブルテスト
    println!("\n=== ランダムスクランブルテスト (100回試行) ===");
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let all_moves = [
        Move::U,
        Move::D,
        Move::R,
        Move::L,
        Move::F,
        Move::B,
        Move::Up,
        Move::Dp,
        Move::Rp,
        Move::Lp,
        Move::Fp,
        Move::Bp,
    ];

    for i in 0..100 {
        let mut c_rand = Cube::new();
        let num_moves = rng.gen_range(10..30);
        let mut history = Vec::new();

        for _ in 0..num_moves {
            let mv = all_moves[rng.gen_range(0..all_moves.len())];
            c_rand.apply_move(mv);
            history.push(mv);

            if !check_corners_silent(&c_rand) {
                println!("❌ ランダムテスト失敗 (試行 {}):", i);
                println!("手順: {:?}", history);
                check_corners(&c_rand);
                return;
            }
        }
    }
    println!("✅ ランダムテスト 100回 クリア");
}

fn get_corners() -> Vec<(&'static str, Vec<usize>)> {
    vec![
        ("ULF", vec![2, 9, 16]),  // Up-Left-Front
        ("URF", vec![3, 12, 17]), // Up-Right-Front
        ("ULB", vec![0, 8, 21]),  // Up-Left-Back
        ("URB", vec![1, 13, 20]), // Up-Right-Back
        ("DLF", vec![4, 11, 18]), // Down-Left-Front
        ("DRF", vec![5, 14, 19]), // Down-Right-Front
        ("DLB", vec![6, 10, 23]), // Down-Left-Back
        ("DRB", vec![7, 15, 22]), // Down-Right-Back
    ]
}

fn check_corners(cube: &Cube) {
    let corners = get_corners();
    for (name, indices) in corners {
        let colors: Vec<String> = indices
            .iter()
            .map(|&i| format!("{:?}", cube.get_sticker(i).color))
            .collect();
        let unique: HashSet<&String> = colors.iter().collect();

        if unique.len() != 3 {
            println!(
                "❌ {}: 異なる色が{}個しかありません {:?} (indices: {:?})",
                name,
                unique.len(),
                colors,
                indices
            );
        } else {
            // println!("✅ {}: OK {:?}", name, colors);
        }
    }
}

fn check_corners_silent(cube: &Cube) -> bool {
    let corners = get_corners();
    for (_, indices) in corners {
        let colors: HashSet<String> = indices
            .iter()
            .map(|&i| format!("{:?}", cube.get_sticker(i).color))
            .collect();
        if colors.len() != 3 {
            return false;
        }
    }
    true
}

fn check_color_distribution(cube: &Cube) {
    use std::collections::HashMap;
    let mut count = HashMap::new();
    for i in 0..24 {
        let c = format!("{:?}", cube.get_sticker(i).color);
        *count.entry(c).or_insert(0) += 1;
    }
    println!("色の分布: {:?}", count);
}

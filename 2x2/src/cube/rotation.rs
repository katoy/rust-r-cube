use super::{Cube, Move};

/// 回転操作を実行
pub fn apply_move(cube: &mut Cube, mv: Move) {
    let (face, od, cycle, rot) = match mv {
        Move::U | Move::Up | Move::U2 => (
            0,
            1,
            [16, 17, 12, 13, 20, 21, 8, 9],
            [0, 0, 0, 0, 0, 0, 0, 0],
        ),
        Move::D | Move::Dp | Move::D2 => (
            4,
            1,
            [18, 19, 10, 11, 22, 23, 14, 15],
            [0, 0, 0, 0, 0, 0, 0, 0],
        ),
        Move::L | Move::Lp | Move::L2 => {
            (8, 3, [0, 2, 23, 21, 4, 6, 16, 18], [2, 2, 2, 2, 0, 0, 0, 0])
        }
        Move::R | Move::Rp | Move::R2 => (
            12,
            3,
            [1, 3, 17, 19, 5, 7, 22, 20],
            [0, 0, 0, 0, 2, 2, 2, 2],
        ),
        Move::F | Move::Fp | Move::F2 => {
            (16, 1, [2, 3, 11, 9, 5, 4, 12, 14], [1, 1, 3, 3, 1, 1, 3, 3])
        }
        Move::B | Move::Bp | Move::B2 => {
            (20, 1, [0, 1, 13, 15, 7, 6, 10, 8], [3, 3, 1, 1, 3, 3, 1, 1])
        }
    };

    let repeat = match mv {
        Move::U | Move::D | Move::L | Move::R | Move::F | Move::B => 1,
        Move::U2 | Move::D2 | Move::L2 | Move::R2 | Move::F2 | Move::B2 => 2,
        Move::Up | Move::Dp | Move::Lp | Move::Rp | Move::Fp | Move::Bp => 3,
    };

    for _ in 0..repeat {
        rotate_internal(cube, face, od, &cycle, &rot);
    }
}

fn rotate_internal(cube: &mut Cube, face: usize, od: u8, cycle: &[usize; 8], rot: &[u8; 8]) {
    // 1. 面自体のステッカーを CW 回転
    let temp = cube.stickers[face];
    cube.stickers[face] = cube.stickers[face + 2];
    cube.stickers[face + 2] = cube.stickers[face + 3];
    cube.stickers[face + 3] = cube.stickers[face + 1];
    cube.stickers[face + 1] = temp;
    for i in 0..4 {
        for _ in 0..od {
            cube.stickers[face + i].rotate_cw();
        }
    }

    // 2. 隣接する面のステッカーを循環移動 (2枚ずつのペアで CW 方向)
    let t0 = cube.stickers[cycle[0]];
    let t1 = cube.stickers[cycle[1]];
    for i in 0..3 {
        cube.stickers[cycle[i * 2]] = cube.stickers[cycle[(i + 1) * 2]];
        cube.stickers[cycle[i * 2 + 1]] = cube.stickers[cycle[(i + 1) * 2 + 1]];
    }
    cube.stickers[cycle[6]] = t0;
    cube.stickers[cycle[7]] = t1;

    // 3. 移動後の向きの調整
    for i in 0..8 {
        for _ in 0..rot[i] {
            cube.stickers[cycle[i]].rotate_cw();
        }
    }
}

/// ランダムなスクランブルを生成します。
pub fn scramble(cube: &mut Cube, moves: usize) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let all_moves = Move::all_moves();

    for _ in 0..moves {
        let mv = all_moves[rng.gen_range(0..all_moves.len())];
        apply_move(cube, mv);
    }
}

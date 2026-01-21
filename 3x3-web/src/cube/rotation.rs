use super::{Cube, Move};

/// 指定された回転操作をキューブに適用します。
pub fn apply_move(cube: &mut Cube, mv: Move) {
    let (base, repeat) = match mv {
        Move::U => (Move::U, 1),
        Move::Up => (Move::U, 3),
        Move::U2 => (Move::U, 2),
        Move::D => (Move::D, 1),
        Move::Dp => (Move::D, 3),
        Move::D2 => (Move::D, 2),
        Move::L => (Move::L, 1),
        Move::Lp => (Move::L, 3),
        Move::L2 => (Move::L, 2),
        Move::R => (Move::R, 1),
        Move::Rp => (Move::R, 3),
        Move::R2 => (Move::R, 2),
        Move::F => (Move::F, 1),
        Move::Fp => (Move::F, 3),
        Move::F2 => (Move::F, 2),
        Move::B => (Move::B, 1),
        Move::Bp => (Move::B, 3),
        Move::B2 => (Move::B, 2),
        Move::M => (Move::M, 1),
        Move::Mp => (Move::M, 3),
        Move::M2 => (Move::M, 2),
        Move::E => (Move::E, 1),
        Move::Ep => (Move::E, 3),
        Move::E2 => (Move::E, 2),
        Move::S => (Move::S, 1),
        Move::Sp => (Move::S, 3),
        Move::S2 => (Move::S, 2),
        Move::X => (Move::X, 1),
        Move::Xp => (Move::X, 3),
        Move::X2 => (Move::X, 2),
        Move::Y => (Move::Y, 1),
        Move::Yp => (Move::Y, 3),
        Move::Y2 => (Move::Y, 2),
        Move::Z => (Move::Z, 1),
        Move::Zp => (Move::Z, 3),
        Move::Z2 => (Move::Z, 2),
    };

    for _ in 0..repeat {
        match base {
            Move::U => rotate_face(
                cube,
                0,
                &[36, 37, 38, 27, 28, 29, 45, 46, 47, 18, 19, 20],
                &[0; 12],
            ),
            Move::D => rotate_face(
                cube,
                9,
                &[42, 43, 44, 33, 34, 35, 51, 52, 53, 24, 25, 26],
                &[0; 12],
            ),
            Move::L => rotate_face(
                cube,
                18,
                &[0, 3, 6, 53, 50, 47, 9, 12, 15, 36, 39, 42],
                &[2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0],
            ),
            Move::R => rotate_face(
                cube,
                27,
                &[11, 14, 17, 45, 48, 51, 2, 5, 8, 38, 41, 44],
                &[0, 0, 0, 2, 2, 2, 0, 0, 0, 2, 2, 2],
            ),
            Move::F => rotate_face(
                cube,
                36,
                &[26, 23, 20, 11, 10, 9, 27, 30, 33, 6, 7, 8],
                &[3, 3, 3, 3, 3, 3, 1, 1, 1, 1, 1, 1],
            ),
            Move::B => rotate_face(
                cube,
                45,
                &[35, 32, 29, 17, 16, 15, 18, 21, 24, 0, 1, 2],
                &[1, 1, 1, 1, 1, 1, 3, 3, 3, 3, 3, 3],
            ),
            Move::M => rotate_slice_internal(
                cube,
                &[37, 40, 43, 10, 13, 16, 52, 49, 46, 1, 4, 7],
                &[0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2],
            ),
            Move::E => rotate_slice_internal(
                cube,
                &[39, 40, 41, 30, 31, 32, 48, 49, 50, 21, 22, 23],
                &[0; 12],
            ),
            Move::S => rotate_slice_internal(
                cube,
                &[3, 4, 5, 28, 31, 34, 14, 13, 12, 25, 22, 19],
                &[1, 1, 1, 1, 1, 1, 3, 3, 3, 3, 3, 3],
            ),
            Move::X => {
                apply_move(cube, Move::R);
                apply_move(cube, Move::Lp);
                apply_move(cube, Move::Mp);
            }
            Move::Y => {
                apply_move(cube, Move::U);
                apply_move(cube, Move::Dp);
                apply_move(cube, Move::Ep);
            }
            Move::Z => {
                apply_move(cube, Move::F);
                apply_move(cube, Move::Bp);
                apply_move(cube, Move::S);
            }
            _ => unreachable!(),
        }
    }
}

fn rotate_face(cube: &mut Cube, face_start: usize, cycle: &[usize; 12], oris: &[u8; 12]) {
    // 1. 面自体の回転
    let s = &mut cube.stickers;
    // 角
    let tmp = s[face_start + 0];
    s[face_start + 0] = s[face_start + 6];
    s[face_start + 6] = s[face_start + 8];
    s[face_start + 8] = s[face_start + 2];
    s[face_start + 2] = tmp;
    // 辺
    let tmp = s[face_start + 1];
    s[face_start + 1] = s[face_start + 3];
    s[face_start + 3] = s[face_start + 7];
    s[face_start + 7] = s[face_start + 5];
    s[face_start + 5] = tmp;
    // 向きの更新
    for i in 0..9 {
        s[face_start + i].rotate_cw();
    }

    // 2. 隣接ステッカーの巡回
    rotate_slice_internal(cube, cycle, oris);
}

fn rotate_slice(cube: &mut Cube, cycle: &[usize; 12], oris: &[u8; 12]) {
    rotate_slice_internal(cube, cycle, oris);
}

fn rotate_centers_internal(cube: &mut Cube, cycle: &[usize; 4], cw: bool) {
    let s = &mut cube.stickers;
    if cw {
        let tmp = s[cycle[0]];
        s[cycle[0]] = s[cycle[3]];
        s[cycle[3]] = s[cycle[2]];
        s[cycle[2]] = s[cycle[1]];
        s[cycle[1]] = tmp;
    } else {
        let tmp = s[cycle[0]];
        s[cycle[0]] = s[cycle[1]];
        s[cycle[1]] = s[cycle[2]];
        s[cycle[2]] = s[cycle[3]];
        s[cycle[3]] = tmp;
    }
    for &idx in cycle {
        s[idx].rotate_cw();
    }
}

fn rotate_slice_internal(cube: &mut Cube, cycle: &[usize; 12], oris: &[u8; 12]) {
    let t0 = cube.stickers[cycle[0]];
    let t1 = cube.stickers[cycle[1]];
    let t2 = cube.stickers[cycle[2]];

    // (A <- B <- C <- D <- A)
    cube.stickers[cycle[0]] = cube.stickers[cycle[3]];
    cube.stickers[cycle[1]] = cube.stickers[cycle[4]];
    cube.stickers[cycle[2]] = cube.stickers[cycle[5]];

    cube.stickers[cycle[3]] = cube.stickers[cycle[6]];
    cube.stickers[cycle[4]] = cube.stickers[cycle[7]];
    cube.stickers[cycle[5]] = cube.stickers[cycle[8]];

    cube.stickers[cycle[6]] = cube.stickers[cycle[9]];
    cube.stickers[cycle[7]] = cube.stickers[cycle[10]];
    cube.stickers[cycle[8]] = cube.stickers[cycle[11]];

    cube.stickers[cycle[9]] = t0;
    cube.stickers[cycle[10]] = t1;
    cube.stickers[cycle[11]] = t2;

    // 3. 向きの調整
    for i in 0..12 {
        for _ in 0..oris[i] {
            cube.stickers[cycle[i]].rotate_cw();
        }
    }
}

pub fn scramble(cube: &mut Cube, moves: usize) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let all_moves = Move::all_moves();

    for _ in 0..moves {
        let mv = all_moves[rng.gen_range(0..all_moves.len())];
        apply_move(cube, mv);
    }
}

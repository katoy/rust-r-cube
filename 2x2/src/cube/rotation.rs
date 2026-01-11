use super::{Cube, Move};

/// 回転操作を実行
pub fn apply_move(cube: &mut Cube, mv: Move) {
    match mv {
        Move::R => rotate_r(cube),
        Move::Rp => rotate_rp(cube),
        Move::R2 => {
            rotate_r(cube);
            rotate_r(cube);
        }
        Move::L => rotate_l(cube),
        Move::Lp => rotate_lp(cube),
        Move::L2 => {
            rotate_l(cube);
            rotate_l(cube);
        }
        Move::U => rotate_u(cube),
        Move::Up => rotate_up(cube),
        Move::U2 => {
            rotate_u(cube);
            rotate_u(cube);
        }
        Move::D => rotate_d(cube),
        Move::Dp => rotate_dp(cube),
        Move::D2 => {
            rotate_d(cube);
            rotate_d(cube);
        }
        Move::F => rotate_f(cube),
        Move::Fp => rotate_fp(cube),
        Move::F2 => {
            rotate_f(cube);
            rotate_f(cube);
        }
        Move::B => rotate_b(cube),
        Move::Bp => rotate_bp(cube),
        Move::B2 => {
            rotate_b(cube);
            rotate_b(cube);
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

/// 面自体を時計回りに回転
fn rotate_face_cw(cube: &mut Cube, start_idx: usize, orient_delta: u8) {
    let temp = cube.stickers[start_idx];
    cube.stickers[start_idx] = cube.stickers[start_idx + 2];
    cube.stickers[start_idx + 2] = cube.stickers[start_idx + 3];
    cube.stickers[start_idx + 3] = cube.stickers[start_idx + 1];
    cube.stickers[start_idx + 1] = temp;

    for i in 0..4 {
        for _ in 0..orient_delta {
            cube.stickers[start_idx + i].rotate_cw();
        }
    }
}

/// 面自体を反時計回りに回転
fn rotate_face_ccw(cube: &mut Cube, start_idx: usize, orient_delta: u8) {
    let temp = cube.stickers[start_idx];
    cube.stickers[start_idx] = cube.stickers[start_idx + 1];
    cube.stickers[start_idx + 1] = cube.stickers[start_idx + 3];
    cube.stickers[start_idx + 3] = cube.stickers[start_idx + 2];
    cube.stickers[start_idx + 2] = temp;

    for i in 0..4 {
        for _ in 0..orient_delta {
            cube.stickers[start_idx + i].rotate_ccw();
        }
    }
}

/// R面を時計回りに回転
fn rotate_r(cube: &mut Cube) {
    rotate_face_cw(cube, 12, 3); // Right face (orient +3)

    let temp0 = cube.stickers[1];
    let temp1 = cube.stickers[3];

    // U <- F (R-slice) [0]
    cube.stickers[1] = cube.stickers[17];
    cube.stickers[3] = cube.stickers[19];

    // F <- D (R-slice) [0]
    cube.stickers[17] = cube.stickers[5];
    cube.stickers[19] = cube.stickers[7];

    // D <- B [+2]
    cube.stickers[5] = cube.stickers[22];
    cube.stickers[5].rotate_cw();
    cube.stickers[5].rotate_cw();
    cube.stickers[7] = cube.stickers[20];
    cube.stickers[7].rotate_cw();
    cube.stickers[7].rotate_cw();

    // B <- U [+2]
    cube.stickers[22] = temp0;
    cube.stickers[22].rotate_cw();
    cube.stickers[22].rotate_cw();
    cube.stickers[20] = temp1;
    cube.stickers[20].rotate_cw();
    cube.stickers[20].rotate_cw();
}

/// R面を反時計回りに回転
fn rotate_rp(cube: &mut Cube) {
    rotate_face_ccw(cube, 12, 3); // Right face (orient +3)

    let temp0 = cube.stickers[1];
    let temp1 = cube.stickers[3];

    // U <- B (+2)
    cube.stickers[1] = cube.stickers[22];
    cube.stickers[1].rotate_cw();
    cube.stickers[1].rotate_cw();
    cube.stickers[3] = cube.stickers[20];
    cube.stickers[3].rotate_cw();
    cube.stickers[3].rotate_cw();

    // B <- D (+2)
    cube.stickers[22] = cube.stickers[5];
    cube.stickers[22].rotate_cw();
    cube.stickers[22].rotate_cw();
    cube.stickers[20] = cube.stickers[7];
    cube.stickers[20].rotate_cw();
    cube.stickers[20].rotate_cw();

    // D <- F (+0)
    cube.stickers[5] = cube.stickers[17];
    cube.stickers[7] = cube.stickers[19];

    // F <- U (+0)
    cube.stickers[17] = temp0;
    cube.stickers[19] = temp1;
}

/// L面を時計回りに回転
fn rotate_l(cube: &mut Cube) {
    rotate_face_cw(cube, 8, 3); // Left face (orient +3)

    let temp0 = cube.stickers[0];
    let temp1 = cube.stickers[2];

    // U <- B [+2]
    cube.stickers[0] = cube.stickers[23];
    cube.stickers[0].rotate_cw();
    cube.stickers[0].rotate_cw();
    cube.stickers[2] = cube.stickers[21];
    cube.stickers[2].rotate_cw();
    cube.stickers[2].rotate_cw();

    // B <- D [+2]
    cube.stickers[23] = cube.stickers[4];
    cube.stickers[23].rotate_cw();
    cube.stickers[23].rotate_cw();
    cube.stickers[21] = cube.stickers[6];
    cube.stickers[21].rotate_cw();
    cube.stickers[21].rotate_cw();

    // D <- F [+0]
    cube.stickers[4] = cube.stickers[16];
    cube.stickers[6] = cube.stickers[18];

    // F <- U [+0]
    cube.stickers[16] = temp0;
    cube.stickers[18] = temp1;
}

/// L面を反時計回りに回転
fn rotate_lp(cube: &mut Cube) {
    rotate_face_ccw(cube, 8, 3); // Left face (orient +3)

    let temp0 = cube.stickers[0];
    let temp1 = cube.stickers[2];

    // U <- F [+0]
    cube.stickers[0] = cube.stickers[16];
    cube.stickers[2] = cube.stickers[18];

    // F <- D [+0]
    cube.stickers[16] = cube.stickers[4];
    cube.stickers[18] = cube.stickers[6];

    // D <- B [+2]
    cube.stickers[4] = cube.stickers[23];
    cube.stickers[4].rotate_cw();
    cube.stickers[4].rotate_cw();
    cube.stickers[6] = cube.stickers[21];
    cube.stickers[6].rotate_cw();
    cube.stickers[6].rotate_cw();

    // B <- U [+2]
    cube.stickers[23] = temp0;
    cube.stickers[23].rotate_cw();
    cube.stickers[23].rotate_cw();
    cube.stickers[21] = temp1;
    cube.stickers[21].rotate_cw();
    cube.stickers[21].rotate_cw();
}

/// U面を時計回りに回転
fn rotate_u(cube: &mut Cube) {
    rotate_face_cw(cube, 0, 1); // Up face (orient +1)

    let temp0 = cube.stickers[16];
    let temp1 = cube.stickers[17];

    cube.stickers[16] = cube.stickers[12];
    cube.stickers[17] = cube.stickers[13];

    cube.stickers[12] = cube.stickers[20];
    cube.stickers[13] = cube.stickers[21];

    cube.stickers[20] = cube.stickers[8];
    cube.stickers[21] = cube.stickers[9];

    cube.stickers[8] = temp0;
    cube.stickers[9] = temp1;
}

/// U面を反時計回りに回転
fn rotate_up(cube: &mut Cube) {
    rotate_face_ccw(cube, 0, 1); // Up face (orient +1)

    let temp0 = cube.stickers[16];
    let temp1 = cube.stickers[17];

    cube.stickers[16] = cube.stickers[8];
    cube.stickers[17] = cube.stickers[9];

    cube.stickers[8] = cube.stickers[20];
    cube.stickers[9] = cube.stickers[21];

    cube.stickers[20] = cube.stickers[12];
    cube.stickers[21] = cube.stickers[13];

    cube.stickers[12] = temp0;
    cube.stickers[13] = temp1;
}

/// D面を時計回りに回転
fn rotate_d(cube: &mut Cube) {
    rotate_face_cw(cube, 4, 1); // Down face (orient +1)

    let temp0 = cube.stickers[18];
    let temp1 = cube.stickers[19];

    cube.stickers[18] = cube.stickers[10];
    cube.stickers[19] = cube.stickers[11];

    cube.stickers[10] = cube.stickers[22];
    cube.stickers[11] = cube.stickers[23];

    cube.stickers[22] = cube.stickers[14];
    cube.stickers[23] = cube.stickers[15];

    cube.stickers[14] = temp0;
    cube.stickers[15] = temp1;
}

/// D面を反時計回りに回転
fn rotate_dp(cube: &mut Cube) {
    rotate_face_ccw(cube, 4, 1); // Down face (orient +1)

    let temp0 = cube.stickers[18];
    let temp1 = cube.stickers[19];

    cube.stickers[18] = cube.stickers[14];
    cube.stickers[19] = cube.stickers[15];

    cube.stickers[14] = cube.stickers[22];
    cube.stickers[15] = cube.stickers[23];

    cube.stickers[22] = cube.stickers[10];
    cube.stickers[23] = cube.stickers[11];

    cube.stickers[10] = temp0;
    cube.stickers[11] = temp1;
}

/// F面を時計回りに回転
fn rotate_f(cube: &mut Cube) {
    rotate_face_cw(cube, 16, 1); // Front face (orient +1)

    let temp0 = cube.stickers[2];
    let temp1 = cube.stickers[3];

    // U -> R -> D -> L -> U
    // F CW: idx 11->2 (L->U) orient 1. idx 2->12 (U->R) orient 3. idx 12->5 (R->D) orient 1. idx 5->11 (D->L) orient 3.
    cube.stickers[2] = cube.stickers[11];
    cube.stickers[2].rotate_cw();
    cube.stickers[3] = cube.stickers[9];
    cube.stickers[3].rotate_cw();

    cube.stickers[11] = cube.stickers[5];
    cube.stickers[11].rotate_ccw();
    cube.stickers[9] = cube.stickers[4];
    cube.stickers[9].rotate_ccw();

    cube.stickers[5] = cube.stickers[12];
    cube.stickers[5].rotate_cw();
    cube.stickers[4] = cube.stickers[14];
    cube.stickers[4].rotate_cw();

    let mut t0 = temp0;
    t0.rotate_ccw();
    let mut t1 = temp1;
    t1.rotate_ccw();
    cube.stickers[12] = t0;
    cube.stickers[14] = t1;
}

/// F面を反時計回りに回転
fn rotate_fp(cube: &mut Cube) {
    rotate_face_ccw(cube, 16, 1); // Front face (orient +1)

    let temp0 = cube.stickers[2];
    let temp1 = cube.stickers[3];

    // U -> L -> D -> R -> U
    // Fp CCW: idx 12->2 (R->U) orient 1. idx 2->11 (U->L) orient 3. idx 11->5 (L->D) orient 1. idx 5->12 (D->R) orient 3.
    cube.stickers[2] = cube.stickers[12];
    cube.stickers[2].rotate_cw();
    cube.stickers[3] = cube.stickers[14];
    cube.stickers[3].rotate_cw();

    cube.stickers[12] = cube.stickers[5];
    cube.stickers[12].rotate_ccw();
    cube.stickers[14] = cube.stickers[4];
    cube.stickers[14].rotate_ccw();

    cube.stickers[5] = cube.stickers[11];
    cube.stickers[5].rotate_cw();
    cube.stickers[4] = cube.stickers[9];
    cube.stickers[4].rotate_cw();

    let mut t0 = temp0;
    t0.rotate_ccw();
    let mut t1 = temp1;
    t1.rotate_ccw();
    cube.stickers[9] = t1; // U[3] moves to L-TL=9
    cube.stickers[11] = t0; // U[2] moves to L-BL=11
}

/// B面を時計回りに回転
fn rotate_b(cube: &mut Cube) {
    rotate_face_cw(cube, 20, 1); // Back face (orient +1)

    let temp0 = cube.stickers[0];
    let temp1 = cube.stickers[1];

    // U -> R -> D -> L -> U (CW from back)
    // rotate_bpの逆操作
    // Bp: U<-L(3), L<-D(1), D<-R(3), R<-U(1)
    // B:  U<-R(1), R<-D(3), D<-L(1), L<-U(3)

    // U <- R (+3)
    cube.stickers[0] = cube.stickers[13];
    cube.stickers[0].rotate_ccw();
    cube.stickers[1] = cube.stickers[15];
    cube.stickers[1].rotate_ccw();

    // R <- D (+1)
    cube.stickers[13] = cube.stickers[7];
    cube.stickers[13].rotate_cw();
    cube.stickers[15] = cube.stickers[6];
    cube.stickers[15].rotate_cw();

    // D <- L (+3)
    cube.stickers[7] = cube.stickers[10];
    cube.stickers[7].rotate_ccw();
    cube.stickers[6] = cube.stickers[8];
    cube.stickers[6].rotate_ccw();

    // L <- U (+1)
    cube.stickers[10] = temp0;
    cube.stickers[10].rotate_cw();
    cube.stickers[8] = temp1;
    cube.stickers[8].rotate_cw();
}

/// B面を反時計回りに回転
fn rotate_bp(cube: &mut Cube) {
    rotate_face_ccw(cube, 20, 1); // Back face (orient +1)

    let temp0 = cube.stickers[0];
    let temp1 = cube.stickers[1];

    // U -> R -> D -> L -> U (CCW from back)
    // B CW:  U<-R(3), R<-D(1), D<-L(3), L<-U(1)
    // Bp CCW: U<-L(3), L<-D(1), D<-R(3), R<-U(1)

    // U <- L (+3)
    cube.stickers[0] = cube.stickers[10];
    cube.stickers[0].rotate_ccw();
    cube.stickers[1] = cube.stickers[8];
    cube.stickers[1].rotate_ccw();

    // L <- D (+1)
    cube.stickers[10] = cube.stickers[7];
    cube.stickers[10].rotate_cw();
    cube.stickers[8] = cube.stickers[6];
    cube.stickers[8].rotate_cw();

    // D <- R (+3)
    cube.stickers[7] = cube.stickers[13];
    cube.stickers[7].rotate_ccw();
    cube.stickers[6] = cube.stickers[15];
    cube.stickers[6].rotate_ccw();

    // R <- U (+1)
    cube.stickers[13] = temp0;
    cube.stickers[13].rotate_cw();
    cube.stickers[15] = temp1;
    cube.stickers[15].rotate_cw();
}

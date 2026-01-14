use super::{Cube, Move};

struct MoveData {
    face_idx: usize,
    orientation_delta: u8,
    cycle: [usize; 8],
    rotations: [u8; 8],
}

const MOVE_DATA_TABLE: [(Move, MoveData); 6] = [
    (
        Move::U,
        MoveData {
            face_idx: 0,
            orientation_delta: 1,
            cycle: [16, 17, 12, 13, 20, 21, 8, 9],
            rotations: [0, 0, 0, 0, 0, 0, 0, 0],
        },
    ),
    (
        Move::D,
        MoveData {
            face_idx: 4,
            orientation_delta: 1,
            cycle: [18, 19, 10, 11, 22, 23, 14, 15],
            rotations: [0, 0, 0, 0, 0, 0, 0, 0],
        },
    ),
    (
        Move::L,
        MoveData {
            face_idx: 8,
            orientation_delta: 1,
            cycle: [0, 2, 23, 21, 4, 6, 16, 18],
            rotations: [2, 2, 2, 2, 0, 0, 0, 0],
        },
    ),
    (
        Move::R,
        MoveData {
            face_idx: 12,
            orientation_delta: 1,
            cycle: [1, 3, 17, 19, 5, 7, 22, 20],
            rotations: [0, 0, 0, 0, 2, 2, 2, 2],
        },
    ),
    (
        Move::F,
        MoveData {
            face_idx: 16,
            orientation_delta: 1,
            cycle: [2, 3, 11, 9, 5, 4, 12, 14],
            rotations: [1, 1, 1, 1, 1, 1, 1, 1],
        },
    ),
    (
        Move::B,
        MoveData {
            face_idx: 20,
            orientation_delta: 1,
            cycle: [0, 1, 13, 15, 7, 6, 10, 8],
            rotations: [3, 3, 3, 3, 3, 3, 3, 3],
        },
    ),
];

/// 指定された回転操作をキューブに適用します。
///
/// この関数は2x2ルービックキューブの6つの面（U, D, L, R, F, B）に対する
/// 回転操作を実行します。90度回転、逆回転（90度反時計回り）、
/// 180度回転の3種類がサポートされています。
///
/// # 引数
///
/// - `cube` - 操作対象のキューブへの可変参照
/// - `mv` - 実行する回転操作（Move enum）
///
/// # 例
///
/// ```
/// use rubiks_cube_2x2::cube::{Cube, Move};
/// use rubiks_cube_2x2::cube::rotation::apply_move;
///
/// let mut cube = Cube::new();
/// apply_move(&mut cube, Move::R);  // R面を90度時計回り
/// apply_move(&mut cube, Move::Up); // U面を90度反時計回り
/// ```
///
/// # 実装詳細
///
/// - 90度回転（R, L, U, D, F, B）: 基本回転を1回実行
/// - 180度回転（R2, L2, U2, D2, F2, B2）: 基本回転を2回実行
/// - 逆回転（Rp, Lp, Up, Dp, Fp, Bp）: 基本回転を3回実行
pub fn apply_move(cube: &mut Cube, mv: Move) {
    let base_move = match mv {
        Move::U | Move::Up | Move::U2 => Move::U,
        Move::D | Move::Dp | Move::D2 => Move::D,
        Move::L | Move::Lp | Move::L2 => Move::L,
        Move::R | Move::Rp | Move::R2 => Move::R,
        Move::F | Move::Fp | Move::F2 => Move::F,
        Move::B | Move::Bp | Move::B2 => Move::B,
    };

    let data = MOVE_DATA_TABLE
        .iter()
        .find(|(m, _)| *m == base_move)
        .map(|(_, d)| d)
        .expect("All moves should be in the table");

    let repeat = match mv {
        Move::U | Move::D | Move::L | Move::R | Move::F | Move::B => 1,
        Move::U2 | Move::D2 | Move::L2 | Move::R2 | Move::F2 | Move::B2 => 2,
        Move::Up | Move::Dp | Move::Lp | Move::Rp | Move::Fp | Move::Bp => 3,
    };

    for _ in 0..repeat {
        rotate_internal(
            cube,
            data.face_idx,
            data.orientation_delta,
            &data.cycle,
            &data.rotations,
        );
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

/// ランダムなスクランブルを生成してキューブに適用します。
///
/// 指定された手数分のランダムな回転操作を実行し、キューブをスクランブル状態にします。
/// 各手は18種類の可能な操作（R, Rp, R2, L, Lp, L2, ...）からランダムに選択されます。
///
/// # 引数
///
/// - `cube` - スクランブルするキューブへの可変参照
/// - `moves` - 実行するランダム操作の回数
///
/// # 例
///
/// ```
/// use rubiks_cube_2x2::cube::Cube;
/// use rubiks_cube_2x2::cube::rotation::scramble;
///
/// let mut cube = Cube::new();
/// scramble(&mut cube, 10);  // 10手のランダムスクランブル
/// assert!(!cube.is_solved()); // ほぼ確実に未完成状態
/// ```
///
/// # 注意
///
/// - 連続する手で互いに逆操作になる可能性があります（例: R直後にRp）
/// - より複雑なスクランブル（逆操作を避けるなど）が必要な場合は、
///   別途カスタムロジックを実装してください
pub fn scramble(cube: &mut Cube, moves: usize) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let all_moves = Move::all_moves();

    for _ in 0..moves {
        let mv = all_moves[rng.gen_range(0..all_moves.len())];
        apply_move(cube, mv);
    }
}

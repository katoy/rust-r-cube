use super::coord::RawCube;
use std::sync::OnceLock;

/// Kociemba アルゴリズムで使用する遷移テーブル (Move Tables)。
///
/// 探索中に毎回キューブを物理的に回転させるのは非常に重い処理であるため、
/// 各状態（Twist, Flip, Permutation等）が特定の回転操作によって
/// どの状態に遷移するかを事前にすべて計算し、配列として保持します。
pub struct MoveTable {
    /// コーナーの向き (3^7 = 2187通り) の遷移テーブル
    pub twist: Box<[[u16; 18]; 2187]>,
    /// エッジの向き (2^11 = 2048通り) の遷移テーブル
    pub flip: Box<[[u16; 18]; 2048]>,
    /// 中層エッジの所属 (12C4 = 495通り) の遷移テーブル
    pub ud_slice: Box<[[u16; 18]; 495]>,
    /// コーナーの配置 (8! = 40320通り) の遷移テーブル
    pub cp: Box<[[u16; 18]; 40320]>,
    /// Phase 2 用のエッジの配置 (8! = 40320通り) の遷移テーブル
    pub ep8: Box<[[u16; 18]; 40320]>,
    /// Phase 2 用の Slice パーツの置換 (24通り) の遷移テーブル
    pub slice_p: Box<[[u16; 18]; 24]>,
}

/// IDA* 探索で使用する枝刈りテーブル (Pruning Tables)。
///
/// 各状態から完成（または目標状態）までの最短手数の「下限値」を保持します。
/// 探索中、[現在の手数 + 枝刈りテーブルの値 > 制限手数] となった場合、
/// その先を探索しても解が見つからないことが保証されるため、探索を打ち切ることができます。
pub struct PruningTable {
    /// Twist と Slice を組み合わせた Phase 1 用の枝刈りテーブル
    pub twist_slice: Box<[u8]>,
    /// Flip と Slice を組み合わせた Phase 1 用の枝刈りテーブル
    pub flip_slice: Box<[u8]>,
    /// コーナー配置と Slice 配置を組み合わせた Phase 2 用の枝刈りテーブル
    pub cp_slice: Box<[u8]>,
    /// エッジ配置と Slice 配置を組み合わせた Phase 2 用の枝刈りテーブル
    pub ep8_slice: Box<[u8]>,
}

impl MoveTable {
    /// 移動テーブルを取得します。初回呼び出し時に生成（事前計算）されます。
    pub fn get() -> &'static MoveTable {
        static TABLE: OnceLock<MoveTable> = OnceLock::new();
        TABLE.get_or_init(|| MoveTable {
            twist: generate_twist_move_table(),
            flip: generate_flip_move_table(),
            ud_slice: generate_ud_slice_move_table(),
            cp: generate_cp_move_table(),
            ep8: generate_ep8_move_table(),
            slice_p: generate_slice_p_move_table(),
        })
    }
}

impl PruningTable {
    /// 枝刈りテーブルを取得します。初回呼び出し時に幅優先探索（BFS）を用いて生成されます。
    pub fn get() -> &'static PruningTable {
        static TABLE: OnceLock<PruningTable> = OnceLock::new();
        let move_table = MoveTable::get();
        TABLE.get_or_init(|| PruningTable {
            twist_slice: generate_twist_slice_pruning_table(move_table),
            flip_slice: generate_flip_slice_pruning_table(move_table),
            cp_slice: generate_cp_slice_pruning_table(move_table),
            ep8_slice: generate_ep8_slice_pruning_table(move_table),
        })
    }
}

fn generate_twist_move_table() -> Box<[[u16; 18]; 2187]> {
    let mut table: Vec<[u16; 18]> = vec![[0u16; 18]; 2187];
    let mut rc = RawCube::default();
    for (i, table_i) in table.iter_mut().enumerate() {
        rc.set_twist(i as u16);
        for m in 0..6 {
            let mut move_rc = rc;
            for r in 0..3 {
                move_rc = move_rc.multiply(RawCube::move_cube(m));
                table_i[m * 3 + r] = move_rc.get_twist();
            }
        }
    }
    table.into_boxed_slice().try_into().unwrap()
}

fn generate_flip_move_table() -> Box<[[u16; 18]; 2048]> {
    let mut table: Vec<[u16; 18]> = vec![[0u16; 18]; 2048];
    let mut rc = RawCube::default();
    for (i, table_i) in table.iter_mut().enumerate() {
        rc.set_flip(i as u16);
        for m in 0..6 {
            let mut move_rc = rc;
            for r in 0..3 {
                move_rc = move_rc.multiply(RawCube::move_cube(m));
                table_i[m * 3 + r] = move_rc.get_flip();
            }
        }
    }
    table.into_boxed_slice().try_into().unwrap()
}

fn generate_ud_slice_move_table() -> Box<[[u16; 18]; 495]> {
    let mut table: Vec<[u16; 18]> = vec![[0u16; 18]; 495];
    let mut rc = RawCube::default();
    for (i, table_i) in table.iter_mut().enumerate() {
        rc.set_ud_slice(i as u16);
        for m in 0..6 {
            let mut move_rc = rc;
            for r in 0..3 {
                move_rc = move_rc.multiply(RawCube::move_cube(m));
                table_i[m * 3 + r] = move_rc.get_ud_slice();
            }
        }
    }
    table.into_boxed_slice().try_into().unwrap()
}

fn generate_cp_move_table() -> Box<[[u16; 18]; 40320]> {
    let mut table: Vec<[u16; 18]> = vec![[0u16; 18]; 40320];
    let mut rc = RawCube::default();
    for (i, table_i) in table.iter_mut().enumerate() {
        rc.set_cp(i as u16);
        for m in 0..6 {
            let mut move_rc = rc;
            for r in 0..3 {
                move_rc = move_rc.multiply(RawCube::move_cube(m));
                table_i[m * 3 + r] = move_rc.get_cp();
            }
        }
    }
    table.into_boxed_slice().try_into().unwrap()
}

fn generate_ep8_move_table() -> Box<[[u16; 18]; 40320]> {
    let mut table: Vec<[u16; 18]> = vec![[0u16; 18]; 40320];
    let mut rc = RawCube::default();
    for (i, table_i) in table.iter_mut().enumerate() {
        rc.set_ep8(i as u16);
        for m in 0..6 {
            let mut move_rc = rc;
            for r in 0..3 {
                move_rc = move_rc.multiply(RawCube::move_cube(m));
                table_i[m * 3 + r] = move_rc.get_ep8();
            }
        }
    }
    table.into_boxed_slice().try_into().unwrap()
}

fn generate_slice_p_move_table() -> Box<[[u16; 18]; 24]> {
    let mut table: Vec<[u16; 18]> = vec![[0u16; 18]; 24];
    let mut rc = RawCube::default();
    for (i, table_i) in table.iter_mut().enumerate() {
        rc.set_slice_p(i as u16);
        for m in 0..6 {
            let mut move_rc = rc;
            for r in 0..3 {
                move_rc = move_rc.multiply(RawCube::move_cube(m));
                table_i[m * 3 + r] = move_rc.get_slice_p();
            }
        }
    }
    table.into_boxed_slice().try_into().unwrap()
}

fn generate_twist_slice_pruning_table(mt: &MoveTable) -> Box<[u8]> {
    let size1 = 2187;
    let size2 = 495;
    let total_size = size1 * size2;
    let mut table = vec![255u8; total_size];

    // ソルブ状態: twist=0, ud_slice=0
    let initial1 = 0;
    let initial2 = 0;
    table[initial1 * size2 + initial2] = 0;

    let mut distance = 0;
    let mut count = 1;
    while count < total_size {
        let mut found = false;
        for i in 0..total_size {
            if table[i] == distance {
                let s1 = i / size2;
                let s2 = i % size2;
                for m in 0..18 {
                    let ns1 = mt.twist[s1][m] as usize;
                    let ns2 = mt.ud_slice[s2][m] as usize;
                    let idx = ns1 * size2 + ns2;
                    if table[idx] == 255 {
                        table[idx] = distance + 1;
                        count += 1;
                        found = true;
                    }
                }
            }
        }
        if !found {
            break;
        }
        distance += 1;
    }
    table.into_boxed_slice()
}

fn generate_flip_slice_pruning_table(mt: &MoveTable) -> Box<[u8]> {
    let size1 = 2048;
    let size2 = 495;
    let total_size = size1 * size2;
    let mut table = vec![255u8; total_size];

    let initial1 = 0;
    let initial2 = 0;
    table[initial1 * size2 + initial2] = 0;

    let mut distance = 0;
    let mut count = 1;
    while count < total_size {
        let mut found = false;
        for i in 0..total_size {
            if table[i] == distance {
                let s1 = i / size2;
                let s2 = i % size2;
                for m in 0..18 {
                    let ns1 = mt.flip[s1][m] as usize;
                    let ns2 = mt.ud_slice[s2][m] as usize;
                    let idx = ns1 * size2 + ns2;
                    if table[idx] == 255 {
                        table[idx] = distance + 1;
                        count += 1;
                        found = true;
                    }
                }
            }
        }
        if !found {
            break;
        }
        distance += 1;
    }
    table.into_boxed_slice()
}

fn generate_cp_slice_pruning_table(mt: &MoveTable) -> Box<[u8]> {
    let size1 = 40320;
    let size2 = 24;
    let total_size = size1 * size2;
    let mut table = vec![255u8; total_size];

    let initial1 = 0;
    let initial2 = 0;
    table[initial1 * size2 + initial2] = 0;

    // Phase 2 許可移動: U(0,1,2), R2(4), F2(7), D(9,10,11), L2(13), B2(16)
    let allowed_moves = [0, 1, 2, 4, 7, 9, 10, 11, 13, 16];

    let mut distance = 0;
    let mut count = 1;
    while count < total_size {
        let mut found = false;
        for i in 0..total_size {
            if table[i] == distance {
                let s1 = i / size2;
                let s2 = i % size2;
                for &m in &allowed_moves {
                    let ns1 = mt.cp[s1][m] as usize;
                    let ns2 = mt.slice_p[s2][m] as usize;
                    let idx = ns1 * size2 + ns2;
                    if table[idx] == 255 {
                        table[idx] = distance + 1;
                        count += 1;
                        found = true;
                    }
                }
            }
        }
        if !found {
            break;
        }
        distance += 1;
    }
    table.into_boxed_slice()
}

fn generate_ep8_slice_pruning_table(mt: &MoveTable) -> Box<[u8]> {
    let size1 = 40320;
    let size2 = 24;
    let total_size = size1 * size2;
    let mut table = vec![255u8; total_size];

    let initial1 = 0;
    let initial2 = 0;
    table[initial1 * size2 + initial2] = 0;

    // Phase 2 許可移動: U(0,1,2), R2(4), F2(7), D(9,10,11), L2(13), B2(16)
    let allowed_moves = [0, 1, 2, 4, 7, 9, 10, 11, 13, 16];

    let mut distance = 0;
    let mut count = 1;
    while count < total_size {
        let mut found = false;
        for i in 0..total_size {
            if table[i] == distance {
                let s1 = i / size2;
                let s2 = i % size2;
                for &m in &allowed_moves {
                    let ns1 = mt.ep8[s1][m] as usize;
                    let ns2 = mt.slice_p[s2][m] as usize;
                    let idx = ns1 * size2 + ns2;
                    if table[idx] == 255 {
                        table[idx] = distance + 1;
                        count += 1;
                        found = true;
                    }
                }
            }
        }
        if !found {
            break;
        }
        distance += 1;
    }
    table.into_boxed_slice()
}

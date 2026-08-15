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
    /// Twist と Slice を組み合わせた Phase 1 用の枝刈りテーブル (Symmetryにより半減)
    pub twist_slice: Box<[u8]>,
    /// Flip と Slice を組み合わせた Phase 1 用の枝刈りテーブル (Symmetryにより半減)
    pub flip_slice: Box<[u8]>,
    /// コーナー配置と Slice 配置を組み合わせた Phase 2 用の枝刈りテーブル
    pub cp_slice: Box<[u8]>,
    /// エッジ配置と Slice 配置を組み合わせた Phase 2 用の枝刈りテーブル
    pub ep8_slice: Box<[u8]>,

    // 対称関係のマップ
    pub twist_class: Box<[u16; 2187]>,
    pub twist_sym: Box<[bool; 2187]>,
    pub twist_self_sym: Box<[bool; 2187]>,
    pub flip_class: Box<[u16; 2048]>,
    pub flip_sym: Box<[bool; 2048]>,
    pub flip_self_sym: Box<[bool; 2048]>,
    pub ud_slice_x2: Box<[u16; 495]>,
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
        TABLE.get_or_init(|| {
            let (twist_class, twist_sym, twist_self_sym, flip_class, flip_sym, flip_self_sym, ud_slice_x2) = generate_x2_maps();
            PruningTable {
                twist_slice: generate_twist_slice_pruning_table(move_table, &twist_class, &twist_sym, &twist_self_sym, &ud_slice_x2),
                flip_slice: generate_flip_slice_pruning_table(move_table, &flip_class, &flip_sym, &flip_self_sym, &ud_slice_x2),
                cp_slice: generate_cp_slice_pruning_table(move_table),
                ep8_slice: generate_ep8_slice_pruning_table(move_table),
                twist_class,
                twist_sym,
                twist_self_sym,
                flip_class,
                flip_sym,
                flip_self_sym,
                ud_slice_x2,
            }
        })
    }

    pub fn get_twist_slice(&self, twist: usize, slice: usize) -> u8 {
        let c = self.twist_class[twist] as usize;
        let s = if self.twist_self_sym[twist] {
            slice.min(self.ud_slice_x2[slice] as usize)
        } else if self.twist_sym[twist] {
            self.ud_slice_x2[slice] as usize
        } else {
            slice
        };
        self.twist_slice[c * 495 + s]
    }

    pub fn get_flip_slice(&self, flip: usize, slice: usize) -> u8 {
        let c = self.flip_class[flip] as usize;
        let s = if self.flip_self_sym[flip] {
            slice.min(self.ud_slice_x2[slice] as usize)
        } else if self.flip_sym[flip] {
            self.ud_slice_x2[slice] as usize
        } else {
            slice
        };
        self.flip_slice[c * 495 + s]
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

fn generate_x2_maps() -> (
    Box<[u16; 2187]>,
    Box<[bool; 2187]>,
    Box<[bool; 2187]>,
    Box<[u16; 2048]>,
    Box<[bool; 2048]>,
    Box<[bool; 2048]>,
    Box<[u16; 495]>,
) {
    use crate::kociemba::coord::Corner;
    use crate::kociemba::coord::Edge;

    let x2_cube = RawCube {
        cp: [
            Corner::DBL,
            Corner::DRB,
            Corner::DFR,
            Corner::DLF,
            Corner::ULB,
            Corner::UBR,
            Corner::UFR,
            Corner::UFL,
        ],
        co: [0, 0, 0, 0, 0, 0, 0, 0],
        ep: [
            Edge::DL,
            Edge::DB,
            Edge::DR,
            Edge::DF,
            Edge::UL,
            Edge::UB,
            Edge::UR,
            Edge::UF,
            Edge::BL,
            Edge::BR,
            Edge::FR,
            Edge::FL,
        ],
        eo: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    };

    let mut twist_class = Box::new([0u16; 2187]);
    let mut twist_sym = Box::new([false; 2187]);
    let mut twist_self_sym = Box::new([false; 2187]);
    let mut twist_seen = [false; 2187];
    let mut next_twist_class = 0;

    let mut rc = RawCube::default();
    for i in 0..2187 {
        if twist_seen[i] {
            continue;
        }
        rc.set_twist(i as u16);
        let sym_rc = rc.multiply(&x2_cube);
        let sym_i = sym_rc.get_twist() as usize;

        twist_seen[i] = true;
        twist_seen[sym_i] = true;

        let repr = i.min(sym_i);
        let class_id = next_twist_class;
        next_twist_class += 1;

        twist_class[i] = class_id;
        twist_class[sym_i] = class_id;

        twist_sym[i] = i != repr;
        twist_sym[sym_i] = sym_i != repr;

        twist_self_sym[i] = i == sym_i;
        twist_self_sym[sym_i] = i == sym_i;
    }

    let mut flip_class = Box::new([0u16; 2048]);
    let mut flip_sym = Box::new([false; 2048]);
    let mut flip_self_sym = Box::new([false; 2048]);
    let mut flip_seen = [false; 2048];
    let mut next_flip_class = 0;

    for i in 0..2048 {
        if flip_seen[i] {
            continue;
        }
        rc.set_flip(i as u16);
        let sym_rc = rc.multiply(&x2_cube);
        let sym_i = sym_rc.get_flip() as usize;

        flip_seen[i] = true;
        flip_seen[sym_i] = true;

        let repr = i.min(sym_i);
        let class_id = next_flip_class;
        next_flip_class += 1;

        flip_class[i] = class_id;
        flip_class[sym_i] = class_id;

        flip_sym[i] = i != repr;
        flip_sym[sym_i] = sym_i != repr;

        flip_self_sym[i] = i == sym_i;
        flip_self_sym[sym_i] = i == sym_i;
    }

    let mut ud_slice_x2 = Box::new([0u16; 495]);
    for i in 0..495 {
        rc.set_ud_slice(i as u16);
        let sym_rc = rc.multiply(&x2_cube);
        ud_slice_x2[i] = sym_rc.get_ud_slice();
    }

    (
        twist_class,
        twist_sym,
        twist_self_sym,
        flip_class,
        flip_sym,
        flip_self_sym,
        ud_slice_x2,
    )
}

fn generate_twist_slice_pruning_table(
    mt: &MoveTable,
    twist_class: &[u16; 2187],
    twist_sym: &[bool; 2187],
    twist_self_sym: &[bool; 2187],
    ud_slice_x2: &[u16; 495],
) -> Box<[u8]> {
    let num_classes = *twist_class.iter().max().unwrap() as usize + 1;
    let size2 = 495;
    let total_size = num_classes * size2;
    let mut table = vec![255u8; total_size];

    let init_class = twist_class[0] as usize;
    let init_slice = 0;
    table[init_class * size2 + init_slice] = 0;

    let mut distance = 0;
    let mut count = 1;

    let mut class_to_twist = vec![0u16; num_classes];
    for twist in 0..2187 {
        if !twist_sym[twist] {
            class_to_twist[twist_class[twist] as usize] = twist as u16;
        }
    }

    while count < total_size {
        let mut found = false;
        for c in 0..num_classes {
            for s2 in 0..size2 {
                let idx = c * size2 + s2;
                if table[idx] == distance {
                    let s1 = class_to_twist[c] as usize;
                    for m in 0..18 {
                        let ns1 = mt.twist[s1][m] as usize;
                        let ns2 = mt.ud_slice[s2][m] as usize;

                        let nc = twist_class[ns1] as usize;
                        let n_slice = if twist_self_sym[ns1] {
                            ns2.min(ud_slice_x2[ns2] as usize)
                        } else if twist_sym[ns1] {
                            ud_slice_x2[ns2] as usize
                        } else {
                            ns2
                        };

                        let n_idx = nc * size2 + n_slice;
                        if table[n_idx] == 255 {
                            table[n_idx] = distance + 1;
                            count += 1;
                            found = true;
                        }
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

fn generate_flip_slice_pruning_table(
    mt: &MoveTable,
    flip_class: &[u16; 2048],
    flip_sym: &[bool; 2048],
    flip_self_sym: &[bool; 2048],
    ud_slice_x2: &[u16; 495],
) -> Box<[u8]> {
    let num_classes = *flip_class.iter().max().unwrap() as usize + 1;
    let size2 = 495;
    let total_size = num_classes * size2;
    let mut table = vec![255u8; total_size];

    let init_class = flip_class[0] as usize;
    let init_slice = 0;
    table[init_class * size2 + init_slice] = 0;

    let mut distance = 0;
    let mut count = 1;

    let mut class_to_flip = vec![0u16; num_classes];
    for flip in 0..2048 {
        if !flip_sym[flip] {
            class_to_flip[flip_class[flip] as usize] = flip as u16;
        }
    }

    while count < total_size {
        let mut found = false;
        for c in 0..num_classes {
            for s2 in 0..size2 {
                let idx = c * size2 + s2;
                if table[idx] == distance {
                    let s1 = class_to_flip[c] as usize;
                    for m in 0..18 {
                        let ns1 = mt.flip[s1][m] as usize;
                        let ns2 = mt.ud_slice[s2][m] as usize;

                        let nc = flip_class[ns1] as usize;
                        let n_slice = if flip_self_sym[ns1] {
                            ns2.min(ud_slice_x2[ns2] as usize)
                        } else if flip_sym[ns1] {
                            ud_slice_x2[ns2] as usize
                        } else {
                            ns2
                        };

                        let n_idx = nc * size2 + n_slice;
                        if table[n_idx] == 255 {
                            table[n_idx] = distance + 1;
                            count += 1;
                            found = true;
                        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pruning_tables_unreachable_break() {
        let mt = MoveTable {
            twist: vec![[0u16; 18]; 2187].into_boxed_slice().try_into().unwrap(),
            flip: vec![[0u16; 18]; 2048].into_boxed_slice().try_into().unwrap(),
            ud_slice: vec![[0u16; 18]; 495].into_boxed_slice().try_into().unwrap(),
            cp: vec![[0u16; 18]; 40320].into_boxed_slice().try_into().unwrap(),
            ep8: vec![[0u16; 18]; 40320].into_boxed_slice().try_into().unwrap(),
            slice_p: vec![[0u16; 18]; 24].into_boxed_slice().try_into().unwrap(),
        };

        let (twist_class, twist_sym, twist_self_sym, flip_class, flip_sym, flip_self_sym, ud_slice_x2) = generate_x2_maps();
        let _ = generate_twist_slice_pruning_table(&mt, &twist_class, &twist_sym, &twist_self_sym, &ud_slice_x2);
        let _ = generate_flip_slice_pruning_table(&mt, &flip_class, &flip_sym, &flip_self_sym, &ud_slice_x2);
        let _ = generate_cp_slice_pruning_table(&mt);
        let _ = generate_ep8_slice_pruning_table(&mt);
    }

    #[test]
    fn test_x2_symmetry_maps() {
        let (twist_class, _twist_sym, _twist_self_sym, flip_class, _flip_sym, _flip_self_sym, _ud_slice_x2) = generate_x2_maps();
        let max_twist_class = twist_class.iter().max().unwrap();
        assert_eq!(*max_twist_class, 1106);

        let max_flip_class = flip_class.iter().max().unwrap();
        assert_eq!(*max_flip_class, 1055);
    }

    #[test]
    fn test_pruning_table_symmetry_getters() {
        let pruning = PruningTable::get();
        let (_, _, _, _, _, _, ud_slice_x2) = generate_x2_maps();
        
        for t in 0..2187 {
            let mut rc = RawCube::default();
            rc.set_twist(t as u16);
            let sym_t = {
                use crate::kociemba::coord::Corner;
                use crate::kociemba::coord::Edge;
                let x2_cube = RawCube {
                    cp: [Corner::DBL, Corner::DRB, Corner::DFR, Corner::DLF, Corner::ULB, Corner::UBR, Corner::UFR, Corner::UFL],
                    co: [0, 0, 0, 0, 0, 0, 0, 0],
                    ep: [Edge::DL, Edge::DB, Edge::DR, Edge::DF, Edge::UL, Edge::UB, Edge::UR, Edge::UF, Edge::BL, Edge::BR, Edge::FR, Edge::FL],
                    eo: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                };
                rc.multiply(&x2_cube).get_twist() as usize
            };
            
            for s in [0, 100, 200, 300, 494] {
                let d1 = pruning.get_twist_slice(t, s);
                let sym_s = ud_slice_x2[s] as usize;
                let d2 = pruning.get_twist_slice(sym_t, sym_s);
                assert_eq!(d1, d2, "twist={} と sym_twist={} (slice={}, sym_slice={}) で距離が一致しません", t, sym_t, s, sym_s);
            }
        }
    }
}

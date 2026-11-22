use super::coord::{move_cube_18, rank_perm4, rank_perm8, unrank_perm4, unrank_perm8, RawCube};
use std::borrow::Cow;
use std::sync::OnceLock;
include!("table_io.rs");

/// Phase 2 で許可される 10 通りの基本手（0..18 の移動インデックス）
/// U(0,1,2), R2(4), F2(7), D(9,10,11), L2(13), B2(16)
pub const PHASE2_MOVES: [usize; 10] = [0, 1, 2, 4, 7, 9, 10, 11, 13, 16];

pub const TWIST_SLICE_STATES: usize = 2187 * 495; // 1_082_565
pub const FLIP_SLICE_STATES: usize = 2048 * 495; // 1_013_760
pub const CP_SLICE_STATES: usize = 40320 * 24; //   967_680
pub const EP8_SLICE_STATES: usize = 40320 * 24; //   967_680

pub const TWIST_SLICE_PACKED_LEN: usize = TWIST_SLICE_STATES.div_ceil(2); // 541_283
pub const FLIP_SLICE_PACKED_LEN: usize = FLIP_SLICE_STATES.div_ceil(2); // 506_880
pub const CP_SLICE_PACKED_LEN: usize = CP_SLICE_STATES.div_ceil(2); // 483_840
pub const EP8_SLICE_PACKED_LEN: usize = EP8_SLICE_STATES.div_ceil(2); // 483_840

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
    /// コーナーの配置 (8! = 40320通り) の Phase 2 (10手) 遷移テーブル
    pub cp: Box<[[u16; 10]; 40320]>,
    /// Phase 2 用のエッジの配置 (8! = 40320通り) の Phase 2 (10手) 遷移テーブル
    pub ep8: Box<[[u16; 10]; 40320]>,
    /// Phase 2 用の Slice パーツの置換 (24通り) の Phase 2 (10手) 遷移テーブル
    pub slice_p: Box<[[u16; 10]; 24]>,
}

/// IDA* 探索で使用する枝刈りテーブル (Pruning Tables)。
///
/// 各状態から完成（または目標状態）までの最短手数の「下限値」を 4-bit パック直接配列で保持します。
/// WASM 本番実行時は静的バイト列からゼロコピー (`Cow::Borrowed`) で参照します。
pub struct PruningTable {
    /// Twist と Slice を組み合わせた Phase 1 用の 4-bit パック枝刈りテーブル
    pub twist_slice: Cow<'static, [u8]>,
    /// Flip と Slice を組み合わせた Phase 1 用の 4-bit パック枝刈りテーブル
    pub flip_slice: Cow<'static, [u8]>,
    /// コーナー配置と Slice 配置を組み合わせた Phase 2 用の 4-bit パック枝刈りテーブル
    pub cp_slice: Cow<'static, [u8]>,
    /// エッジ配置と Slice 配置を組み合わせた Phase 2 用の 4-bit パック枝刈りテーブル
    pub ep8_slice: Cow<'static, [u8]>,
}

impl MoveTable {
    /// 移動テーブルを取得します。初回呼び出し時に構築されます。
    pub fn get() -> &'static MoveTable {
        static TABLE: OnceLock<MoveTable> = OnceLock::new();
        TABLE.get_or_init(build_move_table)
    }
}

/// 移動テーブルをゼロから生成します。
pub(crate) fn build_move_table() -> MoveTable {
    MoveTable {
        twist: generate_twist_move_table(),
        flip: generate_flip_move_table(),
        ud_slice: generate_ud_slice_move_table(),
        cp: generate_cp_move_table(),
        ep8: generate_ep8_move_table(),
        slice_p: generate_slice_p_move_table(),
    }
}

impl PruningTable {
    /// 枝刈りテーブルを取得します。静的アセットがあればゼロコピーで復号し、なければ BFS で構築します。
    pub fn get() -> &'static PruningTable {
        static TABLE: OnceLock<PruningTable> = OnceLock::new();
        TABLE.get_or_init(|| match crate::TABLE_BYTES {
            Some(bytes) => decode_pruning(bytes),
            None => build_pruning_table(MoveTable::get()),
        })
    }
}

/// 枝刈りテーブルをゼロから生成します（事前計算バイナリなし）。
pub(crate) fn build_pruning_table(move_table: &MoveTable) -> PruningTable {
    PruningTable {
        twist_slice: Cow::Owned(generate_twist_slice_pruning_table(move_table).into_vec()),
        flip_slice: Cow::Owned(generate_flip_slice_pruning_table(move_table).into_vec()),
        cp_slice: Cow::Owned(generate_cp_slice_pruning_table(move_table).into_vec()),
        ep8_slice: Cow::Owned(generate_ep8_slice_pruning_table(move_table).into_vec()),
    }
}

#[inline(always)]
fn get_nibble(table: &[u8], idx: usize) -> u8 {
    (table[idx >> 1] >> ((idx & 1) << 2)) & 0x0f
}

fn pack_4bit(unpacked: &[u8]) -> Box<[u8]> {
    let mut packed = vec![0u8; unpacked.len().div_ceil(2)];
    for (idx, &val) in unpacked.iter().enumerate() {
        assert!(val <= 15, "4-bit pruning table value overflow: {val}");
        packed[idx >> 1] |= val << ((idx & 1) << 2);
    }
    packed.into_boxed_slice()
}

impl PruningTable {
    #[inline(always)]
    pub fn get_twist_slice(&self, twist: usize, slice: usize) -> u8 {
        get_nibble(&self.twist_slice, twist * 495 + slice)
    }

    #[inline(always)]
    pub fn get_flip_slice(&self, flip: usize, slice: usize) -> u8 {
        get_nibble(&self.flip_slice, flip * 495 + slice)
    }

    #[inline(always)]
    pub fn get_cp_slice(&self, cp: usize, slice_p: usize) -> u8 {
        get_nibble(&self.cp_slice, cp * 24 + slice_p)
    }

    #[inline(always)]
    pub fn get_ep8_slice(&self, ep8: usize, slice_p: usize) -> u8 {
        get_nibble(&self.ep8_slice, ep8 * 24 + slice_p)
    }
}

pub(crate) fn generate_twist_move_table() -> Box<[[u16; 18]; 2187]> {
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

pub(crate) fn generate_flip_move_table() -> Box<[[u16; 18]; 2048]> {
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

pub(crate) fn generate_ud_slice_move_table() -> Box<[[u16; 18]; 495]> {
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

fn generate_perm8_move_table(mv_perms: &[[u8; 8]; 10]) -> Box<[[u16; 10]; 40320]> {
    let mut table: Vec<[u16; 10]> = vec![[0u16; 10]; 40320];
    for (i, table_i) in table.iter_mut().enumerate() {
        let p = unrank_perm8(i as u16);
        for (k, mv) in mv_perms.iter().enumerate() {
            let q = [
                p[mv[0] as usize],
                p[mv[1] as usize],
                p[mv[2] as usize],
                p[mv[3] as usize],
                p[mv[4] as usize],
                p[mv[5] as usize],
                p[mv[6] as usize],
                p[mv[7] as usize],
            ];
            table_i[k] = rank_perm8(&q);
        }
    }
    table.into_boxed_slice().try_into().unwrap()
}

pub(crate) fn generate_cp_move_table() -> Box<[[u16; 10]; 40320]> {
    let mv_perms: [[u8; 8]; 10] = std::array::from_fn(|k| {
        let mc = move_cube_18(PHASE2_MOVES[k]);
        std::array::from_fn(|j| mc.cp[j] as u8)
    });
    generate_perm8_move_table(&mv_perms)
}

pub(crate) fn generate_ep8_move_table() -> Box<[[u16; 10]; 40320]> {
    let mv_perms: [[u8; 8]; 10] = std::array::from_fn(|k| {
        let mc = move_cube_18(PHASE2_MOVES[k]);
        std::array::from_fn(|j| {
            let e = mc.ep[j] as u8;
            assert!(e < 8);
            e
        })
    });
    generate_perm8_move_table(&mv_perms)
}

pub(crate) fn generate_slice_p_move_table() -> Box<[[u16; 10]; 24]> {
    let mv_perms: [[u8; 4]; 10] = std::array::from_fn(|k| {
        let mc = move_cube_18(PHASE2_MOVES[k]);
        std::array::from_fn(|j| mc.ep[8 + j] as u8 - 8)
    });
    let mut table = Box::new([[0u16; 10]; 24]);
    for (i, table_i) in table.iter_mut().enumerate() {
        let p = unrank_perm4(i as u16);
        for (k, mv) in mv_perms.iter().enumerate() {
            let q = [
                p[mv[0] as usize],
                p[mv[1] as usize],
                p[mv[2] as usize],
                p[mv[3] as usize],
            ];
            table_i[k] = rank_perm4(&q);
        }
    }
    table
}

pub(crate) fn generate_twist_slice_pruning_table(mt: &MoveTable) -> Box<[u8]> {
    let size2 = 495;
    let total_size = TWIST_SLICE_STATES;
    let mut table = vec![255u8; total_size];
    table[0] = 0;

    let mut distance = 0;
    let mut count = 1;
    while count < total_size {
        let mut found = false;
        for s1 in 0..2187 {
            let base = s1 * size2;
            for s2 in 0..size2 {
                if table[base + s2] == distance {
                    for m in 0..18 {
                        let ns1 = mt.twist[s1][m] as usize;
                        let ns2 = mt.ud_slice[s2][m] as usize;
                        let n_idx = ns1 * size2 + ns2;
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
    assert_eq!(count, total_size, "twist_slice pruning BFS incomplete");
    pack_4bit(&table)
}

pub(crate) fn generate_flip_slice_pruning_table(mt: &MoveTable) -> Box<[u8]> {
    let size2 = 495;
    let total_size = FLIP_SLICE_STATES;
    let mut table = vec![255u8; total_size];
    table[0] = 0;

    let mut distance = 0;
    let mut count = 1;
    while count < total_size {
        let mut found = false;
        for s1 in 0..2048 {
            let base = s1 * size2;
            for s2 in 0..size2 {
                if table[base + s2] == distance {
                    for m in 0..18 {
                        let ns1 = mt.flip[s1][m] as usize;
                        let ns2 = mt.ud_slice[s2][m] as usize;
                        let n_idx = ns1 * size2 + ns2;
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
    assert_eq!(count, total_size, "flip_slice pruning BFS incomplete");
    pack_4bit(&table)
}

pub(crate) fn generate_cp_slice_pruning_table(mt: &MoveTable) -> Box<[u8]> {
    let size2 = 24;
    let total_size = CP_SLICE_STATES;
    let mut table = vec![255u8; total_size];
    table[0] = 0;

    let mut distance = 0;
    let mut count = 1;
    while count < total_size {
        let mut found = false;
        for s1 in 0..40320 {
            let base = s1 * size2;
            for s2 in 0..size2 {
                if table[base + s2] == distance {
                    for p2_idx in 0..10 {
                        let ns1 = mt.cp[s1][p2_idx] as usize;
                        let ns2 = mt.slice_p[s2][p2_idx] as usize;
                        let idx = ns1 * size2 + ns2;
                        if table[idx] == 255 {
                            table[idx] = distance + 1;
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
    assert_eq!(count, total_size, "cp_slice pruning BFS incomplete");
    pack_4bit(&table)
}

pub(crate) fn generate_ep8_slice_pruning_table(mt: &MoveTable) -> Box<[u8]> {
    let size2 = 24;
    let total_size = EP8_SLICE_STATES;
    let mut table = vec![255u8; total_size];
    table[0] = 0;

    let mut distance = 0;
    let mut count = 1;
    while count < total_size {
        let mut found = false;
        for s1 in 0..40320 {
            let base = s1 * size2;
            for s2 in 0..size2 {
                if table[base + s2] == distance {
                    for p2_idx in 0..10 {
                        let ns1 = mt.ep8[s1][p2_idx] as usize;
                        let ns2 = mt.slice_p[s2][p2_idx] as usize;
                        let idx = ns1 * size2 + ns2;
                        if table[idx] == 255 {
                            table[idx] = distance + 1;
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
    assert_eq!(count, total_size, "ep8_slice pruning BFS incomplete");
    pack_4bit(&table)
}

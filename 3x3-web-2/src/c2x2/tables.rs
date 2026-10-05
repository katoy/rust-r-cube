use super::coord::{
    get_group_a_idx, get_group_b_idx, get_group_c_idx, get_group_d_idx, move_cube_18, RawCube,
};
use std::collections::VecDeque;
use std::sync::OnceLock;

/// 2x2 コーナーの遷移テーブル
pub struct MoveTable {
    pub cp: Box<[[u16; 18]; 40320]>,
    pub twist: Box<[[u16; 18]; 2187]>,
}

/// 4グループ統合 Pruning Table
pub struct PruningTable {
    pub group_a: Box<[u8]>,
    pub group_b: Box<[u8]>,
    pub group_c: Box<[u8]>,
    pub group_d: Box<[u8]>,
}

impl MoveTable {
    pub fn get() -> &'static MoveTable {
        static TABLE: OnceLock<MoveTable> = OnceLock::new();
        TABLE.get_or_init(|| MoveTable {
            cp: generate_cp_move_table(),
            twist: generate_twist_move_table(),
        })
    }
}

impl PruningTable {
    pub fn get() -> &'static PruningTable {
        static TABLE: OnceLock<PruningTable> = OnceLock::new();
        TABLE.get_or_init(|| PruningTable {
            group_a: generate_group_pruning_table(get_group_a_idx),
            group_b: generate_group_pruning_table(get_group_b_idx),
            group_c: generate_group_pruning_table(get_group_c_idx),
            group_d: generate_group_pruning_table(get_group_d_idx),
        })
    }
}

fn generate_cp_move_table() -> Box<[[u16; 18]; 40320]> {
    let mut table = vec![0u16; 40320 * 18];
    for i in 0..40320 {
        let mut rc = RawCube::default();
        rc.set_cp(i as u16);
        for m in 0..18 {
            let next_rc = rc.multiply(move_cube_18(m));
            table[i * 18 + m] = next_rc.get_cp();
        }
    }
    let ptr = Box::into_raw(table.into_boxed_slice()) as *mut [[u16; 18]; 40320];
    unsafe { Box::from_raw(ptr) }
}

fn generate_twist_move_table() -> Box<[[u16; 18]; 2187]> {
    let mut table = vec![0u16; 2187 * 18];
    for i in 0..2187 {
        let mut rc = RawCube::default();
        rc.set_twist(i as u16);
        for m in 0..18 {
            let next_rc = rc.multiply(move_cube_18(m));
            table[i * 18 + m] = next_rc.get_twist();
        }
    }
    let ptr = Box::into_raw(table.into_boxed_slice()) as *mut [[u16; 18]; 2187];
    unsafe { Box::from_raw(ptr) }
}

fn generate_group_pruning_table(idx_fn: fn(&RawCube) -> usize) -> Box<[u8]> {
    let mut table = vec![255u8; 136080];
    let start_rc = RawCube::default();
    let start_idx = idx_fn(&start_rc);
    table[start_idx] = 0;

    let mut q = VecDeque::with_capacity(136080);
    q.push_back((start_rc, 0u8));
    let mut visited_count = 1;

    while let Some((rc, dist)) = q.pop_front() {
        for m in 0..18 {
            let next_rc = rc.multiply(move_cube_18(m));
            let next_idx = idx_fn(&next_rc);
            if table[next_idx] == 255 {
                table[next_idx] = dist + 1;
                visited_count += 1;
                q.push_back((next_rc, dist + 1));
            }
        }
    }
    assert_eq!(
        visited_count, 136080,
        "2x2 pruning table generation incomplete"
    );
    table.into_boxed_slice()
}

use super::coord::{move_cube_18, RawCube};
use std::sync::OnceLock;

pub struct MoveTable {
    pub cp: Box<[[u16; 18]; 40320]>,
    pub twist: Box<[[u16; 18]; 2187]>,
}

pub struct PruningTable {
    pub cp: Box<[u8]>,
    pub twist: Box<[u8]>,
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
        TABLE.get_or_init(|| {
            let mt = MoveTable::get();
            PruningTable {
                cp: generate_cp_pruning_table(mt),
                twist: generate_twist_pruning_table(mt),
                group_a: generate_group_a_pruning_table(),
                group_b: generate_group_b_pruning_table(),
                group_c: generate_group_c_pruning_table(),
                group_d: generate_group_d_pruning_table(),
            }
        })
    }

    /// CPの枝刈り距離を取得します。
    pub fn get_cp_dist(&self, cp_idx: usize) -> u8 {
        self.cp[cp_idx]
    }

    /// Twistの枝刈り距離を取得します。
    pub fn get_twist_dist(&self, twist_idx: usize) -> u8 {
        self.twist[twist_idx]
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

fn generate_cp_pruning_table(mt: &MoveTable) -> Box<[u8]> {
    let mut table = vec![255u8; 40320];
    table[0] = 0;
    let mut distance = 0u8;
    let mut count = 1;
    // 2x2キューブのCP空間は連結なので、BFSは必ず全状態を網羅する。
    // ループは count が 40320 に達したとき自然に終了する。
    while count < 40320 {
        for i in 0..40320 {
            if table[i] == distance {
                for m in 0..18 {
                    let next = mt.cp[i][m] as usize;
                    if table[next] == 255 {
                        table[next] = distance + 1;
                        count += 1;
                    }
                }
            }
        }
        distance += 1;
    }
    table.into_boxed_slice()
}

fn generate_twist_pruning_table(mt: &MoveTable) -> Box<[u8]> {
    let mut table = vec![255u8; 2187];
    table[0] = 0;
    let mut distance = 0u8;
    let mut count = 1;
    // Twist空間（2187状態）も連結なので、BFSは必ず全状態を網羅する。
    while count < 2187 {
        for i in 0..2187 {
            if table[i] == distance {
                for m in 0..18 {
                    let next = mt.twist[i][m] as usize;
                    if table[next] == 255 {
                        table[next] = distance + 1;
                        count += 1;
                    }
                }
            }
        }
        distance += 1;
    }
    table.into_boxed_slice()
}

fn generate_group_a_pruning_table() -> Box<[u8]> {
    use super::coord::get_group_a_idx;
    use std::collections::VecDeque;

    let mut table = vec![255u8; 136080];
    let start_idx = get_group_a_idx(&RawCube::default());
    table[start_idx] = 0; // 初期状態
    let mut q = VecDeque::new();
    q.push_back((RawCube::default(), 0u8));
    let mut visited_count = 1;

    while let Some((rc, dist)) = q.pop_front() {
        for m in 0..18 {
            let next_rc = rc.multiply(move_cube_18(m));
            let next_idx = get_group_a_idx(&next_rc);
            if table[next_idx] == 255 {
                table[next_idx] = dist + 1;
                visited_count += 1;
                q.push_back((next_rc, dist + 1));
            }
        }
    }
    assert_eq!(
        visited_count, 136080,
        "Group A pruning table did not visit all states!"
    );
    table.into_boxed_slice()
}

fn generate_group_b_pruning_table() -> Box<[u8]> {
    use super::coord::get_group_b_idx;
    use std::collections::VecDeque;

    let mut table = vec![255u8; 136080];
    let start_idx = get_group_b_idx(&RawCube::default());
    table[start_idx] = 0; // 初期状態
    let mut q = VecDeque::new();
    q.push_back((RawCube::default(), 0u8));
    let mut visited_count = 1;

    while let Some((rc, dist)) = q.pop_front() {
        for m in 0..18 {
            let next_rc = rc.multiply(move_cube_18(m));
            let next_idx = get_group_b_idx(&next_rc);
            if table[next_idx] == 255 {
                table[next_idx] = dist + 1;
                visited_count += 1;
                q.push_back((next_rc, dist + 1));
            }
        }
    }
    assert_eq!(
        visited_count, 136080,
        "Group B pruning table did not visit all states!"
    );
    table.into_boxed_slice()
}

fn generate_group_c_pruning_table() -> Box<[u8]> {
    use super::coord::get_group_c_idx;
    use std::collections::VecDeque;

    let mut table = vec![255u8; 136080];
    let start_idx = get_group_c_idx(&RawCube::default());
    table[start_idx] = 0; // 初期状態
    let mut q = VecDeque::new();
    q.push_back((RawCube::default(), 0u8));
    let mut visited_count = 1;

    while let Some((rc, dist)) = q.pop_front() {
        for m in 0..18 {
            let next_rc = rc.multiply(move_cube_18(m));
            let next_idx = get_group_c_idx(&next_rc);
            if table[next_idx] == 255 {
                table[next_idx] = dist + 1;
                visited_count += 1;
                q.push_back((next_rc, dist + 1));
            }
        }
    }
    assert_eq!(
        visited_count, 136080,
        "Group C pruning table did not visit all states!"
    );
    table.into_boxed_slice()
}

fn generate_group_d_pruning_table() -> Box<[u8]> {
    use super::coord::get_group_d_idx;
    use std::collections::VecDeque;

    let mut table = vec![255u8; 136080];
    let start_idx = get_group_d_idx(&RawCube::default());
    table[start_idx] = 0; // 初期状態
    let mut q = VecDeque::new();
    q.push_back((RawCube::default(), 0u8));
    let mut visited_count = 1;

    while let Some((rc, dist)) = q.pop_front() {
        for m in 0..18 {
            let next_rc = rc.multiply(move_cube_18(m));
            let next_idx = get_group_d_idx(&next_rc);
            if table[next_idx] == 255 {
                table[next_idx] = dist + 1;
                visited_count += 1;
                q.push_back((next_rc, dist + 1));
            }
        }
    }
    assert_eq!(
        visited_count, 136080,
        "Group D pruning table did not visit all states!"
    );
    table.into_boxed_slice()
}

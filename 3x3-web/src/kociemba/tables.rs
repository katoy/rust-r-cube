use super::coord::RawCube;
use std::sync::OnceLock;

/// Kociemba アルゴリズムで使用する遷移テーブル (Move Tables)
pub struct MoveTable {
    pub twist: Box<[[u16; 18]; 2187]>,
    pub flip: Box<[[u16; 18]; 2048]>,
    pub ud_slice: Box<[[u16; 18]; 495]>,
    pub cp: Box<[[u16; 18]; 40320]>,
    pub ep8: Box<[[u16; 18]; 40320]>,
    pub slice_p: Box<[[u16; 18]; 24]>,
}

pub struct PruningTable {
    pub twist_slice: Box<[u8]>,
    pub flip_slice: Box<[u8]>,
    pub cp_slice: Box<[u8]>,
    pub ep8_slice: Box<[u8]>,
}

impl MoveTable {
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
    let mut table = vec![0u16; 2187 * 18];
    let mut rc = RawCube::default();
    for i in 0..2187 {
        rc.set_twist(i as u16);
        for m in 0..6 {
            let mut move_rc = rc;
            for r in 0..3 {
                move_rc = move_rc.multiply(RawCube::move_cube(m));
                table[i * 18 + m * 3 + r] = move_rc.get_twist();
            }
        }
    }
    // SAFETY: table は vec![0u16; 2187 * 18] から作られた Box<[u16]> で、
    // [[u16; 18]; 2187] と完全にメモリレイアウトが一致するため、transmute は安全
    let ptr = Box::into_raw(table.into_boxed_slice()) as *mut [[u16; 18]; 2187];
    unsafe { Box::from_raw(ptr) }
}

fn generate_flip_move_table() -> Box<[[u16; 18]; 2048]> {
    let mut table = vec![0u16; 2048 * 18];
    let mut rc = RawCube::default();
    for i in 0..2048 {
        rc.set_flip(i as u16);
        for m in 0..6 {
            let mut move_rc = rc;
            for r in 0..3 {
                move_rc = move_rc.multiply(RawCube::move_cube(m));
                table[i * 18 + m * 3 + r] = move_rc.get_flip();
            }
        }
    }
    // SAFETY: table は vec![0u16; 2048 * 18] から作られた Box<[u16]> で、
    // [[u16; 18]; 2048] と完全にメモリレイアウトが一致するため、transmute は安全
    let ptr = Box::into_raw(table.into_boxed_slice()) as *mut [[u16; 18]; 2048];
    unsafe { Box::from_raw(ptr) }
}

fn generate_ud_slice_move_table() -> Box<[[u16; 18]; 495]> {
    let mut table = vec![0u16; 495 * 18];
    let mut rc = RawCube::default();
    for i in 0..495 {
        rc.set_ud_slice(i as u16);
        for m in 0..6 {
            let mut move_rc = rc;
            for r in 0..3 {
                move_rc = move_rc.multiply(RawCube::move_cube(m));
                table[i * 18 + m * 3 + r] = move_rc.get_ud_slice();
            }
        }
    }
    // SAFETY: table は vec![0u16; 495 * 18] から作られた Box<[u16]> で、
    // [[u16; 18]; 495] と完全にメモリレイアウトが一致するため、transmute は安全
    let ptr = Box::into_raw(table.into_boxed_slice()) as *mut [[u16; 18]; 495];
    unsafe { Box::from_raw(ptr) }
}

fn generate_cp_move_table() -> Box<[[u16; 18]; 40320]> {
    let mut table = vec![0u16; 40320 * 18];
    let mut rc = RawCube::default();
    for i in 0..40320 {
        rc.set_cp(i as u16);
        for m in 0..6 {
            let mut move_rc = rc;
            for r in 0..3 {
                move_rc = move_rc.multiply(RawCube::move_cube(m));
                table[i * 18 + m * 3 + r] = move_rc.get_cp();
            }
        }
    }
    // SAFETY: table は vec![0u16; 40320 * 18] から作られた Box<[u16]> で、
    // [[u16; 18]; 40320] と完全にメモリレイアウトが一致するため、transmute は安全
    let ptr = Box::into_raw(table.into_boxed_slice()) as *mut [[u16; 18]; 40320];
    unsafe { Box::from_raw(ptr) }
}

fn generate_ep8_move_table() -> Box<[[u16; 18]; 40320]> {
    let mut table = vec![0u16; 40320 * 18];
    let mut rc = RawCube::default();
    for i in 0..40320 {
        rc.set_ep8(i as u16);
        for m in 0..6 {
            let mut move_rc = rc;
            for r in 0..3 {
                move_rc = move_rc.multiply(RawCube::move_cube(m));
                table[i * 18 + m * 3 + r] = move_rc.get_ep8();
            }
        }
    }
    // SAFETY: table は vec![0u16; 40320 * 18] から作られた Box<[u16]> で、
    // [[u16; 18]; 40320] と完全にメモリレイアウトが一致するため、transmute は安全
    let ptr = Box::into_raw(table.into_boxed_slice()) as *mut [[u16; 18]; 40320];
    unsafe { Box::from_raw(ptr) }
}

fn generate_slice_p_move_table() -> Box<[[u16; 18]; 24]> {
    let mut table = vec![0u16; 24 * 18];
    let mut rc = RawCube::default();
    for i in 0..24 {
        rc.set_slice_p(i as u16);
        for m in 0..6 {
            let mut move_rc = rc;
            for r in 0..3 {
                move_rc = move_rc.multiply(RawCube::move_cube(m));
                table[i * 18 + m * 3 + r] = move_rc.get_slice_p();
            }
        }
    }
    // SAFETY: table は vec![0u16; 24 * 18] から作られた Box<[u16]> で、
    // [[u16; 18]; 24] と完全にメモリレイアウトが一致するため、transmute は安全
    let ptr = Box::into_raw(table.into_boxed_slice()) as *mut [[u16; 18]; 24];
    unsafe { Box::from_raw(ptr) }
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

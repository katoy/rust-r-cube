use super::coord::{move_cube_18, RawCube};
use std::sync::OnceLock;

pub struct MoveTable {
    pub cp: Box<[[u16; 18]; 40320]>,
    pub twist: Box<[[u16; 18]; 2187]>,
}

pub struct PruningTable {
    pub cp: Box<[u8]>,
    pub twist: Box<[u8]>,
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
            }
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

fn generate_cp_pruning_table(mt: &MoveTable) -> Box<[u8]> {
    let mut table = vec![255u8; 40320];
    table[0] = 0;
    let mut distance = 0;
    let mut count = 1;
    while count < 40320 {
        let mut found = false;
        for i in 0..40320 {
            if table[i] == distance {
                for m in 0..18 {
                    let next = mt.cp[i][m] as usize;
                    if table[next] == 255 {
                        table[next] = distance + 1;
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

fn generate_twist_pruning_table(mt: &MoveTable) -> Box<[u8]> {
    let mut table = vec![255u8; 2187];
    table[0] = 0;
    let mut distance = 0;
    let mut count = 1;
    while count < 2187 {
        let mut found = false;
        for i in 0..2187 {
            if table[i] == distance {
                for m in 0..18 {
                    let next = mt.twist[i][m] as usize;
                    if table[next] == 255 {
                        table[next] = distance + 1;
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
    fn test_tables_coverage() {
        let _ = MoveTable::get();
        let _ = PruningTable::get();
    }
}

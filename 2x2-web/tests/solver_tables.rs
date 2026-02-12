use rubiks_cube_2x2::solver::tables::{MoveTable, PruningTable};

#[test]
fn test_tables_extra_coverage() {
    // Transferred from src/solver/tables.rs
    let mt = MoveTable::get();
    assert!(mt.cp[0][0] < 40320);
    let pt = PruningTable::get();
    assert!(pt.get_cp_dist(0) == 0);
}

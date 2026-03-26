use rubiks_cube_2x2::solver::tables::{MoveTable, PruningTable};

#[test]
fn test_tables_extra_coverage() {
    let mt = MoveTable::get();
    assert!(mt.cp[0][0] < 40320);
    let pt = PruningTable::get();
    // get_cp_dist と get_twist_dist の両方を呼び出してカバレッジを確保
    assert_eq!(pt.get_cp_dist(0), 0);
    assert_eq!(pt.get_twist_dist(0), 0);
}

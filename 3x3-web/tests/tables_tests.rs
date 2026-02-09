use rubiks_cube_3x3::kociemba::PruningTable;

#[test]
fn test_pruning_tables() {
    let pruning = PruningTable::get();
    // Phase 1 ソルブ状態: twist=0, ud_slice=0 -> 距離 0
    let idx1 = 0;
    assert_eq!(pruning.twist_slice[idx1], 0);
    assert_eq!(pruning.flip_slice[idx1], 0);

    // Phase 2 ソルブ状態: cp=0, ep8=0, slice_p=0 -> 距離 0
    let idx2 = 0;
    assert_eq!(pruning.cp_slice[idx2], 0);
    assert_eq!(pruning.ep8_slice[idx2], 0);

    // 初期状態でなければ距離 > 0 (インデックス 1 で確認)
    assert!(pruning.twist_slice[1] > 0);
}

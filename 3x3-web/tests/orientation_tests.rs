use rubiks_cube_3x3::cube::{Cube, Face, Move};
use rubiks_cube_3x3::solver;

#[test]
fn test_strict_physical_consistency_all_moves() {
    let base_moves = Move::all_moves();

    for mv in base_moves {
        // M, E, S, X, Y, Z は1手（18種類の基本操作）では解決できないためスキップ
        // または、中心が動く操作は現在のソルバーのロジック（中心相対）では既製品として扱われる
        let mv_str = format!("{:?}", mv);
        if mv_str.contains('M')
            || mv_str.contains('E')
            || mv_str.contains('S')
            || mv_str.contains('X')
            || mv_str.contains('Y')
            || mv_str.contains('Z')
        {
            continue;
        }

        // 1. 理想的な方位から開始
        let mut cube = Cube::new().with_clockwise_orientations();

        // 2. 操作を実行
        cube.apply_move(mv);

        // 3. この状態からソルバー（向き考慮）で解決を試みる
        let solution = solver::solve(&cube, 1, false);

        assert!(
            solution.found,
            "Move {:?} created a state that is not solvable in 1 move to any of the 24 ideal solved orientations.",
            mv
        );

        // 4. 解決後の状態を実際に作り、全ての面が [1, 2, 0, 3] であることを確認
        let mut resolved = cube.clone();
        for &m in &solution.moves {
            resolved.apply_move(m);
        }

        for face in Face::all() {
            let start = face.start_index();
            let pattern: Vec<u8> = (0..9)
                .map(|i| resolved.get_sticker(start + i).orientation)
                .collect();
            let expected = vec![0u8; 9];
            assert_eq!(
                pattern, expected,
                "Face {:?} orientation pattern is broken after solving move {:?}",
                face, mv
            );
        }
    }
}

#[test]
fn test_move_identity_4_times() {
    // 任意の操作を4回繰り返すと、向きも含めて完全に元に戻ることを確認
    for mv in Move::all_moves() {
        let mut cube = Cube::new();
        for _ in 0..4 {
            cube.apply_move(mv);
        }
        assert!(cube.is_solved_with_orientation());
    }
}

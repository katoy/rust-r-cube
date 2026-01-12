use rubiks_cube_2x2::cube::{Cube, Move, Face};
use rubiks_cube_2x2::solver;

#[test]
fn test_strict_physical_consistency_all_moves() {
    let base_moves = Move::all_moves();
    
    for mv in base_moves {
        println!("Verifying move: {:?}", mv);
        
        // 1. 理想的な方位 (全面 [1, 2, 0, 3]) から開始
        let mut cube = Cube::new().with_clockwise_orientations();
        
        // 2. 操作を実行
        cube.apply_move(mv);
        
        // 3. この状態からソルバー（向き考慮）で解決を試みる
        // もし回転ロジックが「理想的な解決状態」の定義と物理的に矛盾していれば、
        // 逆操作をしても「理想的な解決状態」のいずれにもヒットしない。
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
            let pattern: Vec<u8> = (0..4).map(|i| resolved.get_sticker(start + i).orientation).collect();
            assert_eq!(
                pattern, 
                vec![1, 2, 0, 3], 
                "Face {:?} orientation pattern is broken after solving move {:?}", 
                face, mv
            );
        }
    }
}

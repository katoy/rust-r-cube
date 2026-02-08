use rubiks_cube_3x3::cube::{Cube, Face, Move};
use rubiks_cube_3x3::solver::{is_fully_solved, solve};

#[test]
fn test_repro_180_single() {
    let mut cube = Cube::new();
    cube.stickers[Face::Front.start_index() + 4].orientation = 2;
    cube.force_sync_orientation_to_pieces();
    let sol = solve(&cube, 64, false);
    assert!(sol.found);
    let mut final_cube = cube.clone();
    for &m in &sol.moves {
        final_cube.apply_move(m);
    }
    assert!(is_fully_solved(&final_cube));
}

#[test]
fn test_repro_90_pair() {
    let mut cube = Cube::new();
    cube.stickers[Face::Up.start_index() + 4].orientation = 1;
    cube.stickers[Face::Front.start_index() + 4].orientation = 3;
    cube.force_sync_orientation_to_pieces();
    let sol = solve(&cube, 64, false);
    assert!(sol.found);
    let mut final_cube = cube.clone();
    for &m in &sol.moves {
        final_cube.apply_move(m);
    }
    assert!(is_fully_solved(&final_cube));
}

#[test]
fn test_repro_image_873_complex() {
    let mut cube = Cube::new();
    cube.stickers[Face::Up.start_index() + 4].orientation = 1;
    cube.stickers[Face::Down.start_index() + 4].orientation = 1;
    cube.stickers[Face::Front.start_index() + 4].orientation = 3;
    cube.stickers[Face::Back.start_index() + 4].orientation = 3;
    cube.stickers[Face::Left.start_index() + 4].orientation = 0;
    cube.stickers[Face::Right.start_index() + 4].orientation = 2;

    // エッジ不整合も追加
    cube.stickers[Face::Up.start_index() + 1].orientation = 2;

    cube.force_sync_orientation_to_pieces();

    let sol = solve(&cube, 64, false);
    assert!(sol.found, "Moves: {:?}", sol.moves);
    let mut final_cube = cube.clone();
    for &m in &sol.moves {
        final_cube.apply_move(m);
    }
    assert!(is_fully_solved(&final_cube));
}

#[test]
fn test_repro_impossible_parity() {
    let mut cube = Cube::new();
    cube.stickers[Face::Up.start_index() + 4].orientation = 1;
    cube.force_sync_orientation_to_pieces();
    let sol = solve(&cube, 64, false);
    assert!(
        !sol.found,
        "Single 90-degree center should be impossible. Moves found: {:?}",
        sol.moves
    );
}

#[test]
fn test_slice_idempotency() {
    let moves = [Move::M, Move::E, Move::S, Move::X, Move::Y, Move::Z];
    for &mv in &moves {
        let mut cube = Cube::new();
        for _ in 0..4 {
            cube.apply_move(mv);
        }
        for face in Face::all() {
            let start = face.start_index();
            for i in 0..9 {
                assert_eq!(cube.stickers[start + i].orientation, 0);
            }
        }
    }
}

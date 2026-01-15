use rubiks_cube_2x2::cube::{Cube, Move};
use rubiks_cube_2x2::solver::solve_with_progress;

#[test]
fn test_specific_cube_file_operations() {
    let input_content = r#"     WWWW
GGGG RRRR BBBB OOOO
     YYYY
"#;
    let cube = Cube::from_file_format(input_content).expect("ファイルフォーマットエラー");
    let output_content = cube.to_file_format();

    let input_normalized: String = input_content
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let output_normalized: String = output_content
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    assert_eq!(input_normalized, output_normalized);
}

#[test]
fn test_valid_cube_complete_workflow() {
    let mut cube = Cube::new();
    let scramble = vec![Move::R, Move::U, Move::Fp, Move::D, Move::L];
    for move_op in &scramble {
        cube.apply_move(*move_op);
    }

    let saved_content = cube.to_file_format();
    let cube_from_file =
        Cube::from_file_format(&saved_content).expect("ファイルフォーマットエラー");

    let (tx, _rx) = std::sync::mpsc::channel();
    let mut cube_clone = cube_from_file.clone();
    let solution = solve_with_progress(&cube_from_file, 14, true, Some(tx));

    assert!(solution.found);
    for move_op in &solution.moves {
        cube_clone.apply_move(*move_op);
    }

    for face_start in [0, 4, 8, 12, 16, 20] {
        let first_color = cube_clone.get_sticker(face_start).color;
        for offset in 1..4 {
            assert_eq!(
                cube_clone.get_sticker(face_start + offset).color,
                first_color
            );
        }
    }
}

#[test]
fn test_file_roundtrip_preserves_colors() {
    let input1 = r#"     WWWW
GGGG RRRR BBBB OOOO
     YYYY
"#;
    let cube1 = Cube::from_file_format(input1).expect("読み込みエラー");
    let saved = cube1.to_file_format();
    let cube2 = Cube::from_file_format(&saved).expect("再読み込みエラー");

    for i in 0..24 {
        assert_eq!(cube1.get_sticker(i).color, cube2.get_sticker(i).color);
    }
}

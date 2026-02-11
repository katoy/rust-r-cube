use rubiks_cube_3x3::cube::{Color, Cube, Face, Move};
use std::collections::HashSet;

#[test]
fn test_face_integrity_opposing_colors() {
    // どのコーナーやエッジも、対向する面の色を同時に持つことはできない
    let cube = Cube::new();
    let mut scrambled = cube.clone();
    scrambled.scramble(30);

    assert!(check_corners_integrity(&scrambled).is_ok());
    assert!(check_edges_integrity(&scrambled).is_ok());
}

#[test]
fn test_random_scramble_physical_consistency() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let all_moves = Move::all_moves();

    for _ in 0..50 {
        let mut cube = Cube::new();
        for _ in 0..20 {
            let mv = all_moves[rng.gen_range(0..all_moves.len())];
            cube.apply_move(mv);
        }
        assert!(check_corners_integrity(&cube).is_ok());
        assert!(check_edges_integrity(&cube).is_ok());
    }
}

// === Helpers (Moved from cube_tests.rs or other old files) ===

fn is_opposite(c1: Color, c2: Color) -> bool {
    matches!(
        (c1, c2),
        (Color::White, Color::Yellow)
            | (Color::Yellow, Color::White)
            | (Color::Red, Color::Orange)
            | (Color::Orange, Color::Red)
            | (Color::Blue, Color::Green)
            | (Color::Green, Color::Blue)
    )
}

fn check_corners_integrity(cube: &Cube) -> Result<(), String> {
    let corners = vec![
        ("UFL", vec![6, 36, 20]),
        ("UFR", vec![8, 27, 38]),
        ("UBR", vec![2, 45, 29]),
        ("UBL", vec![0, 18, 47]),
        ("DFL", vec![9, 26, 42]),
        ("DFR", vec![11, 44, 33]),
        ("DBR", vec![17, 35, 51]),
        ("DBL", vec![15, 53, 24]),
    ];
    for (name, indices) in corners {
        let colors: Vec<Color> = indices.iter().map(|&i| cube.get_sticker(i).color).collect();
        let unique: HashSet<Color> = colors.iter().cloned().collect();
        if unique.len() != 3 {
            return Err(format!("{}: Not 3 unique colors {:?}", name, colors));
        }
        for i in 0..3 {
            for j in i + 1..3 {
                if is_opposite(colors[i], colors[j]) {
                    return Err(format!("{}: Opposite colors adjacent {:?}", name, colors));
                }
            }
        }
    }
    Ok(())
}

fn check_edges_integrity(cube: &Cube) -> Result<(), String> {
    let edges = vec![
        ("UR", vec![5, 28]),
        ("UF", vec![7, 37]),
        ("UL", vec![3, 19]),
        ("UB", vec![1, 46]),
        ("DR", vec![14, 34]),
        ("DF", vec![10, 43]),
        ("DL", vec![12, 25]),
        ("DB", vec![16, 52]),
        ("FR", vec![41, 30]),
        ("FL", vec![39, 23]),
        ("BL", vec![50, 21]),
        ("BR", vec![48, 32]),
    ];
    for (name, indices) in edges {
        let colors: Vec<Color> = indices.iter().map(|&i| cube.get_sticker(i).color).collect();
        if colors[0] == colors[1] {
            return Err(format!("{}: Same color on edge {:?}", name, colors));
        }
        if is_opposite(colors[0], colors[1]) {
            return Err(format!("{}: Opposite colors on edge {:?}", name, colors));
        }
    }
    Ok(())
}

#[test]
fn test_centers_fixed_relative_to_each_other() {
    let mut cube = Cube::new();
    cube.scramble(20);

    // センターピースの相対的な位置（向かい合う色）は不変
    let up = cube.get_sticker(Face::Up.start_index() + 4).color;
    let down = cube.get_sticker(Face::Down.start_index() + 4).color;
    assert!(
        is_opposite(up, down),
        "Up and Down centers must be opposite"
    );

    let front = cube.get_sticker(Face::Front.start_index() + 4).color;
    let back = cube.get_sticker(Face::Back.start_index() + 4).color;
    assert!(
        is_opposite(front, back),
        "Front and Back centers must be opposite"
    );

    let left = cube.get_sticker(Face::Left.start_index() + 4).color;
    let right = cube.get_sticker(Face::Right.start_index() + 4).color;
    assert!(
        is_opposite(left, right),
        "Left and Right centers must be opposite"
    );
}

use rubiks_cube_3x3::cube::{Cube, Move};

#[test]
fn test_holy_grail_90_90() {
    let mut c = Cube::new();
    // (Mp Up M U) * 9
    for _ in 0..9 {
        let moves = vec![Move::Mp, Move::Up, Move::M, Move::U];
        for &m in &moves {
            c.apply_move(m);
        }
    }

    println!("Holy Grail 90-90 - Solved: {}", c.is_solved());
    println!("U orientation: {}", c.stickers[4].orientation);
    println!("F orientation: {}", c.stickers[40].orientation);

    // Check all stickers orientation
    let non_zero: Vec<usize> = c
        .stickers
        .iter()
        .enumerate()
        .filter(|&(_, s)| s.orientation != 0)
        .map(|(i, _)| i)
        .collect();
    println!("Non-zero orientations: {:?}", non_zero);
}

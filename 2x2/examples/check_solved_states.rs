use rubiks_cube_2x2::cube::Cube;
use rubiks_cube_2x2::solver;

fn main() {
    println!("Checking all 24 solved states...\n");

    let solved_states = solver::get_solved_states();
    println!("Total solved states: {}\n", solved_states.len());

    let clockwise_pattern = [1, 2, 0, 3];

    for (idx, cube) in solved_states.iter().enumerate() {
        println!("Solved state #{}:", idx);

        // 各面のorientationパターンをチェック
        let faces = [
            ("Up   ", 0..4),
            ("Down ", 4..8),
            ("Left ", 8..12),
            ("Right", 12..16),
            ("Front", 16..20),
            ("Back ", 20..24),
        ];

        let mut all_clockwise = true;

        for (face_name, range) in &faces {
            let orientations: Vec<u8> = range
                .clone()
                .map(|i| cube.get_sticker(i).orientation)
                .collect();

            let is_clockwise = orientations == clockwise_pattern;

            print!(
                "  {} [{}, {}, {}, {}]",
                face_name, orientations[0], orientations[1], orientations[2], orientations[3]
            );

            if !is_clockwise {
                print!(" ❌ NOT clockwise!");
                all_clockwise = false;
            }
            println!();
        }

        if all_clockwise {
            println!("  ✅ All faces are clockwise");
        } else {
            println!("  ❌ Some faces are NOT clockwise");
        }
        println!();
    }
}

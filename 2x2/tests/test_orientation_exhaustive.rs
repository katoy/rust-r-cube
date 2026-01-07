use rubiks_cube_2x2::cube::{Color, Cube, Move};

#[test]
fn test_all_moves_exhaustive_24_stickers() {
    let moves = Move::all_moves();

    for &mv in &moves {
        let mut cube = Cube::new();
        cube.apply_move(mv);
        let msg = format!("操作: {:?}", mv);

        let mut cube_back = cube.clone();
        cube_back.apply_move(mv.inverse());
        assert_eq!(
            cube_back,
            Cube::new(),
            "{} -> inverse で元に戻りません",
            msg
        );

        let mut cube_cycle = cube.clone();
        for _ in 0..3 {
            cube_cycle.apply_move(mv);
        }
        assert_eq!(
            cube_cycle,
            Cube::new(),
            "{} を4回繰り返しても元に戻りません",
            msg
        );

        match mv {
            Move::U => {
                for i in 0..4 {
                    check_sticker(&cube, i, Color::White, 1, &msg);
                }
                check_sticker(&cube, 8, Color::Red, 0, &msg);
                check_sticker(&cube, 9, Color::Red, 0, &msg);
                check_sticker(&cube, 12, Color::Orange, 0, &msg);
                check_sticker(&cube, 13, Color::Orange, 0, &msg);
                check_sticker(&cube, 16, Color::Blue, 0, &msg);
                check_sticker(&cube, 17, Color::Blue, 0, &msg);
                check_sticker(&cube, 20, Color::Green, 0, &msg);
                check_sticker(&cube, 21, Color::Green, 0, &msg);
            }
            Move::R => {
                for i in 12..16 {
                    check_sticker(&cube, i, Color::Blue, 3, &msg);
                }
                check_sticker(&cube, 22, Color::White, 2, &msg);
                check_sticker(&cube, 20, Color::White, 2, &msg);
                check_sticker(&cube, 17, Color::Yellow, 0, &msg);
                check_sticker(&cube, 19, Color::Yellow, 0, &msg);
                check_sticker(&cube, 5, Color::Orange, 2, &msg);
                check_sticker(&cube, 7, Color::Orange, 2, &msg);
                check_sticker(&cube, 1, Color::Red, 0, &msg);
                check_sticker(&cube, 3, Color::Red, 0, &msg);
            }
            Move::L => {
                for i in 8..12 {
                    check_sticker(&cube, i, Color::Green, 3, &msg);
                }
                check_sticker(&cube, 16, Color::White, 0, &msg);
                check_sticker(&cube, 18, Color::White, 0, &msg);
                check_sticker(&cube, 23, Color::Yellow, 2, &msg);
                check_sticker(&cube, 21, Color::Yellow, 2, &msg);
                check_sticker(&cube, 4, Color::Red, 0, &msg);
                check_sticker(&cube, 6, Color::Red, 0, &msg);
                check_sticker(&cube, 0, Color::Orange, 2, &msg);
                check_sticker(&cube, 2, Color::Orange, 2, &msg);
            }
            Move::F => {
                for i in 16..20 {
                    check_sticker(&cube, i, Color::Red, 1, &msg);
                }
                check_sticker(&cube, 12, Color::White, 3, &msg);
                check_sticker(&cube, 14, Color::White, 3, &msg);
                check_sticker(&cube, 5, Color::Blue, 1, &msg);
                check_sticker(&cube, 4, Color::Blue, 1, &msg);
                check_sticker(&cube, 11, Color::Yellow, 3, &msg);
                check_sticker(&cube, 9, Color::Yellow, 3, &msg);
                check_sticker(&cube, 2, Color::Green, 1, &msg);
                check_sticker(&cube, 3, Color::Green, 1, &msg);
            }
            Move::B => {
                for i in 20..24 {
                    check_sticker(&cube, i, Color::Orange, 1, &msg);
                }
                check_sticker(&cube, 10, Color::White, 1, &msg);
                check_sticker(&cube, 8, Color::White, 1, &msg);
                check_sticker(&cube, 7, Color::Green, 3, &msg);
                check_sticker(&cube, 6, Color::Green, 3, &msg);
                check_sticker(&cube, 13, Color::Yellow, 1, &msg);
                check_sticker(&cube, 15, Color::Yellow, 1, &msg);
                check_sticker(&cube, 0, Color::Blue, 3, &msg);
                check_sticker(&cube, 1, Color::Blue, 3, &msg);
            }
            _ => {}
        }
    }
}

fn check_sticker(
    cube: &Cube,
    idx: usize,
    expected_color: Color,
    expected_orientation: u8,
    msg: &str,
) {
    let s = cube.get_sticker(idx);
    assert_eq!(
        s.color, expected_color,
        "Index {} color mismatch for {}. Expected {:?}, got {:?}",
        idx, msg, expected_color, s.color
    );
    assert_eq!(
        s.orientation, expected_orientation,
        "Index {} orientation mismatch for {}. Expected {}, got {}",
        idx, msg, expected_orientation, s.orientation
    );
}

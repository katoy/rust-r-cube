use rubiks_cube_3x3::cube::{Cube, Move, NUM_STICKERS};
use rubiks_cube_3x3::gui::mapping::{get_oris_delta, get_source_index};

#[test]
fn test_all_moves_comprehensive_continuity() {
    let all_moves = Move::all_moves();

    for &mv in &all_moves {
        let move_idx = all_moves.iter().position(|&m| m == mv).unwrap();
        if move_idx >= 27 {
            continue;
        }
        test_single_move_continuity(mv);
    }
}

fn test_single_move_continuity(mv: Move) {
    let initial_cube = Cube::new();
    let mut after_cube = initial_cube.clone();
    after_cube.apply_move(mv);

    println!("Testing Move: {}", mv);

    for dst_idx in 0..NUM_STICKERS {
        let src_idx = get_source_index(mv, dst_idx);

        // 1. 色の整合性チェック
        // apply_move (Pieceベース) の結果が、2D用マッピング (MOVE_MAPPING_TABLE) と一致しているか
        let initial_sticker = initial_cube.get_sticker(src_idx);
        let after_sticker = after_cube.get_sticker(dst_idx);

        assert_eq!(
            after_sticker.color, initial_sticker.color,
            "Color mismatch for move {}: sticker at index {} came from {}, but colors don't match ({:?} vs {:?})",
            mv, dst_idx, src_idx, after_sticker.color, initial_sticker.color
        );

        // 2. 向きの連続性チェック
        if is_moving(mv, src_idx) {
            let oris_delta = get_oris_delta(mv, src_idx);
            let start_ori =
                (after_sticker.orientation as i32 + 4 - (oris_delta % 4) as i32) as u8 % 4;

            assert_eq!(
                start_ori, initial_sticker.orientation,
                "Flicker detected for move {}: sticker at index {} (from {}) starts animation at ori {} but source ori was {}. (oris_delta={})",
                mv, dst_idx, src_idx, start_ori, initial_sticker.orientation, oris_delta
            );

            // 3. 面上ステッカーの自転チェック (Orbit中の向き変化の抑制)
            // renderer.rs で is_arc と判定されるステッカーが、面上での自転をしないことを確認
            let is_face_sticker = match mv {
                Move::U | Move::Up | Move::U2 => (0..=8).contains(&dst_idx),
                Move::D | Move::Dp | Move::D2 => (9..=17).contains(&dst_idx),
                Move::L | Move::Lp | Move::L2 => (18..=26).contains(&dst_idx),
                Move::R | Move::Rp | Move::R2 => (27..=35).contains(&dst_idx),
                Move::F | Move::Fp | Move::F2 => (36..=44).contains(&dst_idx),
                Move::B | Move::Bp | Move::B2 => (45..=53).contains(&dst_idx),
                _ => false,
            };

            if is_face_sticker {
                use rubiks_cube_3x3::gui::mapping::FACE_ROTATION_TABLE;
                let all_moves = Move::all_moves();
                let move_idx = all_moves.iter().position(|&m| m == mv).unwrap();
                let move_angle = FACE_ROTATION_TABLE[move_idx].1;
                let target_sub_rot = (oris_delta as f32 * 90.0 - move_angle).rem_euclid(360.0);
                let target_sub_rot = if target_sub_rot > 180.0 {
                    target_sub_rot - 360.0
                } else {
                    target_sub_rot
                };

                assert!(
                    target_sub_rot.abs() < 1.0,
                    "Face sticker rotation detected! Move: {}, index: {}, target_sub_rot: {} (oris_delta: {}, move_angle: {})",
                    mv, dst_idx, target_sub_rot, oris_delta, move_angle
                );
            }
        } else {
            // 移動していない場合は、向きが変わっていないことを確認
            assert_eq!(
                after_sticker.orientation, initial_sticker.orientation,
                "Orientation changed for non-moving sticker! Move: {}, index: {}",
                mv, dst_idx
            );
        }
    }
}

fn is_moving(mv: Move, idx: usize) -> bool {
    use rubiks_cube_3x3::gui::mapping::{FACE_ROTATION_TABLE, MOVE_MAPPING_TABLE};
    let all_moves = Move::all_moves();
    let move_idx = all_moves.iter().position(|&m| m == mv).unwrap();

    let (face_start, _) = FACE_ROTATION_TABLE[move_idx];
    let (src, dst) = MOVE_MAPPING_TABLE[move_idx][idx];

    let is_on_face = if face_start != usize::MAX {
        dst >= face_start && dst < face_start + 9
    } else {
        false // Slice moves don't have a single face rotation in 2D
    };

    src != dst || is_on_face
}

use super::{Cube, Move};
use glam::Vec3;

/// 指定された回転操作をキューブに適用します。
pub fn apply_move(cube: &mut Cube, mv: Move) {
    let (axis, layer_val, angle) = move_to_geometric_params(mv);

    // 全体回転 (X, Y, Z) の場合は全ピースを対象にする
    let is_global = matches!(
        mv,
        Move::X
            | Move::Xp
            | Move::X2
            | Move::Y
            | Move::Yp
            | Move::Y2
            | Move::Z
            | Move::Zp
            | Move::Z2
    );

    for piece in &mut cube.pieces {
        if is_global || is_in_layer(piece.current_pos, axis, layer_val) {
            piece.rotate(axis, angle);
        }
    }

    // ピースの状態を Facelet（ステッカー）配列に反映
    cube.sync_stickers();
}

/// 指定回数のランダムな回転操作を適用します。
pub fn scramble(cube: &mut Cube, moves: usize) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let all_moves: Vec<Move> = Move::all_moves()
        .into_iter()
        .filter(|m| !m.is_global() && !m.is_middle_layer())
        .collect();

    for _ in 0..moves {
        let mv = all_moves[rng.gen_range(0..all_moves.len())];
        apply_move(cube, mv);
    }
}

/// 操作を幾何学的なパラメータ（回転軸、対象層、角度）に変換します。
fn move_to_geometric_params(mv: Move) -> (Vec3, i8, f32) {
    let pi_2 = std::f32::consts::FRAC_PI_2;
    match mv {
        // R層 (x = 1)
        Move::R => (Vec3::X, 1, -pi_2),
        Move::Rp => (Vec3::X, 1, pi_2),
        Move::R2 => (Vec3::X, 1, pi_2 * 2.0),
        // L層 (x = -1)
        Move::L => (Vec3::X, -1, pi_2),
        Move::Lp => (Vec3::X, -1, -pi_2),
        Move::L2 => (Vec3::X, -1, pi_2 * 2.0),
        // M層 (x = 0) - Lと同じ回転方向
        Move::M => (Vec3::X, 0, pi_2),
        Move::Mp => (Vec3::X, 0, -pi_2),
        Move::M2 => (Vec3::X, 0, pi_2 * 2.0),

        // U層 (y = 1)
        Move::U => (Vec3::Y, 1, -pi_2),
        Move::Up => (Vec3::Y, 1, pi_2),
        Move::U2 => (Vec3::Y, 1, pi_2 * 2.0),
        // D層 (y = -1)
        Move::D => (Vec3::Y, -1, pi_2),
        Move::Dp => (Vec3::Y, -1, -pi_2),
        Move::D2 => (Vec3::Y, -1, pi_2 * 2.0),
        // E層 (y = 0) - Dと同じ回転方向
        Move::E => (Vec3::Y, 0, pi_2),
        Move::Ep => (Vec3::Y, 0, -pi_2),
        Move::E2 => (Vec3::Y, 0, pi_2 * 2.0),

        // F層 (z = 1)
        Move::F => (Vec3::Z, 1, -pi_2),
        Move::Fp => (Vec3::Z, 1, pi_2),
        Move::F2 => (Vec3::Z, 1, pi_2 * 2.0),
        // B層 (z = -1)
        Move::B => (Vec3::Z, -1, pi_2),
        Move::Bp => (Vec3::Z, -1, -pi_2),
        Move::B2 => (Vec3::Z, -1, pi_2 * 2.0),
        // S層 (z = 0) - Fと同じ回転方向
        Move::S => (Vec3::Z, 0, -pi_2),
        Move::Sp => (Vec3::Z, 0, pi_2),
        Move::S2 => (Vec3::Z, 0, pi_2 * 2.0),

        // 全体 X
        Move::X => (Vec3::X, 2, -pi_2),
        Move::Xp => (Vec3::X, 2, pi_2),
        Move::X2 => (Vec3::X, 2, pi_2 * 2.0),
        // 全体 Y
        Move::Y => (Vec3::Y, 2, -pi_2),
        Move::Yp => (Vec3::Y, 2, pi_2),
        Move::Y2 => (Vec3::Y, 2, pi_2 * 2.0),
        // 全体 Z
        Move::Z => (Vec3::Z, 2, -pi_2),
        Move::Zp => (Vec3::Z, 2, pi_2),
        Move::Z2 => (Vec3::Z, 2, pi_2 * 2.0),
    }
}

/// ピースがある回転軸上の指定された層に含まれるか判定します。
fn is_in_layer(pos: Vec3, axis: Vec3, layer_val: i8) -> bool {
    if axis == Vec3::X {
        (pos.x.round() as i8) == layer_val
    } else if axis == Vec3::Y {
        (pos.y.round() as i8) == layer_val
    } else if axis == Vec3::Z {
        (pos.z.round() as i8) == layer_val
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_in_layer_invalid_axis() {
        assert!(!is_in_layer(Vec3::ZERO, Vec3::ZERO, 0));
    }
}

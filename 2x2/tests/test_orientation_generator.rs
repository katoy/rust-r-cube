use glam::{Mat3, Vec3};
use rubiks_cube_2x2::cube::{Color, Cube, Move};

/// renderer_3d.rs の定義に基づき、ステッカーの幾何学的情報を定義
struct StickerGeom {
    index: usize,
    center: Vec3,
    u_vec: Vec3,
    v_vec: Vec3,
}

fn get_initial_geom() -> Vec<StickerGeom> {
    let mut stickers = Vec::new();
    // U (Up): normal=Y, u=X, v=Z
    for i in 0..4 {
        let x = if i % 2 == 0 { -0.5 } else { 0.5 };
        let z = if i / 2 == 0 { -0.5 } else { 0.5 };
        stickers.push(StickerGeom {
            index: i,
            center: Vec3::new(x, 1.0, z),
            u_vec: Vec3::X,
            v_vec: Vec3::Z,
        });
    }
    // D (Down): normal=-Y, u=X, v=-Z
    for i in 0..4 {
        let x = if i % 2 == 0 { -0.5 } else { 0.5 };
        let z = if i / 2 == 0 { 0.5 } else { -0.5 };
        stickers.push(StickerGeom {
            index: 4 + i,
            center: Vec3::new(x, -1.0, z),
            u_vec: Vec3::X,
            v_vec: -Vec3::Z,
        });
    }
    // L (Left): normal=-X, u=-Z, v=-Y
    for i in 0..4 {
        let z = if i % 2 == 0 { -0.5 } else { 0.5 };
        let y = if i / 2 == 0 { 0.5 } else { -0.5 };
        stickers.push(StickerGeom {
            index: 8 + i,
            center: Vec3::new(-1.0, y, z),
            u_vec: -Vec3::Z,
            v_vec: -Vec3::Y,
        });
    }
    // R (Right): normal=X, u=Z, v=-Y
    for i in 0..4 {
        let z = if i % 2 == 0 { 0.5 } else { -0.5 };
        let y = if i / 2 == 0 { 0.5 } else { -0.5 };
        stickers.push(StickerGeom {
            index: 12 + i,
            center: Vec3::new(1.0, y, z),
            u_vec: Vec3::Z,
            v_vec: -Vec3::Y,
        });
    }
    // F (Front): normal=Z, u=X, v=-Y
    for i in 0..4 {
        let x = if i % 2 == 0 { -0.5 } else { 0.5 };
        let y = if i / 2 == 0 { 0.5 } else { -0.5 };
        stickers.push(StickerGeom {
            index: 16 + i,
            center: Vec3::new(x, y, 1.0),
            u_vec: Vec3::X,
            v_vec: -Vec3::Y,
        });
    }
    // B (Back): normal=-Z, u=-X, v=-Y
    for i in 0..4 {
        let x = if i % 2 == 0 { 0.5 } else { -0.5 };
        let y = if i / 2 == 0 { 0.5 } else { -0.5 };
        stickers.push(StickerGeom {
            index: 20 + i,
            center: Vec3::new(x, y, -1.0),
            u_vec: -Vec3::X,
            v_vec: -Vec3::Y,
        });
    }
    stickers
}

fn get_rot_matrix(mv: Move) -> Mat3 {
    match mv {
        Move::U => Mat3::from_rotation_y(-std::f32::consts::FRAC_PI_2),
        Move::Up => Mat3::from_rotation_y(std::f32::consts::FRAC_PI_2),
        Move::D => Mat3::from_rotation_y(std::f32::consts::FRAC_PI_2),
        Move::Dp => Mat3::from_rotation_y(-std::f32::consts::FRAC_PI_2),
        Move::L => Mat3::from_rotation_x(std::f32::consts::FRAC_PI_2),
        Move::Lp => Mat3::from_rotation_x(-std::f32::consts::FRAC_PI_2),
        Move::R => Mat3::from_rotation_x(-std::f32::consts::FRAC_PI_2),
        Move::Rp => Mat3::from_rotation_x(std::f32::consts::FRAC_PI_2),
        Move::F => Mat3::from_rotation_z(-std::f32::consts::FRAC_PI_2),
        Move::Fp => Mat3::from_rotation_z(std::f32::consts::FRAC_PI_2),
        Move::B => Mat3::from_rotation_z(std::f32::consts::FRAC_PI_2),
        Move::Bp => Mat3::from_rotation_z(-std::f32::consts::FRAC_PI_2),
    }
}

fn is_affected(mv: Move, center: Vec3) -> bool {
    // 2x2の場合は単純に座標の正負で判定（スライスが原点を通るため）
    match mv {
        Move::U | Move::Up => center.y > 0.0,
        Move::D | Move::Dp => center.y < 0.0,
        Move::L | Move::Lp => center.x < 0.0,
        Move::R | Move::Rp => center.x > 0.0,
        Move::F | Move::Fp => center.z > 0.0,
        Move::B | Move::Bp => center.z < 0.0,
    }
}

#[test]
fn generate_orientation_truth() {
    let initial_geom = get_initial_geom();
    let moves = Move::all_moves();

    for mv in moves {
        println!("--- {:?} ---", mv);
        let rot = get_rot_matrix(mv);

        for g in &initial_geom {
            if is_affected(mv, g.center) {
                let new_center = rot * g.center;
                // 遷移先のステッカーを特定
                let mut target_idx = 999;
                for g_target in &initial_geom {
                    if (g_target.center - new_center).length() < 0.1 {
                        target_idx = g_target.index;
                        break;
                    }
                }

                // 向きの変位を計算
                // 初期状態の向き 0 は -v_vec
                let initial_v_arrow = -g.v_vec;
                let rotated_v_arrow = rot * initial_v_arrow;

                // ターゲット面での向き 0,1,2,3 を算出
                let target_g = &initial_geom[target_idx];
                let mut best_orient = 0;
                let mut max_dot = -2.0;
                let candidates = [
                    (0, -target_g.v_vec),
                    (1, target_g.u_vec),
                    (2, target_g.v_vec),
                    (3, -target_g.u_vec),
                ];
                for (o, v) in candidates {
                    let d = rotated_v_arrow.dot(v);
                    if d > max_dot {
                        max_dot = d;
                        best_orient = o;
                    }
                }

                println!(
                    "  idx {} -> idx {}, orient {}",
                    g.index, target_idx, best_orient
                );
            }
        }
    }
}

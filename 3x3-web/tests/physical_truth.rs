use glam::{Mat4, Vec3};
use rubiks_cube_3x3::cube::Move;

/// renderer_3d.rs と同じ面定義
struct FaceDef {
    normal: Vec3,
    u_axis: Vec3,
    v_axis: Vec3,
}

fn get_face_defs() -> [FaceDef; 6] {
    [
        FaceDef {
            normal: Vec3::Y,
            u_axis: Vec3::X,
            v_axis: -Vec3::Z,
        }, // U
        FaceDef {
            normal: -Vec3::Y,
            u_axis: Vec3::X,
            v_axis: Vec3::Z,
        }, // D
        FaceDef {
            normal: -Vec3::X,
            u_axis: Vec3::Z,
            v_axis: Vec3::Y,
        }, // L
        FaceDef {
            normal: Vec3::X,
            u_axis: -Vec3::Z,
            v_axis: Vec3::Y,
        }, // R
        FaceDef {
            normal: Vec3::Z,
            u_axis: Vec3::X,
            v_axis: Vec3::Y,
        }, // F
        FaceDef {
            normal: -Vec3::Z,
            u_axis: -Vec3::X,
            v_axis: Vec3::Y,
        }, // B
    ]
}

/// 3D空間でのステッカーの状態
#[derive(Debug, Clone, Copy)]
struct PhysicalSticker {
    center: Vec3,
    normal: Vec3,
    arrow_dir: Vec3, // orientation 0 の時の向き (= v_axis)
}

fn get_initial_physical_state() -> Vec<PhysicalSticker> {
    let mut stickers = Vec::new();
    let defs = get_face_defs();
    for def in defs.iter() {
        for i in 0..9 {
            let col = (i % 3) as f32;
            let row = (i / 3) as f32;
            let u_val = (col - 1.0) * (2.0 / 3.0);
            let v_val = (row - 1.0) * (2.0 / 3.0); // renderer_3d.rs の flip_v と整合性をとる

            // renderer_3d.rs の初期配置ロジックを簡略化して再現
            let center = def.normal + def.u_axis * u_val + def.v_axis * v_val;

            stickers.push(PhysicalSticker {
                center,
                normal: def.normal,
                arrow_dir: def.v_axis,
            });
        }
    }
    stickers
}

/// 回転後の3D状態から、2Dでの「論理的な向き(0-3)」を判定する
fn calculate_logical_orientation(sticker: &PhysicalSticker) -> u8 {
    let defs = get_face_defs();
    // どの面にいるか判定
    let face_idx: usize = if sticker.normal.y > 0.9 {
        0
    } else if sticker.normal.y < -0.9 {
        1
    } else if sticker.normal.x < -0.9 {
        2
    } else if sticker.normal.x > 0.9 {
        3
    } else if sticker.normal.z > 0.9 {
        4
    } else if sticker.normal.z < -0.9 {
        5
    } else {
        panic!("Invalid normal: {:?}", sticker.normal)
    };

    let def = &defs[face_idx];

    // arrow_dir が def.v_axis (orientation 0) から見て、
    // def.normal の周りにどれだけ回転しているか

    // 各方位のベクトルを計算
    let v0 = def.v_axis; // 0
    let v1 = Mat4::from_axis_angle(def.normal, -std::f32::consts::FRAC_PI_2).transform_vector3(v0); // 1
    let v2 = Mat4::from_axis_angle(def.normal, -std::f32::consts::PI).transform_vector3(v0); // 2
    let v3 =
        Mat4::from_axis_angle(def.normal, -3.0 * std::f32::consts::FRAC_PI_2).transform_vector3(v0); // 3

    let dirs = [v0, v1, v2, v3];
    let mut best_dir = 0;
    let mut max_dot = -2.0;
    for (i, d) in dirs.iter().enumerate() {
        let dot = d.dot(sticker.arrow_dir);
        if dot > max_dot {
            max_dot = dot;
            best_dir = i;
        }
    }
    best_dir as u8
}

fn is_affected(mv: Move, p: Vec3) -> bool {
    match mv {
        Move::R | Move::Rp | Move::R2 => p.x > 0.3,
        Move::L | Move::Lp | Move::L2 => p.x < -0.3,
        Move::M | Move::Mp | Move::M2 => p.x.abs() < 0.3,
        Move::U | Move::Up | Move::U2 => p.y > 0.3,
        Move::D | Move::Dp | Move::D2 => p.y < -0.3,
        Move::E | Move::Ep | Move::E2 => p.y.abs() < 0.3,
        Move::F | Move::Fp | Move::F2 => p.z > 0.3,
        Move::B | Move::Bp | Move::B2 => p.z < -0.3,
        Move::S | Move::Sp | Move::S2 => p.z.abs() < 0.3,
        Move::X | Move::Xp | Move::X2 => true,
        Move::Y | Move::Yp | Move::Y2 => true,
        Move::Z | Move::Zp | Move::Z2 => true,
    }
}

fn get_move_params(mv: Move) -> (Vec3, f32) {
    match mv {
        Move::R => (Vec3::X, -90.0),
        Move::Rp => (Vec3::X, 90.0),
        Move::R2 => (Vec3::X, 180.0),
        Move::L => (Vec3::X, 90.0),
        Move::Lp => (Vec3::X, -90.0),
        Move::L2 => (Vec3::X, 180.0),
        Move::U => (Vec3::Y, -90.0),
        Move::Up => (Vec3::Y, 90.0),
        Move::U2 => (Vec3::Y, 180.0),
        Move::D => (Vec3::Y, 90.0),
        Move::Dp => (Vec3::Y, -90.0),
        Move::D2 => (Vec3::Y, 180.0),
        Move::F => (Vec3::Z, -90.0),
        Move::Fp => (Vec3::Z, 90.0),
        Move::F2 => (Vec3::Z, 180.0),
        Move::B => (Vec3::Z, 90.0),
        Move::Bp => (Vec3::Z, -90.0),
        Move::B2 => (Vec3::Z, 180.0),
        Move::M => (Vec3::X, 90.0),
        Move::Mp => (Vec3::X, -90.0),
        Move::M2 => (Vec3::X, 180.0),
        Move::E => (Vec3::Y, 90.0),
        Move::Ep => (Vec3::Y, -90.0),
        Move::E2 => (Vec3::Y, 180.0),
        Move::S => (Vec3::Z, -90.0),
        Move::Sp => (Vec3::Z, 90.0),
        Move::S2 => (Vec3::Z, 180.0),
        Move::X => (Vec3::X, -90.0),
        Move::Xp => (Vec3::X, 90.0),
        Move::X2 => (Vec3::X, 180.0),
        Move::Y => (Vec3::Y, -90.0),
        Move::Yp => (Vec3::Y, 90.0),
        Move::Y2 => (Vec3::Y, 180.0),
        Move::Z => (Vec3::Z, -90.0),
        Move::Zp => (Vec3::Z, 90.0),
        Move::Z2 => (Vec3::Z, 180.0),
    }
}

fn apply_physical_move(stickers: &mut [PhysicalSticker], mv: Move) {
    let (axis, angle_deg) = get_move_params(mv);
    let mat = Mat4::from_axis_angle(axis, angle_deg.to_radians());

    for s in stickers.iter_mut() {
        if is_affected(mv, s.center) {
            s.center = mat.transform_point3(s.center);
            s.normal = mat.transform_vector3(s.normal);
            s.arrow_dir = mat.transform_vector3(s.arrow_dir);
        }
    }
}

#[test]
fn generate_all_oris_delta() {
    let moves = [
        Move::U,
        Move::D,
        Move::L,
        Move::R,
        Move::F,
        Move::B,
        Move::M,
        Move::E,
        Move::S,
    ];

    for mv in moves {
        let mut stickers = get_initial_physical_state();
        apply_physical_move(&mut stickers, mv);

        let mut _oris: Vec<u8> = Vec::new();
        // サイクルに対応する12個のステッカーを特定する必要がある。
        // 面回転の場合は face stickers 9個も確認。
    }
}

/// 指定された操作後の全ステッカーの期待される orientation を表示する
#[test]
fn print_expected_orientations() {
    let moves = [
        Move::U,
        Move::D,
        Move::L,
        Move::R,
        Move::F,
        Move::B,
        Move::M,
        Move::E,
        Move::S,
    ];

    for mv in moves {
        let mut stickers = get_initial_physical_state();
        apply_physical_move(&mut stickers, mv);

        println!("\n=== Move: {:?} ===", mv);
        for f in 0..6 {
            let face_name = ["U", "D", "L", "R", "F", "B"][f];
            print!("{}: ", face_name);
            for i in 0..9 {
                // 現在のスロット f*9+i にあるステッカーを探す
                let sticker = stickers
                    .iter()
                    .find(|s| {
                        let logical_f = if s.normal.y > 0.9 {
                            0
                        } else if s.normal.y < -0.9 {
                            1
                        } else if s.normal.x < -0.9 {
                            2
                        } else if s.normal.x > 0.9 {
                            3
                        } else if s.normal.z > 0.9 {
                            4
                        } else if s.normal.z < -0.9 {
                            5
                        } else {
                            99
                        };

                        if logical_f != f {
                            return false;
                        }

                        let defs = get_face_defs();
                        let def = &defs[f];
                        let local = s.center - def.normal;
                        let u = local.dot(def.u_axis);
                        let v = local.dot(def.v_axis);

                        let row = (v / (2.0 / 3.0) + 1.0).round() as i32;
                        let col = (u / (2.0 / 3.0) + 1.0).round() as i32;
                        row * 3 + col == i
                    })
                    .unwrap();

                print!("{}, ", calculate_logical_orientation(sticker));
            }
            println!();
        }
    }
}

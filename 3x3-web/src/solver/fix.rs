use super::{get_all_rotations, get_orientations_vec, get_solved_states, undo_setup};
use crate::cube::{Cube, Face, Move};
use crate::kociemba::Search;
use glam::Vec3;

/// 現在のキューブのセンター方位に基づき、正しい完成状態の方位（ターゲット）を特定します。
///
/// キューブが全体的に X, Y, Z 回転されている場合でも、センターの色配置から、
/// 24通りの完成状態のうち現在の色配置に一致するものを探し出し、その向きを目標とします。
pub fn get_target_oris(cube: &Cube) -> Vec<u8> {
    let states = get_solved_states();
    for (_i, s) in states.iter().enumerate() {
        let mut match_centers = true;
        for f in Face::all() {
            let sc = s.stickers[f.start_index() + 4].color;
            let cc = cube.stickers[f.start_index() + 4].color;
            if sc != cc {
                match_centers = false;
                break;
            }
        }
        if match_centers {
            if std::env::var("SOLVER_DEBUG").is_ok() {
                println!("DEBUG: get_target_oris: Matched solved state pattern {} based on center colors.", _i);
                let colors: Vec<_> = Face::all()
                    .iter()
                    .map(|f| s.stickers[f.start_index() + 4].color)
                    .collect();
                println!("DEBUG: get_target_oris: Pattern colors={:?}", colors);
            }
            return get_orientations_vec(s);
        }
    }
    if std::env::var("SOLVER_DEBUG").is_ok() {
        println!("DEBUG: get_target_oris: No match found! Falling back to Pattern 0.");
    }
    vec![0, 0, 0, 0, 0, 0]
}

/// スーパーキューブ（センターに絵や向きがあるキューブ）としての解決手順を生成します。
///
/// 1. 現在のセンターの向きとターゲットの向きのズレを計算します。
/// 2. 180度回転が必要な面がある場合、単独で180度回転させる手順を適用します。
/// 3. 90度回転が必要な面が複数ある場合、ペア（片方を時計回り、もう片方を反時計回り）で修正する手順を適用します。
/// これらの手順は「真に色保存的」であり、センター以外のピース（エッジ、コーナー）の配置を一切崩しません。
pub fn apply_supercube_fixes(cube: &Cube, _search: &mut Search) -> Vec<Move> {
    let mut current_cube = cube.clone();
    let mut final_moves = Vec::new();
    let target_oris = get_target_oris(cube);

    for iter in 0..12 {
        let oris = get_orientations_vec(&current_cube);
        if std::env::var("SOLVER_DEBUG").is_ok() {
            println!(
                "DEBUG: apply_supercube_fixes: iter={}, oris={:?}, target={:?}",
                iter, oris, target_oris
            );
        }
        if oris == target_oris {
            break;
        }

        // 相対的なズレを計算 (0:なし, 1:CW, 2:180, 3:CCW)
        let mut rel_oris = [0u8; 6];
        for i in 0..6 {
            rel_oris[i] = (oris[i] as i8 - target_oris[i] as i8).rem_euclid(4) as u8;
        }

        let mut d180s = Vec::new();
        let mut d90s = Vec::new(); // (Face, rel_ori)
        for (i, &rel_o) in rel_oris.iter().enumerate() {
            let f = Face::from_index(i * 9);
            if rel_o == 2 {
                d180s.push(f);
            } else if rel_o != 0 {
                d90s.push((f, rel_o));
            }
        }

        let fix = if let Some(&f) = d180s.first() {
            // 単独180度修正
            get_fix_180(f)
        } else if d90s.len() >= 2 {
            let (f1, r1) = d90s[0];
            let (f2, r2) = d90s[1];

            if !is_opposite_face(f1, f2) {
                // 隣接する2つの面で90度ペア修正
                if r1 == 1 && r2 == 3 {
                    get_fix_90_pair(f1, f2)
                } else if r1 == 3 && r2 == 1 {
                    get_fix_90_pair(f2, f1)
                } else if r1 == 1 && r2 == 1 {
                    get_fix_90_pair(f1, f2)
                } else {
                    get_fix_90_pair(f2, f1)
                }
            } else {
                // 反対側の面同士の場合、中継面（バッファ）を使用して2ステップで修正
                let buffer = get_buffer_face(f1, f2);
                if r1 == 1 {
                    get_fix_90_pair(f1, buffer)
                } else {
                    get_fix_90_pair(buffer, f1)
                }
            }
        } else {
            if std::env::var("SOLVER_DEBUG").is_ok() {
                println!(
                    "DEBUG: apply_supercube_fixes: breaking at iter {} with d90s.len={}",
                    iter,
                    d90s.len()
                );
            }
            break;
        };

        if std::env::var("SOLVER_DEBUG").is_ok() {
            let oris_before = get_orientations_vec(&current_cube);
            for &m in &fix {
                current_cube.apply_move(m);
            }
            println!(
                "DEBUG: apply_supercube_fixes: applied fix of len {}. Oris: {:?} -> {:?}",
                fix.len(),
                oris_before,
                get_orientations_vec(&current_cube)
            );
        } else {
            for &m in &fix {
                current_cube.apply_move(m);
            }
        }
        final_moves.extend(fix);
    }
    final_moves
}

pub fn is_opposite_face(f1: Face, f2: Face) -> bool {
    matches!(
        (f1, f2),
        (Face::Up, Face::Down)
            | (Face::Down, Face::Up)
            | (Face::Front, Face::Back)
            | (Face::Back, Face::Front)
            | (Face::Right, Face::Left)
            | (Face::Left, Face::Right)
    )
}

pub fn get_buffer_face(f1: Face, f2: Face) -> Face {
    for &f in &[
        Face::Up,
        Face::Down,
        Face::Front,
        Face::Back,
        Face::Right,
        Face::Left,
    ] {
        if !is_opposite_face(f1, f) && !is_opposite_face(f2, f) && f != f1 && f != f2 {
            return f;
        }
    }
    Face::Up
}

fn get_fix_180(face: Face) -> Vec<Move> {
    let rot = get_setup_to_up(face);
    let mut moves = rot.clone();
    let seq = vec![
        Move::U,
        Move::R,
        Move::L,
        Move::U2,
        Move::Rp,
        Move::Lp,
        Move::U,
        Move::R,
        Move::L,
        Move::U2,
        Move::Rp,
        Move::Lp,
    ];
    moves.extend(seq);
    moves.extend(undo_setup(rot));
    moves
}

fn get_fix_90_pair(f_cw: Face, f_ccw: Face) -> Vec<Move> {
    let rot = get_setup_to_up_right(f_cw, f_ccw);
    let mut moves = rot.clone();
    let seq = vec![
        Move::Mp,
        Move::E,
        Move::M,
        Move::U,
        Move::Mp,
        Move::Ep,
        Move::M,
        Move::Up,
    ];
    moves.extend(seq);
    moves.extend(undo_setup(rot));
    moves
}

pub fn get_setup_to_up(face: Face) -> Vec<Move> {
    for rot in get_all_rotations() {
        let result_face = apply_rot_to_face(face, &rot);
        if result_face == Face::Up {
            return rot.to_vec();
        }
    }
    vec![]
}

pub fn get_setup_to_up_right(f_up: Face, f_right: Face) -> Vec<Move> {
    for rot in get_all_rotations() {
        if apply_rot_to_face(f_up, &rot) == Face::Up
            && apply_rot_to_face(f_right, &rot) == Face::Right
        {
            return rot.to_vec();
        }
    }
    vec![]
}

pub fn apply_rot_to_face(face: Face, rot: &[Move]) -> Face {
    let mut normal = match face {
        Face::Up => Vec3::Y,
        Face::Down => -Vec3::Y,
        Face::Left => -Vec3::X,
        Face::Right => Vec3::X,
        Face::Front => Vec3::Z,
        Face::Back => -Vec3::Z,
    };
    for &m in rot {
        let (axis, _, angle) = move_to_geometric_params_for_rot(m);
        let mat = glam::Mat4::from_axis_angle(axis, angle);
        normal = mat.transform_vector3(normal);
    }
    Face::all()
        .iter()
        .copied()
        .find(|&f| {
            let fnorm = match f {
                Face::Up => Vec3::Y,
                Face::Down => -Vec3::Y,
                Face::Left => -Vec3::X,
                Face::Right => Vec3::X,
                Face::Front => Vec3::Z,
                Face::Back => -Vec3::Z,
            };
            (normal - fnorm).length() < 0.1
        })
        .unwrap_or(Face::Up)
}

fn move_to_geometric_params_for_rot(mv: Move) -> (Vec3, i8, f32) {
    let pi_2 = std::f32::consts::FRAC_PI_2;
    match mv {
        Move::X => (Vec3::X, 0, -pi_2),
        Move::Xp => (Vec3::X, 0, pi_2),
        Move::X2 => (Vec3::X, 0, std::f32::consts::PI),
        Move::Y => (Vec3::Y, 0, -pi_2),
        Move::Yp => (Vec3::Y, 0, pi_2),
        Move::Y2 => (Vec3::Y, 0, std::f32::consts::PI),
        Move::Z => (Vec3::Z, 0, -pi_2),
        Move::Zp => (Vec3::Z, 0, pi_2),
        Move::Z2 => (Vec3::Z, 0, std::f32::consts::PI),
        _ => (Vec3::Y, 0, 0.0),
    }
}

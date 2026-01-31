use crate::cube::{piece::face_to_local_index, Cube, Face, Move};
use crate::gui::app::AnimationState;
use crate::gui::constants::*;
use egui::{Color32, Pos2, Rect, Stroke};
use glam::{Mat4, Vec3};

/// 3D描画の設定
pub struct View3D {
    pub yaw: f32,
    pub pitch: f32,
    pub scale: f32,
}

impl Default for View3D {
    fn default() -> Self {
        Self {
            yaw: VIEW3D_DEFAULT_YAW,
            pitch: VIEW3D_DEFAULT_PITCH,
            scale: VIEW3D_DEFAULT_SCALE,
        }
    }
}

/// 描画用の頂点データ
struct DrawFace {
    z_score: f32,
    points: Vec<Pos2>,
    color: Color32,
    sticker_index: usize,
    center_2d: Pos2,
    arrow_end_2d: Pos2,
}

/// 3D投影ヘルパー関数
fn project_point(p: Vec3, view_mat: &Mat4, scale: f32, screen_center: Pos2) -> Pos2 {
    let view = view_mat.transform_point3(p);
    let distance = VIEW3D_CAMERA_DISTANCE - view.z;
    let perspective = VIEW3D_CAMERA_DISTANCE / distance;
    Pos2::new(
        screen_center.x + view.x * scale * perspective,
        screen_center.y - view.y * scale * perspective,
    )
}

/// 3D空間で矢印を描画
fn draw_arrow_3d(painter: &egui::Painter, center: Pos2, target: Pos2, color: Color32, width: f32) {
    let direction = target - center;
    let arrow_length = direction.length();
    if arrow_length < 0.1 {
        return;
    }

    let dir_normalized = direction.normalized();
    let arrow_end = center + dir_normalized * arrow_length;

    painter.line_segment([center, arrow_end], Stroke::new(width, color));

    let arrow_head_size = arrow_length * VIEW3D_ARROW_HEAD_RATIO;
    let perpendicular = egui::vec2(-dir_normalized.y, dir_normalized.x);

    let tip = arrow_end;
    let left = arrow_end - dir_normalized * arrow_head_size + perpendicular * arrow_head_size * 0.5;
    let right =
        arrow_end - dir_normalized * arrow_head_size - perpendicular * arrow_head_size * 0.5;

    painter.add(egui::Shape::convex_polygon(
        vec![tip, left, right],
        color,
        Stroke::NONE,
    ));
}

/// 3D描画関数
pub fn draw_cube_3d(
    ui: &mut egui::Ui,
    rect: Rect,
    cube: &Cube,
    animation: Option<&AnimationState>,
    view: &View3D,
    highlight_face_index: Option<usize>,
) {
    let painter = ui.painter();

    // カメラ設定
    let center = rect.center();
    let min_dim = rect.width().min(rect.height());
    let scale = min_dim * VIEW3D_PROJECTION_SCALE * view.scale;

    // ビュー行列 (Orbit camera)
    let view_mat = Mat4::from_rotation_x(view.pitch) * Mat4::from_rotation_y(view.yaw);

    // アニメーション情報
    let (anim_axis, anim_layer, anim_full_angle) = if let Some(anim) = animation {
        move_to_anim_params(anim.current_move)
    } else {
        (Vec3::X, 0, 0.0)
    };

    let anim_angle = if let Some(anim) = animation {
        anim_full_angle * (anim.eased_progress() - 1.0)
    } else {
        0.0
    };

    let mut draw_faces = Vec::new();
    let size = VIEW3D_STICKER_SIZE;

    for (_p_idx, piece) in cube.pieces.iter().enumerate() {
        // アニメーションによる追加回転
        let mut anim_mat = Mat4::IDENTITY;
        if let Some(_anim) = animation {
            if is_affected(piece.current_pos, anim_axis, anim_layer) {
                anim_mat = Mat4::from_axis_angle(anim_axis, anim_angle);
            }
        }

        let total_piece_rot = anim_mat * piece.current_rot;
        // Pieceの現在地（浮動小数点数）
        let current_pos_3d = anim_mat.transform_point3(piece.current_pos);

        for sticker in &piece.stickers {
            let initial_normal = sticker.initial_normal;
            let current_normal = total_piece_rot.transform_vector3(initial_normal);
            let world_normal = view_mat.transform_vector3(current_normal);

            // バックフェイスカリング
            if world_normal.z > VIEW3D_BACKFACE_CULLING_THRESHOLD {
                // 頂点の計算
                // ステッカーのローカル座標系での u, v ベクトル
                let (u_axis, v_axis) = get_sticker_axes(initial_normal);
                let u_vec = total_piece_rot.transform_vector3(u_axis) * size;
                let v_vec = total_piece_rot.transform_vector3(v_axis) * size;
                let sticker_center = current_pos_3d + current_normal * 0.5; // ピースの表面

                let corners = [
                    sticker_center - u_vec - v_vec,
                    sticker_center + u_vec - v_vec,
                    sticker_center + u_vec + v_vec,
                    sticker_center - u_vec + v_vec,
                ];

                let mut transformed_corners = Vec::with_capacity(4);
                let mut avg_z = 0.0;
                for &p in &corners {
                    let p_view = view_mat.transform_point3(p);
                    transformed_corners.push(project_point(p, &view_mat, scale, center));
                    avg_z += p_view.z;
                }
                avg_z /= 4.0;

                // ステッカーのインデックスを特定（ハイライト用）
                let face = normal_to_face(current_normal);
                let local_idx = face_to_local_index(face, piece.current_pos); // ここはオリジナルの位置で判定
                let abs_idx = face.start_index() + local_idx;

                // 矢印の向きを計算 (Piece の回転そのものを使用)
                // v_vec は Piece の初期 Up 方向 (v_axis) を現在の Piece 回転で回したもの。
                // これが物理的な矢印の向きそのものであるため、orientation による追加回転は不要。
                let arrow_dir = v_vec;

                let arrow_base_3d = sticker_center - arrow_dir * (VIEW3D_ARROW_VEC_SCALE * 0.3);
                let arrow_end_3d = sticker_center + arrow_dir * (VIEW3D_ARROW_VEC_SCALE * 0.7);

                draw_faces.push(DrawFace {
                    z_score: avg_z,
                    points: transformed_corners,
                    color: crate::gui::renderer::color_to_color32(sticker.color),
                    sticker_index: abs_idx,
                    center_2d: project_point(arrow_base_3d, &view_mat, scale, center),
                    arrow_end_2d: project_point(arrow_end_3d, &view_mat, scale, center),
                });
            }
        }
    }

    // Zソート
    draw_faces.sort_by(|a, b| {
        a.z_score
            .partial_cmp(&b.z_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // 描画
    for face in draw_faces {
        painter.add(egui::Shape::convex_polygon(
            face.points.clone(),
            face.color,
            Stroke::new(0.5, Color32::BLACK),
        ));

        if let Some(face_idx) = highlight_face_index {
            let start = face_idx * 9;
            if face.sticker_index >= start && face.sticker_index < start + 9 {
                painter.add(egui::Shape::convex_polygon(
                    face.points.clone(),
                    Color32::TRANSPARENT,
                    Stroke::new(3.0, Color32::from_rgb(255, 140, 0)),
                ));
            }
        }

        draw_arrow_3d(
            painter,
            face.center_2d,
            face.arrow_end_2d,
            Color32::BLACK,
            VIEW3D_ARROW_WIDTH,
        );
    }
}

fn move_to_anim_params(mv: Move) -> (Vec3, i8, f32) {
    let pi_2 = std::f32::consts::FRAC_PI_2;
    match mv {
        Move::R => (Vec3::X, 1, -pi_2),
        Move::Rp => (Vec3::X, 1, pi_2),
        Move::R2 => (Vec3::X, 1, pi_2 * 2.0),
        Move::L => (Vec3::X, -1, pi_2),
        Move::Lp => (Vec3::X, -1, -pi_2),
        Move::L2 => (Vec3::X, -1, pi_2 * 2.0),
        Move::U => (Vec3::Y, 1, -pi_2),
        Move::Up => (Vec3::Y, 1, pi_2),
        Move::U2 => (Vec3::Y, 1, pi_2 * 2.0),
        Move::D => (Vec3::Y, -1, pi_2),
        Move::Dp => (Vec3::Y, -1, -pi_2),
        Move::D2 => (Vec3::Y, -1, pi_2 * 2.0),
        Move::F => (Vec3::Z, 1, -pi_2),
        Move::Fp => (Vec3::Z, 1, pi_2),
        Move::F2 => (Vec3::Z, 1, pi_2 * 2.0),
        Move::B => (Vec3::Z, -1, pi_2),
        Move::Bp => (Vec3::Z, -1, -pi_2),
        Move::B2 => (Vec3::Z, -1, pi_2 * 2.0),
        Move::M => (Vec3::X, 0, pi_2),
        Move::Mp => (Vec3::X, 0, -pi_2),
        Move::M2 => (Vec3::X, 0, pi_2 * 2.0),
        Move::E => (Vec3::Y, 0, pi_2),
        Move::Ep => (Vec3::Y, 0, -pi_2),
        Move::E2 => (Vec3::Y, 0, pi_2 * 2.0),
        Move::S => (Vec3::Z, 0, -pi_2),
        Move::Sp => (Vec3::Z, 0, pi_2),
        Move::S2 => (Vec3::Z, 0, pi_2 * 2.0),
        Move::X => (Vec3::X, 2, -90.0_f32.to_radians()),
        Move::Xp => (Vec3::X, 2, 90.0_f32.to_radians()),
        Move::X2 => (Vec3::X, 2, 180.0_f32.to_radians()),
        Move::Y => (Vec3::Y, 2, -90.0_f32.to_radians()),
        Move::Yp => (Vec3::Y, 2, 90.0_f32.to_radians()),
        Move::Y2 => (Vec3::Y, 2, 180.0_f32.to_radians()),
        Move::Z => (Vec3::Z, 2, -90.0_f32.to_radians()),
        Move::Zp => (Vec3::Z, 2, 90.0_f32.to_radians()),
        Move::Z2 => (Vec3::Z, 2, 180.0_f32.to_radians()),
    }
}

fn is_affected(pos: Vec3, axis: Vec3, layer: i8) -> bool {
    if layer == 2 {
        return true;
    }
    if axis == Vec3::X {
        (pos.x.round() as i8) == layer
    } else if axis == Vec3::Y {
        (pos.y.round() as i8) == layer
    } else if axis == Vec3::Z {
        (pos.z.round() as i8) == layer
    } else {
        false
    }
}

fn get_sticker_axes(normal: Vec3) -> (Vec3, Vec3) {
    let n = Vec3::new(normal.x.round(), normal.y.round(), normal.z.round());
    if n == Vec3::Y {
        (Vec3::X, -Vec3::Z)
    } else if n == -Vec3::Y {
        (Vec3::X, Vec3::Z)
    } else if n == -Vec3::X {
        (Vec3::Z, Vec3::Y)
    } else if n == Vec3::X {
        (-Vec3::Z, Vec3::Y)
    } else if n == Vec3::Z {
        (Vec3::X, Vec3::Y)
    } else if n == -Vec3::Z {
        (-Vec3::X, Vec3::Y)
    } else {
        (Vec3::X, Vec3::Y)
    }
}

fn normal_to_face(normal: Vec3) -> Face {
    let n = Vec3::new(normal.x.round(), normal.y.round(), normal.z.round());
    if n.y > 0.5 {
        Face::Up
    } else if n.y < -0.5 {
        Face::Down
    } else if n.x < -0.5 {
        Face::Left
    } else if n.x > 0.5 {
        Face::Right
    } else if n.z > 0.5 {
        Face::Front
    } else {
        Face::Back
    }
}

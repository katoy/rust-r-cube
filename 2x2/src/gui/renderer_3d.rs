use crate::cube::{Cube, Move};
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

/// ステッカーの3D情報
#[derive(Clone, Copy)]
struct Sticker3D {
    index: usize,
    center: Vec3,
    normal: Vec3,
    u_vec: Vec3, // ステッカーの「右」方向
    v_vec: Vec3, // ステッカーの「下」方向
}

/// 描画用の頂点データ
struct DrawFace {
    z_score: f32,
    points: Vec<Pos2>,
    color: Color32,
    sticker_index: usize,
    center_2d: Pos2,
    u_vec_2d: egui::Vec2,
    v_vec_2d: egui::Vec2,
}

/// 3D投影ヘルパー関数
fn project_point(
    p: Vec3,
    model_mat: &Mat4,
    view_mat: &Mat4,
    scale: f32,
    screen_center: Pos2,
) -> Pos2 {
    let world = model_mat.transform_point3(p);
    let view = view_mat.transform_point3(world);
    let distance = VIEW3D_CAMERA_DISTANCE - view.z;
    let perspective = VIEW3D_CAMERA_DISTANCE / distance;
    Pos2::new(
        screen_center.x + view.x * scale * perspective,
        screen_center.y - view.y * scale * perspective,
    )
}

/// ステッカーの初期3D配置を生成
fn get_initial_stickers() -> Vec<Sticker3D> {
    let mut stickers = Vec::with_capacity(crate::cube::NUM_STICKERS);
    let size = VIEW3D_STICKER_SIZE;

    // 各面の定義: (normal, center_base, u_axis, v_axis, flip_v)
    let face_defs = [
        (Vec3::Y, Vec3::Y, Vec3::X, Vec3::Z, false),   // U
        (-Vec3::Y, -Vec3::Y, Vec3::X, Vec3::Z, true),  // D
        (-Vec3::X, -Vec3::X, Vec3::Z, Vec3::Y, true),  // L
        (Vec3::X, Vec3::X, -Vec3::Z, Vec3::Y, true),   // R
        (Vec3::Z, Vec3::Z, Vec3::X, Vec3::Y, true),    // F
        (-Vec3::Z, -Vec3::Z, -Vec3::X, Vec3::Y, true), // B
    ];

    for (f_idx, (normal, center_base, u_axis, v_axis, flip_v)) in face_defs.iter().enumerate() {
        for i in 0..crate::cube::STICKERS_PER_FACE {
            let col = (i % 2) as f32;
            let row = (i / 2) as f32;
            let u_val = (col - 0.5) * 1.0;
            let v_val = if *flip_v {
                (1.0 - row - 0.5) * 1.0
            } else {
                (row - 0.5) * 1.0
            };

            stickers.push(Sticker3D {
                index: f_idx * crate::cube::STICKERS_PER_FACE + i,
                center: *center_base + *u_axis * u_val + *v_axis * v_val,
                normal: *normal,
                u_vec: *u_axis * size,
                v_vec: *v_axis * (if *flip_v { -size } else { size }),
            });
        }
    }

    stickers
}

/// 3D空間で矢印を描画
fn draw_arrow_3d(painter: &egui::Painter, center: Pos2, target: Pos2, color: Color32, width: f32) {
    let direction = target - center;
    let arrow_length = direction.length();
    if arrow_length < 0.1 {
        return; // 矢印が極端に小さい場合はスキップ
    }

    let dir_normalized = direction.normalized();
    let arrow_end = center + dir_normalized * arrow_length;

    // 矢印の本体
    painter.line_segment([center, arrow_end], Stroke::new(width, color));

    // 矢印の先端（三角形）
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
    let rotation = Mat4::from_rotation_x(view.pitch) * Mat4::from_rotation_y(view.yaw);
    let view_mat = rotation; // カメラ位置は固定、オブジェクトを回転させるイメージ

    // アニメーション情報取得: (軸, レイヤー(1 or -1), 基本角度(正負込))
    let (anim_axis, anim_layer, anim_full_angle) = if let Some(anim) = animation {
        match anim.current_move {
            Move::R => (Vec3::X, 1, -90.0_f32.to_radians()),
            Move::Rp => (Vec3::X, 1, 90.0_f32.to_radians()),
            Move::R2 => (Vec3::X, 1, -180.0_f32.to_radians()),
            Move::L => (Vec3::X, -1, 90.0_f32.to_radians()),
            Move::Lp => (Vec3::X, -1, -90.0_f32.to_radians()),
            Move::L2 => (Vec3::X, -1, 180.0_f32.to_radians()),
            Move::U => (Vec3::Y, 1, -90.0_f32.to_radians()),
            Move::Up => (Vec3::Y, 1, 90.0_f32.to_radians()),
            Move::U2 => (Vec3::Y, 1, -180.0_f32.to_radians()),
            Move::D => (Vec3::Y, -1, 90.0_f32.to_radians()),
            Move::Dp => (Vec3::Y, -1, -90.0_f32.to_radians()),
            Move::D2 => (Vec3::Y, -1, 180.0_f32.to_radians()),
            Move::F => (Vec3::Z, 1, -90.0_f32.to_radians()),
            Move::Fp => (Vec3::Z, 1, 90.0_f32.to_radians()),
            Move::F2 => (Vec3::Z, 1, -180.0_f32.to_radians()),
            Move::B => (Vec3::Z, -1, 90.0_f32.to_radians()),
            Move::Bp => (Vec3::Z, -1, -90.0_f32.to_radians()),
            Move::B2 => (Vec3::Z, -1, 180.0_f32.to_radians()),
        }
    } else {
        (Vec3::X, 0, 0.0)
    };

    let anim_angle = if let Some(anim) = animation {
        anim_full_angle * anim.eased_progress()
    } else {
        0.0
    };

    let initial_stickers = get_initial_stickers();
    let mut draw_faces = Vec::new();

    for sticker_def in initial_stickers {
        // 現在のステッカーの状態（色）を取得
        let sticker_data = cube.get_sticker(sticker_def.index);
        let color = crate::gui::renderer::color_to_color32(sticker_data.color);

        // アニメーション回転の適用
        let mut model_mat = Mat4::IDENTITY;

        if animation.is_some() {
            let is_affected = match anim_axis {
                v if v == Vec3::X => {
                    (sticker_def.center.x > 0.0 && anim_layer == 1)
                        || (sticker_def.center.x < 0.0 && anim_layer == -1)
                }
                v if v == Vec3::Y => {
                    (sticker_def.center.y > 0.0 && anim_layer == 1)
                        || (sticker_def.center.y < 0.0 && anim_layer == -1)
                }
                v if v == Vec3::Z => {
                    (sticker_def.center.z > 0.0 && anim_layer == 1)
                        || (sticker_def.center.z < 0.0 && anim_layer == -1)
                }
                _ => false,
            };

            if is_affected {
                model_mat = Mat4::from_axis_angle(anim_axis, anim_angle);
            }
        }

        // 頂点の計算
        let corners = [
            sticker_def.center - sticker_def.u_vec - sticker_def.v_vec,
            sticker_def.center + sticker_def.u_vec - sticker_def.v_vec,
            sticker_def.center + sticker_def.u_vec + sticker_def.v_vec,
            sticker_def.center - sticker_def.u_vec + sticker_def.v_vec,
        ];

        let mut transformed_corners = Vec::with_capacity(4);
        let mut avg_z = 0.0;

        // 法線の変換
        let normal_transformed =
            view_mat.transform_vector3(model_mat.transform_vector3(sticker_def.normal));

        // バックフェイスカリング（簡易）
        // view_matで変換した結果、Zが正ならカメラに向いている
        if normal_transformed.z > VIEW3D_BACKFACE_CULLING_THRESHOLD {
            // 少し余裕を持たせる
            for p in corners {
                let p_world = model_mat.transform_point3(p);
                let p_view = view_mat.transform_point3(p_world);

                // 透視投影っぽい効果 (Zに応じてスケール)
                let distance = VIEW3D_CAMERA_DISTANCE - p_view.z; // カメラ距離
                let perspective = VIEW3D_CAMERA_DISTANCE / distance;

                let x = center.x + p_view.x * scale * perspective;
                let y = center.y - p_view.y * scale * perspective; // Y-up to Y-down screen

                transformed_corners.push(Pos2::new(x, y));
                avg_z += p_view.z;
            }
            avg_z /= 4.0;

            // ステッカーの中心を計算（2D投影後）
            let center_2d = project_point(sticker_def.center, &model_mat, &view_mat, scale, center);

            // U方向とV方向のベクトルを計算（2D投影後）
            // ベクトルの先端を計算してから差分を取る
            let u_end_2d = project_point(
                sticker_def.center + sticker_def.u_vec * VIEW3D_ARROW_VEC_SCALE,
                &model_mat,
                &view_mat,
                scale,
                center,
            );
            let v_end_2d = project_point(
                sticker_def.center + sticker_def.v_vec * VIEW3D_ARROW_VEC_SCALE,
                &model_mat,
                &view_mat,
                scale,
                center,
            );

            let u_vec_2d = u_end_2d - center_2d;
            let v_vec_2d = v_end_2d - center_2d;

            draw_faces.push(DrawFace {
                z_score: avg_z,
                points: transformed_corners,
                color,
                sticker_index: sticker_def.index,
                center_2d,
                u_vec_2d,
                v_vec_2d,
            });
        }
    }

    // Zソート（奥から手前へ）
    draw_faces.sort_by(|a, b| {
        a.z_score
            .partial_cmp(&b.z_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // 描画
    for face in draw_faces {
        if face.points.len() >= 3 {
            painter.add(egui::Shape::convex_polygon(
                face.points.clone(),
                face.color,
                Stroke::new(1.0_f32, Color32::BLACK), // 枠線
            ));

            // 編集中の面をハイライト表示
            if let Some(face_idx) = highlight_face_index {
                let start_idx = face_idx * crate::cube::STICKERS_PER_FACE;
                if face.sticker_index >= start_idx
                    && face.sticker_index < start_idx + crate::cube::STICKERS_PER_FACE
                {
                    // 太いオレンジの枠線で囲む
                    painter.add(egui::Shape::convex_polygon(
                        face.points.clone(),
                        Color32::TRANSPARENT,
                        Stroke::new(3.0_f32, Color32::from_rgb(255, 140, 0)),
                    ));
                }
            }

            // ステッカーの向きを示す矢印を描画
            let sticker_data = cube.get_sticker(face.sticker_index);
            // orientationに応じて方向を決定
            // 0: 上 (-v), 1: 右 (+u), 2: 下 (+v), 3: 左 (-u)
            // ※ rotate_cwで+1されるため
            let arrow_vec = match sticker_data.orientation {
                0 => -face.v_vec_2d,
                1 => face.u_vec_2d,
                2 => face.v_vec_2d,
                3 => -face.u_vec_2d,
                _ => egui::Vec2::ZERO,
            };

            // 黒い矢印のみ描画（太く、大きく）
            draw_arrow_3d(
                painter,
                face.center_2d,
                face.center_2d + arrow_vec,
                Color32::BLACK,
                VIEW3D_ARROW_WIDTH,
            );
        }
    }
}

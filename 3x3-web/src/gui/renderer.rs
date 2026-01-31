use crate::cube::{Color, Cube, Face, Move, Sticker, STICKERS_PER_FACE};
use crate::gui::app::AnimationState;
use crate::gui::constants::*;
use egui::{Color32, Painter, Pos2, Rect, Stroke, Vec2};

/// キューブのステッカー色（[`Color`]）を egui の [`Color32`] に変換します。
///
/// # 引数
///
/// - `color` - 変換元のステッカー色
///
/// # 戻り値
///
/// egui で描画可能な Color32 値
#[must_use]
pub fn color_to_color32(color: Color) -> Color32 {
    match color {
        Color::White => COLOR_WHITE,
        Color::Yellow => COLOR_YELLOW,
        Color::Green => COLOR_GREEN,
        Color::Blue => COLOR_BLUE,
        Color::Red => COLOR_RED,
        Color::Orange => COLOR_ORANGE,
        Color::Gray => COLOR_GRAY,
    }
}

/// 単一のステッカーを描画します。
///
/// 指定された回転角度や透明度を適用して、正方形のステッカーとその上の向きを示す矢印を描画します。
fn draw_sticker(
    painter: &Painter,
    center: Pos2,
    size: f32,
    sticker: Sticker,
    rotation_offset_deg: f32,
    alpha: f32,
    shadow_offset: Vec2,
) {
    let color = color_to_color32(sticker.color).linear_multiply(alpha);
    let stroke_color = Color32::BLACK.linear_multiply(alpha);

    // 影の描画 (もしあれば)
    if shadow_offset.length() > 0.1 {
        let shadow_color = Color32::from_black_alpha((100.0 * alpha) as u8);
        let shadow_rect = Rect::from_center_size(
            center + shadow_offset,
            Vec2::splat(size * STICKER_SIZE_RATIO),
        );
        painter.rect_filled(shadow_rect, STICKER_ROUNDING, shadow_color);
    }

    // ステッカーの背景を描画
    let rect = Rect::from_center_size(center, Vec2::splat(size * STICKER_SIZE_RATIO));

    // 回転を適用した矩形を描画するために、頂点を計算して回転させる
    if rotation_offset_deg.abs() > 0.1 {
        let angle = rotation_offset_deg.to_radians();
        let cos = angle.cos();
        let sin = angle.sin();

        let half = size * STICKER_SIZE_RATIO / 2.0;
        let corners = [
            Pos2::new(-half, -half),
            Pos2::new(half, -half),
            Pos2::new(half, half),
            Pos2::new(-half, half),
        ];

        let rotated_corners: Vec<Pos2> = corners
            .iter()
            .map(|p| {
                Pos2::new(
                    center.x + p.x * cos - p.y * sin,
                    center.y + p.x * sin + p.y * cos,
                )
            })
            .collect();

        painter.add(egui::Shape::convex_polygon(
            rotated_corners.clone(),
            color,
            Stroke::new(STICKER_STROKE_WIDTH, stroke_color),
        ));
    } else {
        painter.rect_filled(rect, STICKER_ROUNDING, color);
        painter.rect_stroke(
            rect,
            STICKER_ROUNDING,
            Stroke::new(STICKER_STROKE_WIDTH, stroke_color),
        );
    }

    // 矢印を描画（向きを示す）
    let arrow_rotation = (sticker.orientation as f32 * 90.0 + rotation_offset_deg).to_radians();
    draw_arrow(
        painter,
        center,
        size * ARROW_LENGTH_RATIO,
        arrow_rotation,
        alpha,
    );
}

/// ステッカーの向き（orientation）を示す矢印を描画します。
fn draw_arrow(painter: &Painter, center: Pos2, length: f32, rotation: f32, alpha: f32) {
    let cos = rotation.cos();
    let sin = rotation.sin();

    // 矢印の先端
    let tip = Pos2::new(center.x + length * sin, center.y - length * cos);

    // 矢印の根元
    let base = Pos2::new(
        center.x - length * ARROW_BASE_RATIO * sin,
        center.y + length * ARROW_BASE_RATIO * cos,
    );

    // 矢印の羽
    let wing_length = length * ARROW_WING_RATIO;
    let wing_angle = ARROW_WING_ANGLE_DEG.to_radians();

    let left_wing = Pos2::new(
        tip.x - wing_length * (rotation + wing_angle).sin(),
        tip.y + wing_length * (rotation + wing_angle).cos(),
    );

    let right_wing = Pos2::new(
        tip.x - wing_length * (rotation - wing_angle).sin(),
        tip.y + wing_length * (rotation - wing_angle).cos(),
    );

    // 矢印を描画
    // Color32::from_black_alpha(180) は alpha=180/255 相当。
    // alpha引数を反映させるため、Color32::BLACK.linear_multiply(alpha)をベースに調整してもいいが、
    // ここでは単純に linear_multiply を使う
    let stroke = Stroke::new(
        ARROW_WIDTH,
        Color32::from_black_alpha(ARROW_ALPHA).linear_multiply(alpha),
    );
    painter.line_segment([base, tip], stroke);
    painter.line_segment([tip, left_wing], stroke);
    painter.line_segment([tip, right_wing], stroke);
}

/// インデックスに対応するグリッド座標 (col, row) を取得
fn get_grid_coords(index: usize) -> Pos2 {
    let (col, row) = match index {
        i if i < 9 => (3.0 + (i % 3) as f32, 0.0 + (i / 3) as f32), // U
        i if i < 18 => (3.0 + ((i - 9) % 3) as f32, 6.0 + ((i - 9) / 3) as f32), // D
        i if i < 27 => (0.0 + ((i - 18) % 3) as f32, 3.0 + ((i - 18) / 3) as f32), // L
        i if i < 36 => (6.0 + ((i - 27) % 3) as f32, 3.0 + ((i - 27) / 3) as f32), // R
        i if i < 45 => (3.0 + ((i - 36) % 3) as f32, 3.0 + ((i - 36) / 3) as f32), // F
        _ => (
            9.0 + ((index - 45) % 3) as f32,
            3.0 + ((index - 45) / 3) as f32,
        ), // B
    };
    Pos2::new(col, row)
}

/// アニメーション情報の型エイリアス: (移動マッピング, 回転面情報)
type AnimationInfo = (Vec<(usize, usize)>, Option<(usize, f32)>);

/// アニメーション情報：移動マッピングと回転面情報
fn get_animation_info(mv: Move) -> AnimationInfo {
    let all_moves = Move::all_moves();
    let move_idx = all_moves.iter().position(|&m| m == mv).unwrap_or(0);

    let mut mapping = Vec::new();
    let (face_start, _) = crate::gui::mapping::FACE_ROTATION_TABLE[move_idx];

    for &(src, dst) in &crate::gui::mapping::MOVE_MAPPING_TABLE[move_idx] {
        if src == 99 {
            continue;
        }

        // 移動があるステッカー、または回転面上のステッカーのみを対象にする
        let is_on_face = if face_start != usize::MAX {
            src >= face_start && src < face_start + STICKERS_PER_FACE
        } else {
            false
        };

        if src != dst || is_on_face {
            mapping.push((src, dst));
        }
    }

    (
        mapping,
        Some(crate::gui::mapping::FACE_ROTATION_TABLE[move_idx]),
    )
}

/// キューブを展開図（2D）として指定された領域に描画します。
///
/// 現在のアニメーション状態（[`AnimationState`]）がある場合は、
/// 回転や移動の演出を適用して描画します。
///
/// # 引数
///
/// - `ui` - egui の Ui コンテキスト
/// - `rect` - 描画先の矩形領域
/// - `cube` - 描画対象のキューブ
/// - `animation` - 現在実行中のアニメーション状態（任意）
/// - `highlight_face_index` - ハイライト表示する面のインデックス（任意、0-5）
pub fn draw_cube(
    ui: &mut egui::Ui,
    mut rect: Rect,
    cube: &Cube,
    animation: Option<&AnimationState>,
    highlight_face_index: Option<usize>,
) {
    if !rect.is_finite() {
        rect = ui.available_rect_before_wrap();
    }
    let painter = ui.painter();

    let grid_cols = GRID_COLS;
    let grid_rows = GRID_ROWS;

    // グリッドサイズ計算
    let mut grid_size =
        (rect.width() / grid_cols).min(rect.height() / grid_rows) * GRID_PADDING_RATIO;
    if grid_size.is_nan() || grid_size < 0.1 {
        grid_size = 1.0;
    }
    let sticker_size = grid_size * STICKER_SIZE_IN_GRID;

    let total_width = grid_size * grid_cols;
    let total_height = grid_size * grid_rows;

    let start_x = rect.left() + (rect.width() - total_width) / 2.0;
    let start_y = rect.top() + (rect.height() - total_height) / 2.0;
    let base_pos = Pos2::new(start_x + grid_size * 0.5, start_y + grid_size * 0.5);

    let to_screen = |grid_p: Pos2| -> Pos2 {
        Pos2::new(
            base_pos.x + grid_p.x * grid_size,
            base_pos.y + grid_p.y * grid_size,
        )
    };

    let (anim_mapping, anim_face_rot) = if let Some(anim) = animation {
        get_animation_info(anim.current_move)
    } else {
        (vec![], None)
    };

    // 0. 回転面の強調表示 (Face Overlay)
    if let Some((face_start, _angle)) = anim_face_rot {
        if face_start != usize::MAX {
            let face_grid_rect = get_face_grid_rect(face_start);
            let top_left = to_screen(face_grid_rect.min) - Vec2::splat(grid_size * 0.5);
            let bottom_right = to_screen(Pos2::new(
                face_grid_rect.max.x - 1.0,
                face_grid_rect.max.y - 1.0,
            )) + Vec2::splat(grid_size * 0.5);
            let highlight_rect = Rect::from_min_max(top_left, bottom_right);

            painter.rect_filled(
                highlight_rect.expand(2.0),
                5.0,
                Color32::from_rgba_premultiplied(128, 128, 128, ANIMATION_FACE_HIGHLIGHT_ALPHA),
            );
        }
    }

    // 全ステッカーを描画
    for i in 0..crate::cube::NUM_STICKERS {
        let grid_pos = get_grid_coords(i);
        let screen_pos = to_screen(grid_pos);
        let mut drawn = false;

        let mut sticker = cube.get_sticker(i);

        if let Some(anim) = animation {
            let progress = anim.eased_progress();

            // 描画対象を決定: 移動中のスロットであればソースからステッカーを取得
            if let Some(&(src, _)) = anim_mapping.iter().find(|(_, dst)| *dst == i) {
                let moving_from_idx = src;
                let oris_delta =
                    crate::gui::mapping::get_oris_delta(anim.current_move, moving_from_idx);

                // Piece ベースに移行したため、現在の位置 (i) にあるステッカーを取得します。
                // アニメーションの開始時 (t=0) の向きを再現するため、oris_delta 分だけ戻した値を src_ori とします。
                sticker = cube.get_sticker(i);
                let src_ori = (sticker.orientation as i32 + 4 - (oris_delta % 4) as i32) as u8 % 4;

                let target_grid_pos = grid_pos; //目的地
                let start_grid_pos = get_grid_coords(src); //出発地
                let dist = start_grid_pos.distance(target_grid_pos);

                let start_screen = to_screen(start_grid_pos);
                let end_screen = to_screen(target_grid_pos);

                let (is_arc, rotation_face_start) = match anim.current_move {
                    Move::U | Move::Up | Move::U2 => (matches!(i, 0..=8), 0),
                    Move::D | Move::Dp | Move::D2 => (matches!(i, 9..=17), 9),
                    Move::L | Move::Lp | Move::L2 => (matches!(i, 18..=26), 18),
                    Move::R | Move::Rp | Move::R2 => (matches!(i, 27..=35), 27),
                    Move::F | Move::Fp | Move::F2 => (
                        matches!(i, 36..=44 | 6..=8 | 27 | 30 | 33 | 9..=11 | 20 | 23 | 26),
                        36,
                    ),
                    Move::B | Move::Bp | Move::B2 => (matches!(i, 45..=53), 45),
                    Move::S | Move::Sp | Move::S2 => (
                        matches!(i, 3..=5 | 28 | 31 | 34 | 12..=14 | 19 | 22 | 25),
                        36,
                    ),
                    _ => (false, 0),
                };

                // アニメーション中のステッカーの論理的向きは、開始時の状態に固定して描画する
                // (実際の回転は sub_rot や p_rot で表現される)
                sticker.orientation = src_ori;

                let calc_state = |p: f32| {
                    let mut pos = start_screen + (end_screen - start_screen) * p;
                    let mut rot = 0.0;
                    let mut alpha = 1.0;
                    let mut size_scale = 1.0;
                    let mut shadow = Vec2::ZERO;

                    if is_arc {
                        if let Some((_, angle)) = anim_face_rot {
                            let center_grid_base = get_face_grid_rect(rotation_face_start).min;
                            let center_grid =
                                Pos2::new(center_grid_base.x + 1.0, center_grid_base.y + 1.0);
                            let center_screen = to_screen(center_grid);

                            let r = start_screen.distance(center_screen);
                            let start_angle = (start_screen.y - center_screen.y)
                                .atan2(start_screen.x - center_screen.x);
                            let current_angle = start_angle + (angle * p).to_radians();

                            pos = Pos2::new(
                                center_screen.x + current_angle.cos() * r,
                                center_screen.y + current_angle.sin() * r,
                            );
                            rot = angle * p;
                        }
                    } else {
                        // 直線移動 (Slide) の場合
                        let mid_p = (p * std::f32::consts::PI).sin();
                        let is_slice = matches!(
                            anim.current_move,
                            Move::M
                                | Move::Mp
                                | Move::M2
                                | Move::E
                                | Move::Ep
                                | Move::E2
                                | Move::S
                                | Move::Sp
                                | Move::S2
                        );

                        if dist >= 3.0 && !is_slice {
                            let bulge_val = ANIMATION_JUMP_BULGE_RATIO * grid_size;
                            pos.y -= bulge_val * mid_p;
                            alpha = 1.0 - 0.5 * mid_p;
                            size_scale = 1.0 - 0.3 * mid_p;
                            shadow = Vec2::new(
                                ANIMATION_JUMP_SHADOW_OFFSET,
                                ANIMATION_JUMP_SHADOW_OFFSET,
                            ) * mid_p;
                        } else if !is_slice {
                            pos.y += ANIMATION_BULGE_RATIO * grid_size * mid_p;
                            shadow =
                                Vec2::new(ANIMATION_SHADOW_OFFSET, ANIMATION_SHADOW_OFFSET) * mid_p;
                        }
                    }
                    (pos, rot, size_scale, alpha, shadow)
                };

                let (p_pos, p_rot, p_scale, p_alpha, p_shadow) = calc_state(progress);

                let move_angle = if is_arc {
                    if let Some((_, angle)) = anim_face_rot {
                        angle
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };

                // 目標とする自転量 (oris_delta * 90) から、アニメーションによる回転分 (move_angle) を引いた残りを補完
                let target_sub_rot = (oris_delta as f32 * 90.0 - move_angle).rem_euclid(360.0);
                let target_sub_rot = if target_sub_rot > 180.0 {
                    target_sub_rot - 360.0
                } else {
                    target_sub_rot
                };
                let sub_rot = target_sub_rot * progress;

                for ghost_t in [0.05, 0.1] {
                    let t = (progress - ghost_t).max(0.0);
                    if t > 0.0 {
                        let (g_pos, g_rot, g_scale, g_alpha, _) = calc_state(t);
                        draw_sticker(
                            painter,
                            g_pos,
                            sticker_size * g_scale * (1.0 - ghost_t * 2.0),
                            sticker,
                            g_rot + sub_rot,
                            ANIMATION_TRAIL_ALPHA_FACTOR
                                * g_alpha
                                * p_alpha
                                * (1.0 - ghost_t * 5.0),
                            Vec2::ZERO,
                        );
                    }
                }

                draw_sticker(
                    painter,
                    p_pos,
                    sticker_size * p_scale,
                    sticker,
                    p_rot + sub_rot,
                    p_alpha,
                    p_shadow,
                );
                drawn = true;
            } else if anim_mapping.iter().any(|(src_m, _)| *src_m == i) {
                drawn = true;
            }
        }

        if !drawn {
            draw_sticker(
                painter,
                screen_pos,
                sticker_size,
                sticker,
                0.0, // 基準向きを 0.0 (UP) に統一
                1.0,
                Vec2::ZERO,
            );
        }
    }

    // ハイライト
    if let Some(face_idx) = highlight_face_index {
        let face_grid_rect = get_face_grid_rect(face_idx * STICKERS_PER_FACE);
        let top_left = to_screen(face_grid_rect.min) - Vec2::splat(grid_size * 0.5);
        let bottom_right = to_screen(Pos2::new(
            face_grid_rect.max.x - 1.0,
            face_grid_rect.max.y - 1.0,
        )) + Vec2::splat(grid_size * 0.5);
        let padding = grid_size * HIGHLIGHT_PADDING_RATIO;
        let highlight_rect = Rect::from_min_max(
            top_left - Vec2::splat(padding),
            bottom_right + Vec2::splat(padding),
        );
        painter.rect_stroke(
            highlight_rect,
            5.0,
            Stroke::new(HIGHLIGHT_STROKE_WIDTH, HIGHLIGHT_COLOR),
        );
    }

    if let Some(anim) = animation {
        let text = format!(
            "動作: {} ({:.0}%)",
            anim.current_move,
            anim.progress * 100.0
        );
        painter.text(
            rect.left_bottom() + egui::vec2(10.0, -30.0),
            egui::Align2::LEFT_BOTTOM,
            text,
            egui::FontId::proportional(16.0),
            Color32::BLACK,
        );
    }
}

/// 移動するステッカーの向きの変更量を取得
///
fn get_face_grid_rect(index: usize) -> Rect {
    let (min_col, min_row) = match index {
        i if i == Face::Up.start_index() => (3.0, 0.0),   // U
        i if i == Face::Down.start_index() => (3.0, 6.0), // D
        i if i == Face::Left.start_index() => (0.0, 3.0), // L
        i if i == Face::Right.start_index() => (6.0, 3.0), // R
        i if i == Face::Front.start_index() => (3.0, 3.0), // F
        i if i == Face::Back.start_index() => (9.0, 3.0), // B
        _ => (0.0, 0.0),
    };
    // 3x3なのでサイズは3.0x3.0
    Rect::from_min_size(Pos2::new(min_col, min_row), Vec2::new(3.0, 3.0))
}

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
        i if i < Face::Down.start_index() => (2.0 + (i % 2) as f32, 0.0 + (i / 2) as f32), // U
        i if i < Face::Left.start_index() => {
            (2.0 + ((i - 4) % 2) as f32, 4.0 + ((i - 4) / 2) as f32)
        } // D
        i if i < Face::Right.start_index() => {
            (0.0 + ((i - 8) % 2) as f32, 2.0 + ((i - 8) / 2) as f32)
        } // L
        i if i < Face::Front.start_index() => {
            (4.0 + ((i - 12) % 2) as f32, 2.0 + ((i - 12) / 2) as f32)
        } // R
        i if i < Face::Back.start_index() => {
            (2.0 + ((i - 16) % 2) as f32, 2.0 + ((i - 16) / 2) as f32)
        } // F
        _ => (
            6.0 + ((index - 20) % 2) as f32,
            2.0 + ((index - 20) / 2) as f32,
        ), // B
    };
    Pos2::new(col, row)
}

/// アニメーション情報の型エイリアス: (移動マッピング, 回転面情報)
type AnimationInfo = (Vec<(usize, usize)>, Option<(usize, f32)>);

/// アニメーション情報（移動マッピングと回転面情報）のデータテーブル
/// 順序は Move::all_moves() に準拠: R, Rp, R2, L, Lp, L2, U, Up, U2, D, Dp, D2, F, Fp, F2, B, Bp, B2
const MOVE_MAPPING_TABLE: [[(usize, usize); 8]; 18] = [
    [
        (17, 1),
        (19, 3),
        (1, 22),
        (3, 20),
        (22, 5),
        (20, 7),
        (5, 17),
        (7, 19),
    ], // R
    [
        (1, 17),
        (3, 19),
        (22, 1),
        (20, 3),
        (5, 22),
        (7, 20),
        (17, 5),
        (19, 7),
    ], // Rp
    [
        (17, 22),
        (19, 20),
        (1, 5),
        (3, 7),
        (22, 17),
        (20, 19),
        (5, 1),
        (7, 3),
    ], // R2
    [
        (23, 0),
        (21, 2),
        (0, 16),
        (2, 18),
        (16, 4),
        (18, 6),
        (4, 23),
        (6, 21),
    ], // L
    [
        (0, 23),
        (2, 21),
        (16, 0),
        (18, 2),
        (4, 16),
        (6, 18),
        (23, 4),
        (21, 6),
    ], // Lp
    [
        (16, 23),
        (18, 21),
        (0, 4),
        (2, 6),
        (23, 16),
        (21, 18),
        (4, 0),
        (6, 2),
    ], // L2
    [
        (16, 8),
        (17, 9),
        (8, 20),
        (9, 21),
        (20, 12),
        (21, 13),
        (12, 16),
        (13, 17),
    ], // U
    [
        (8, 16),
        (9, 17),
        (20, 8),
        (21, 9),
        (12, 20),
        (13, 21),
        (16, 12),
        (17, 13),
    ], // Up
    [
        (16, 20),
        (17, 21),
        (8, 12),
        (9, 13),
        (20, 16),
        (21, 17),
        (12, 8),
        (13, 9),
    ], // U2
    [
        (18, 14),
        (19, 15),
        (14, 22),
        (15, 23),
        (22, 10),
        (23, 11),
        (10, 18),
        (11, 19),
    ], // D
    [
        (14, 18),
        (15, 19),
        (22, 14),
        (23, 15),
        (10, 22),
        (11, 23),
        (18, 10),
        (19, 11),
    ], // Dp
    [
        (18, 22),
        (19, 23),
        (11, 15),
        (10, 14),
        (22, 18),
        (23, 19),
        (15, 11),
        (14, 10),
    ], // D2
    [
        (11, 2),
        (9, 3),
        (2, 12),
        (3, 14),
        (12, 5),
        (14, 4),
        (5, 11),
        (4, 9),
    ], // F
    [
        (2, 11),
        (3, 9),
        (11, 5),
        (9, 4),
        (5, 12),
        (4, 14),
        (12, 2),
        (14, 3),
    ], // Fp
    [
        (2, 5),
        (3, 4),
        (12, 11),
        (14, 9),
        (5, 2),
        (4, 3),
        (11, 12),
        (9, 14),
    ], // F2
    [
        (13, 0),
        (15, 1),
        (0, 10),
        (1, 8),
        (10, 7),
        (8, 6),
        (7, 13),
        (6, 15),
    ], // B
    [
        (0, 13),
        (1, 15),
        (13, 7),
        (15, 6),
        (7, 10),
        (6, 8),
        (10, 0),
        (8, 1),
    ], // Bp
    [
        (0, 7),
        (1, 6),
        (13, 10),
        (15, 8),
        (7, 0),
        (6, 1),
        (10, 13),
        (8, 15),
    ], // B2
];

const FACE_ROTATION_TABLE: [(usize, f32); 18] = [
    (12, 90.0),
    (12, -90.0),
    (12, 180.0), // R, Rp, R2
    (8, 90.0),
    (8, -90.0),
    (8, 180.0), // L, Lp, L2
    (0, 90.0),
    (0, -90.0),
    (0, 180.0), // U, Up, U2
    (4, 90.0),
    (4, -90.0),
    (4, 180.0), // D, Dp, D2
    (16, 90.0),
    (16, -90.0),
    (16, 180.0), // F, Fp, F2
    (20, 90.0),
    (20, -90.0),
    (20, 180.0), // B, Bp, B2
];

/// アニメーション情報：移動マッピングと回転面情報
fn get_animation_info(mv: Move) -> AnimationInfo {
    let all_moves = Move::all_moves();
    let idx = all_moves.iter().position(|&m| m == mv).unwrap_or(0);
    (
        MOVE_MAPPING_TABLE[idx].to_vec(),
        Some(FACE_ROTATION_TABLE[idx]),
    )
}

/// 点を回転させる
fn rotate_point(p: Pos2, center: Pos2, angle_degrees: f32) -> Pos2 {
    let angle = angle_degrees.to_radians();
    let cos = angle.cos();
    let sin = angle.sin();
    Pos2::new(
        center.x + (p.x - center.x) * cos - (p.y - center.y) * sin,
        center.y + (p.x - center.x) * sin + (p.y - center.y) * cos,
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
    rect: Rect,
    cube: &Cube,
    animation: Option<&AnimationState>,
    highlight_face_index: Option<usize>,
) {
    let painter = ui.painter();

    let grid_cols = GRID_COLS;
    let grid_rows = GRID_ROWS;

    // グリッドサイズ計算
    let grid_size = (rect.width() / grid_cols).min(rect.height() / grid_rows) * GRID_PADDING_RATIO;
    let sticker_size = grid_size * STICKER_SIZE_IN_GRID;

    let total_width = grid_size * grid_cols;
    let total_height = grid_size * grid_rows;

    let start_x = rect.left() + (rect.width() - total_width) / 2.0;
    let start_y = rect.top() + (rect.height() - total_height) / 2.0;
    let base_pos = Pos2::new(start_x + grid_size * 0.5, start_y + grid_size * 0.5);

    // グリッド座標からスクリーン座標へ変換するクロージャ
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
        let face_grid_rect = get_face_grid_rect(face_start);
        let top_left = to_screen(face_grid_rect.min) - Vec2::splat(grid_size * 0.5);
        let bottom_right = to_screen(Pos2::new(
            face_grid_rect.max.x - 1.0,
            face_grid_rect.max.y - 1.0,
        )) + Vec2::splat(grid_size * 0.5);
        let highlight_rect = Rect::from_min_max(top_left, bottom_right);

        // 淡い色で塗りつぶし
        painter.rect_filled(
            highlight_rect.expand(2.0),
            5.0,
            Color32::from_rgba_premultiplied(255, 255, 255, ANIMATION_FACE_HIGHLIGHT_ALPHA),
        );
    }

    // 全ステッカーを描画
    for i in 0..crate::cube::NUM_STICKERS {
        let mut sticker = cube.get_sticker(i);
        let grid_pos = get_grid_coords(i);
        let mut rotation = 0.0;
        let mut screen_pos = to_screen(grid_pos);

        let mut drawn = false;

        if let Some(anim) = animation {
            let progress = anim.eased_progress();

            // 1. 回転する面のステッカー: 最終的なorientationを設定
            if let Some((face_start, _angle)) = anim_face_rot {
                if i >= face_start && i < face_start + STICKERS_PER_FACE {
                    let orientation_delta = get_face_orientation_delta(anim.current_move);
                    sticker.orientation = (sticker.orientation + orientation_delta) % 4;
                }
            }

            // 2. 移動するステッカーのorientation調整
            if let Some((_, _target_idx)) = anim_mapping.iter().find(|(src, _)| *src == i) {
                let orientation_delta = get_moving_sticker_orientation_delta(anim.current_move, i);
                if orientation_delta > 0 {
                    sticker.orientation = (sticker.orientation + orientation_delta) % 4;
                }
            }

            // 面回転の処理
            if let Some((face_start, angle)) = anim_face_rot {
                if i >= face_start && i < face_start + STICKERS_PER_FACE {
                    let center_grid_idx = face_start;
                    let center_grid_base = get_grid_coords(center_grid_idx);
                    let center_grid = Pos2::new(center_grid_base.x + 0.5, center_grid_base.y + 0.5);
                    let center_screen = to_screen(center_grid);

                    let current_angle = angle * progress;
                    screen_pos = rotate_point(screen_pos, center_screen, current_angle);

                    let orientation_delta = match anim.current_move {
                        Move::R | Move::L | Move::F | Move::B => 1,
                        Move::Rp | Move::Lp | Move::Fp | Move::Bp => 3,
                        Move::U | Move::D => 1,
                        Move::Up | Move::Dp => 3,
                        Move::U2 | Move::D2 | Move::L2 | Move::R2 | Move::F2 | Move::B2 => 2,
                    };
                    let orientation_change_deg = -(orientation_delta as f32 * 90.0);
                    rotation = current_angle + orientation_change_deg;
                }
            }

            // 移動の処理
            if let Some((_, target_idx)) = anim_mapping.iter().find(|(src, _)| *src == i) {
                let target_grid_pos = get_grid_coords(*target_idx);
                let start_grid_pos = grid_pos;
                let dist = start_grid_pos.distance(target_grid_pos);

                let start_screen = to_screen(start_grid_pos);
                let end_screen = to_screen(target_grid_pos);

                // アニメーション状態計算用のクロージャ
                let calc_state = |p: f32| {
                    let mut pos = start_screen + (end_screen - start_screen) * p;
                    let mut rot = 0.0;
                    let mut alpha = 1.0;
                    let mut size_scale = 1.0;
                    let mut shadow = Vec2::ZERO;

                    let orientation_delta =
                        get_moving_sticker_orientation_delta(anim.current_move, i);
                    let physical_delta_deg = orientation_delta as f32 * 90.0;

                    // FまたはB操作の場合はF面の中心を軸に回転させる（距離に関わらず）
                    if matches!(anim.current_move, Move::F | Move::Fp | Move::F2)
                        || matches!(anim.current_move, Move::B | Move::Bp | Move::B2)
                    {
                        if let Some((_face_start, angle)) = anim_face_rot {
                            let f_face_start = 16;
                            let center_grid_base = get_grid_coords(f_face_start);
                            let center_grid =
                                Pos2::new(center_grid_base.x + 0.5, center_grid_base.y + 0.5);
                            let center_screen = to_screen(center_grid);

                            // B操作の場合は回転方向を反転（前面から見ると反時計回り）
                            let final_angle =
                                if matches!(anim.current_move, Move::B | Move::Bp | Move::B2) {
                                    -angle
                                } else {
                                    angle
                                };

                            let current_angle = final_angle * p;
                            pos = rotate_point(start_screen, center_screen, current_angle);

                            // 傾き（矢印向き）の補正: 前回の向きから今の向きへアニメーション
                            let orientation_change_deg = -(orientation_delta as f32 * 90.0);
                            rot = current_angle + orientation_change_deg;

                            // 放射状の膨らみ（B と Bp の軌道を一致させる）
                            let bulge = ANIMATION_BULGE_RATIO * grid_size;
                            let mid_p = (p * std::f32::consts::PI).sin();
                            let radial_vec = (pos - center_screen).normalized();
                            pos += radial_vec * bulge * mid_p;
                            shadow =
                                Vec2::new(ANIMATION_SHADOW_OFFSET, ANIMATION_SHADOW_OFFSET) * mid_p;
                        }
                    } else {
                        // その他の移動 (R, L, U, D または長距離ジャンプ)
                        let mid_p = (p * std::f32::consts::PI).sin();
                        let face_idx = i / STICKERS_PER_FACE;
                        let is_rl_move = matches!(
                            anim.current_move,
                            Move::R | Move::Rp | Move::R2 | Move::L | Move::Lp | Move::L2
                        );
                        // R/L操作での特別な制御
                        let is_u_face = is_rl_move && face_idx == Face::Up as usize;
                        let is_fd_vertical = is_rl_move
                            && (face_idx == Face::Front as usize
                                || face_idx == Face::Down as usize)
                            && (start_screen.x - end_screen.x).abs() < 1.0;

                        // 自転の計算
                        if is_u_face {
                            // 白色（U面）は時計回りに「転がる」ように見せる
                            // 物理的deltaに従い、必ず + 方向（時計回り）に回るようにする
                            // (p-1)を掛けることで 0からphysical_deltaへ +方向にアニメーション
                            rot = -physical_delta_deg * (1.0 - p);
                        } else if is_fd_vertical {
                            // 赤・黄の垂直移動は自転なし（スライドのみ）
                            // ユーザー要望により物理回転(delta)も無視して 0 固定にする
                            rot = 0.0;
                        } else {
                            // その他は物理的な向きの変化のみを補正してジャンプを防ぐ
                            rot = -physical_delta_deg * (1.0 - p);
                        }

                        // 座標をソートして基準法線を決める (RとRpで逆にならないように)
                        let (p1, p2) = if start_screen.x < end_screen.x
                            || (start_screen.x == end_screen.x && start_screen.y < end_screen.y)
                        {
                            (start_screen, end_screen)
                        } else {
                            (end_screen, start_screen)
                        };
                        let sorted_vec = p2 - p1;
                        let ortho_fixed = Vec2::new(-sorted_vec.y, sorted_vec.x).normalized();

                        if dist < 3.0 {
                            // 隣接面移動
                            let mut bulge_val = ANIMATION_BULGE_RATIO * grid_size;
                            if is_fd_vertical {
                                // 赤・黄の垂直移動は膨らみなし（直線移動）
                                bulge_val = 0.0;
                            }
                            pos += ortho_fixed * bulge_val * mid_p;
                            size_scale = 1.0 + 0.1 * mid_p;
                            shadow =
                                Vec2::new(ANIMATION_SHADOW_OFFSET, ANIMATION_SHADOW_OFFSET) * mid_p;
                        } else {
                            // 非隣接面（ジャンプ）
                            let bulge_val = ANIMATION_JUMP_BULGE_RATIO * grid_size;
                            // 展開図の端を跨ぐ場合の調整
                            let mut adjusted_ortho = ortho_fixed;
                            if adjusted_ortho.y.abs() < 0.1 {
                                adjusted_ortho.y = -adjusted_ortho.y.abs();
                            }
                            pos += adjusted_ortho * bulge_val * mid_p;
                            alpha = 1.0 - 0.5 * mid_p;
                            size_scale = 1.0 - 0.3 * mid_p;
                            shadow = Vec2::new(
                                ANIMATION_JUMP_SHADOW_OFFSET,
                                ANIMATION_JUMP_SHADOW_OFFSET,
                            ) * mid_p;
                        }
                    }
                    (pos, rot, size_scale, alpha, shadow)
                };

                // メインのステッカーを描画
                let (p_pos, p_rot, p_scale, p_alpha, p_shadow) = calc_state(progress);

                // モーショントレイルを描画
                for ghost_t in [0.05, 0.1] {
                    let t = (progress - ghost_t).max(0.0);
                    if t > 0.0 {
                        let (g_pos, g_rot, g_scale, g_alpha, _) = calc_state(t);
                        draw_sticker(
                            painter,
                            g_pos,
                            sticker_size * g_scale * (1.0 - ghost_t * 2.0),
                            sticker,
                            g_rot,
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
                    p_rot,
                    p_alpha,
                    p_shadow,
                );
                drawn = true;
            }
        }

        if !drawn {
            draw_sticker(
                painter,
                screen_pos,
                sticker_size,
                sticker,
                rotation,
                1.0,
                Vec2::ZERO,
            );
        }
    }

    // 編集中の面をハイライト表示
    if let Some(face_idx) = highlight_face_index {
        let start_idx = face_idx * STICKERS_PER_FACE;
        let face_grid_rect = get_face_grid_rect(start_idx);

        // 面の左上セルと右下セルの中心を取得
        // face_grid_rect.maxは排他的（範囲の外側）なので、実際の最後のセルはmax-1
        let top_left_cell_center = to_screen(face_grid_rect.min);
        let bottom_right_cell_center = to_screen(Pos2::new(
            face_grid_rect.max.x - 1.0,
            face_grid_rect.max.y - 1.0,
        ));

        // セルの中心から面全体の境界を計算
        // 左上はセルの中心から-grid_size/2、右下はセルの中心から+grid_size/2
        let top_left = top_left_cell_center - Vec2::splat(grid_size * 0.5);
        let bottom_right = bottom_right_cell_center + Vec2::splat(grid_size * 0.5);

        // 少し余白を持たせる
        let padding = grid_size * HIGHLIGHT_PADDING_RATIO;
        let highlight_rect = Rect::from_min_max(
            top_left - Vec2::splat(padding),
            bottom_right + Vec2::splat(padding),
        );

        // 太いオレンジの枠線で囲む
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
            Pos2::new(rect.left() + 10.0, rect.bottom() - 30.0),
            egui::Align2::LEFT_BOTTOM,
            text,
            egui::FontId::proportional(16.0),
            Color32::BLACK,
        );
    }
}

/// 面の回転による向きの変更量を取得
fn get_face_orientation_delta(mv: Move) -> u8 {
    match mv {
        Move::R | Move::L | Move::F | Move::B | Move::U | Move::D => 1,
        Move::Rp | Move::Lp | Move::Fp | Move::Bp | Move::Up | Move::Dp => 3,
        Move::U2 | Move::D2 | Move::L2 | Move::R2 | Move::F2 | Move::B2 => 2,
    }
}

/// 移動するステッカーの向きの変更量を取得
fn get_moving_sticker_orientation_delta(mv: Move, src_idx: usize) -> u8 {
    match mv {
        Move::R => match src_idx {
            1 | 3 | 22 | 20 => 2,
            _ => 0,
        },
        Move::Rp => match src_idx {
            22 | 20 | 5 | 7 => 2,
            _ => 0,
        },
        Move::R2 => match src_idx {
            22 | 20 | 17 | 19 => 2,
            _ => 0,
        },
        Move::L => match src_idx {
            23 | 21 | 4 | 6 => 2,
            _ => 0,
        },
        Move::Lp => match src_idx {
            23 | 21 | 0 | 2 => 2,
            _ => 0,
        },
        Move::L2 => match src_idx {
            23 | 21 | 16 | 18 => 2,
            _ => 0,
        },
        Move::F | Move::Fp => 1,
        Move::F2 => 2,
        Move::B | Move::Bp => 3,
        Move::B2 => 2,
        _ => 0,
    }
}

/// インデックスに対応する面全体のグリッド領域を取得
fn get_face_grid_rect(index: usize) -> Rect {
    let (min_col, min_row) = match index {
        i if i == Face::Up.start_index() => (2.0, 0.0),   // U
        i if i == Face::Down.start_index() => (2.0, 4.0), // D
        i if i == Face::Left.start_index() => (0.0, 2.0), // L
        i if i == Face::Right.start_index() => (4.0, 2.0), // R
        i if i == Face::Front.start_index() => (2.0, 2.0), // F
        i if i == Face::Back.start_index() => (6.0, 2.0), // B
        _ => (0.0, 0.0),
    };
    // 2x2なのでサイズは2.0x2.0
    Rect::from_min_size(Pos2::new(min_col, min_row), Vec2::new(2.0, 2.0))
}

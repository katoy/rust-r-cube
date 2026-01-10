use crate::cube::{Color, Cube, Move, Sticker};
use crate::gui::app::AnimationState;
use egui::{Color32, Painter, Pos2, Rect, Stroke, Vec2};

/// ステッカーの色をegui Color32に変換
fn color_to_color32(color: Color) -> Color32 {
    match color {
        Color::White => Color32::from_rgb(255, 255, 255),
        Color::Yellow => Color32::from_rgb(255, 255, 0),
        Color::Green => Color32::from_rgb(0, 200, 0),
        Color::Blue => Color32::from_rgb(0, 100, 255),
        Color::Red => Color32::from_rgb(255, 50, 50),
        Color::Orange => Color32::from_rgb(255, 165, 0),
        Color::Gray => Color32::from_rgb(180, 180, 180), // 未設定用グレー
    }
}

/// ステッカーを描画
fn draw_sticker(
    painter: &Painter,
    center: Pos2,
    size: f32,
    sticker: Sticker,
    rotation_offset_deg: f32,
    alpha: f32,
) {
    // ステッカーの背景を描画
    let rect = Rect::from_center_size(center, Vec2::splat(size * 0.95));

    // 回転を適用した矩形を描画するために、頂点を計算して回転させる
    if rotation_offset_deg.abs() > 0.1 {
        let angle = rotation_offset_deg.to_radians();
        let cos = angle.cos();
        let sin = angle.sin();

        let half = size * 0.95 / 2.0;
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
            color_to_color32(sticker.color).linear_multiply(alpha),
            Stroke::new(2.0, Color32::BLACK.linear_multiply(alpha)),
        ));
    } else {
        painter.rect_filled(
            rect,
            3.0,
            color_to_color32(sticker.color).linear_multiply(alpha),
        );
        painter.rect_stroke(
            rect,
            3.0,
            Stroke::new(2.0, Color32::BLACK.linear_multiply(alpha)),
        );
    }

    // 矢印を描画（向きを示す）
    let arrow_rotation = (sticker.orientation as f32 * 90.0 + rotation_offset_deg).to_radians();
    draw_arrow(painter, center, size * 0.3, arrow_rotation, alpha);
}

/// 矢印を描画
fn draw_arrow(painter: &Painter, center: Pos2, length: f32, rotation: f32, alpha: f32) {
    let cos = rotation.cos();
    let sin = rotation.sin();

    // 矢印の先端
    let tip = Pos2::new(center.x + length * sin, center.y - length * cos);

    // 矢印の根元
    let base = Pos2::new(center.x - length * 0.3 * sin, center.y + length * 0.3 * cos);

    // 矢印の羽
    let wing_length = length * 0.4;
    let wing_angle = 30.0_f32.to_radians();

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
    let stroke = Stroke::new(2.0, Color32::from_black_alpha(180).linear_multiply(alpha));
    painter.line_segment([base, tip], stroke);
    painter.line_segment([tip, left_wing], stroke);
    painter.line_segment([tip, right_wing], stroke);
}

/// インデックスに対応するグリッド座標 (col, row) を取得
fn get_grid_coords(index: usize) -> Pos2 {
    let (col, row) = match index {
        0..=3 => (2.0 + (index % 2) as f32, 0.0 + (index / 2) as f32), // U
        4..=7 => (
            2.0 + ((index - 4) % 2) as f32,
            4.0 + ((index - 4) / 2) as f32,
        ), // D
        8..=11 => (
            0.0 + ((index - 8) % 2) as f32,
            2.0 + ((index - 8) / 2) as f32,
        ), // L
        12..=15 => (
            4.0 + ((index - 12) % 2) as f32,
            2.0 + ((index - 12) / 2) as f32,
        ), // R
        16..=19 => (
            2.0 + ((index - 16) % 2) as f32,
            2.0 + ((index - 16) / 2) as f32,
        ), // F
        20..=23 => (
            6.0 + ((index - 20) % 2) as f32,
            2.0 + ((index - 20) / 2) as f32,
        ), // B
        _ => (0.0, 0.0),
    };
    Pos2::new(col, row)
}

/// アニメーション情報の型エイリアス: (移動マッピング, 回転面情報)
type AnimationInfo = (Vec<(usize, usize)>, Option<(usize, f32)>);

/// アニメーション情報：移動マッピングと回転面情報
fn get_animation_info(mv: Move) -> AnimationInfo {
    let mapping = match mv {
        Move::U => vec![
            (16, 8),
            (17, 9),
            (8, 20),
            (9, 21),
            (20, 12),
            (21, 13),
            (12, 16),
            (13, 17),
        ],
        Move::Up => vec![
            (8, 16),
            (9, 17),
            (20, 8),
            (21, 9),
            (12, 20),
            (13, 21),
            (16, 12),
            (17, 13),
        ],
        Move::D => vec![
            (18, 14),
            (19, 15),
            (14, 22),
            (15, 23),
            (22, 10),
            (23, 11),
            (10, 18),
            (11, 19),
        ],
        Move::Dp => vec![
            (14, 18),
            (15, 19),
            (22, 14),
            (23, 15),
            (10, 22),
            (11, 23),
            (18, 10),
            (19, 11),
        ],
        Move::R => vec![
            (17, 1),
            (19, 3),
            (1, 22),
            (3, 20),
            (22, 5),
            (20, 7),
            (5, 17),
            (7, 19),
        ],
        Move::Rp => vec![
            (1, 17),
            (3, 19),
            (22, 1),
            (20, 3),
            (5, 22),
            (7, 20),
            (17, 5),
            (19, 7),
        ],
        Move::L => vec![
            (21, 0),
            (23, 2),
            (0, 16),
            (2, 18),
            (16, 4),
            (18, 6),
            (4, 21),
            (6, 23),
        ],
        Move::Lp => vec![
            (0, 21),
            (2, 23),
            (16, 0),
            (18, 2),
            (4, 16),
            (6, 18),
            (21, 4),
            (23, 6),
        ],
        Move::F => vec![
            (11, 2),
            (9, 3),
            (2, 12),
            (3, 14),
            (12, 4),
            (14, 5),
            (4, 11),
            (5, 9),
        ],
        Move::Fp => vec![
            (2, 11),
            (3, 9),
            (12, 2),
            (14, 3),
            (4, 12),
            (5, 14),
            (11, 4),
            (9, 5),
        ],
        Move::B => vec![
            (13, 0),
            (15, 1),
            (0, 10),
            (1, 8),
            (10, 6),
            (8, 7),
            (6, 13),
            (7, 15),
        ],
        Move::Bp => vec![
            (0, 13),
            (1, 15),
            (10, 0),
            (8, 1),
            (6, 10),
            (7, 8),
            (13, 6),
            (15, 7),
        ],
    };

    let face_rotation = match mv {
        Move::U => Some((0, 90.0)),
        Move::Up => Some((0, -90.0)),
        Move::D => Some((4, 90.0)),
        Move::Dp => Some((4, -90.0)),
        Move::L => Some((8, 90.0)),
        Move::Lp => Some((8, -90.0)),
        Move::R => Some((12, 90.0)),
        Move::Rp => Some((12, -90.0)),
        Move::F => Some((16, 90.0)),
        Move::Fp => Some((16, -90.0)),
        Move::B => Some((20, 90.0)),
        Move::Bp => Some((20, -90.0)),
    };

    (mapping, face_rotation)
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

/// キューブを展開図として描画
pub fn draw_cube(
    ui: &mut egui::Ui,
    rect: Rect,
    cube: &Cube,
    animation: Option<&AnimationState>,
    highlight_face_index: Option<usize>,
) {
    let painter = ui.painter();

    let grid_cols = 8.0;
    let grid_rows = 6.0;

    // グリッドサイズ計算
    let grid_size = (rect.width() / grid_cols).min(rect.height() / grid_rows) * 0.95;
    let sticker_size = grid_size * 0.85;

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

    // 全ステッカーを描画
    for i in 0..24 {
        let mut sticker = cube.get_sticker(i);
        let grid_pos = get_grid_coords(i);
        let mut rotation = 0.0;
        let mut screen_pos = to_screen(grid_pos);

        let mut drawn = false;

        if let Some(anim) = animation {
            // アニメーション中の操作に応じてorientationを調整

            // 1. 回転する面のステッカー: 最終的なorientationを設定
            if let Some((face_start, _angle)) = anim_face_rot {
                if i >= face_start && i < face_start + 4 {
                    let orientation_delta = match anim.current_move {
                        Move::R | Move::L | Move::F | Move::B => 1, // 時計回り: +1
                        Move::Rp | Move::Lp | Move::Fp | Move::Bp => 3, // 反時計回り: +3
                        Move::U | Move::D => 1,                     // Up/Down: +1
                        Move::Up | Move::Dp => 3,                   // Up'/Down': +3
                    };
                    sticker.orientation = (sticker.orientation + orientation_delta) % 4;
                }
            }

            // 2. 移動するステッカーのorientation調整
            if let Some((_, _target_idx)) = anim_mapping.iter().find(|(src, _)| *src == i) {
                let orientation_delta = match anim.current_move {
                    Move::R | Move::Rp => {
                        // U面の右列 (1, 3) → Back面へ: +2
                        // B面の右列 (22, 20) → Down面へ: +2
                        if i == 1 || i == 3 || i == 22 || i == 20 {
                            2
                        } else {
                            0
                        }
                    }
                    Move::L | Move::Lp => {
                        // U面の左列 (0, 2) → Front面へ: +2
                        // F面の左列 (16, 18) → Down面へ: 変更なし
                        // D面の左列 (4, 6) → Back面へ: 変更なし
                        // B面の左列 (21, 23) → Up面へ: +2
                        if i == 0 || i == 2 || i == 21 || i == 23 {
                            2
                        } else {
                            0
                        }
                    }
                    Move::F | Move::Fp => {
                        // U面の下列 (2, 3) → Left面へ: +3
                        // L面の右列 (9, 11) → Down面へ: +1
                        // D面の上列 (4, 5) → Right面へ: +3
                        // R面の左列 (12, 14) → Up面へ: +1
                        match i {
                            2 | 3 => 3,
                            9 | 11 => 1,
                            4 | 5 => 3,
                            12 | 14 => 1,
                            _ => 0,
                        }
                    }
                    Move::B | Move::Bp => {
                        // U面の上列 (0, 1) → Right面へ: +1
                        // R面の右列 (13, 15) → Down面へ: +3
                        // D面の下列 (6, 7) → Left面へ: +1
                        // L面の左列 (8, 10) → Up面へ: +3
                        match i {
                            0 | 1 => 1,
                            13 | 15 => 3,
                            6 | 7 => 1,
                            8 | 10 => 3,
                            _ => 0,
                        }
                    }
                    _ => 0,
                };

                if orientation_delta > 0 {
                    sticker.orientation = (sticker.orientation + orientation_delta) % 4;
                }
            }

            // 面回転の処理
            if let Some((face_start, angle)) = anim_face_rot {
                if i >= face_start && i < face_start + 4 {
                    // 面の中心
                    let center_grid_idx = face_start;
                    let center_grid_base = get_grid_coords(center_grid_idx);
                    // 2x2の中心は (col+0.5, row+0.5)
                    let center_grid = Pos2::new(center_grid_base.x + 0.5, center_grid_base.y + 0.5);
                    let center_screen = to_screen(center_grid);

                    let current_angle = angle * anim.eased_progress();
                    screen_pos = rotate_point(screen_pos, center_screen, current_angle);

                    // rotationを計算
                    // orientation変化分の相殺は一時的に無効化
                    // orientationの変化分を差し引いて、矢印が物理的な回転と一致するようにする
                    let orientation_delta = match anim.current_move {
                        Move::R | Move::L | Move::F | Move::B => 1, // +1 = +90度
                        Move::Rp | Move::Lp | Move::Fp | Move::Bp => 3, // +3 = +270度 = -90度
                        Move::U | Move::D => 1,
                        Move::Up | Move::Dp => 3,
                    };
                    // orientation変化分を相殺：+1なら-90度、+3なら-270度
                    let orientation_change_deg = -(orientation_delta as f32 * 90.0);
                    rotation = current_angle + orientation_change_deg;
                }
            }

            // 移動の処理
            if let Some((_, target_idx)) = anim_mapping.iter().find(|(src, _)| *src == i) {
                let target_grid_pos = get_grid_coords(*target_idx);

                // ワープ回避：距離が遠すぎる場合はラップアラウンドアニメーション
                // 閾値を3.0に設定（F回転の距離2.23は通常移動。裏側への移動など距離3.0以上のみワープ）
                if grid_pos.distance(target_grid_pos) < 3.0 {
                    let src_screen = screen_pos; // 回転なしの場合の初期位置
                    let dst_screen = to_screen(target_grid_pos);

                    screen_pos = Pos2::new(
                        src_screen.x + (dst_screen.x - src_screen.x) * anim.eased_progress(),
                        src_screen.y + (dst_screen.y - src_screen.y) * anim.eased_progress(),
                    );
                } else {
                    // ラップアラウンド
                    let diff = target_grid_pos - grid_pos;

                    // 最適なラップ方向を探す
                    // 展開図サイズは 横8, 縦6
                    // 対角方向のショートカットも含めて探索することで、より自然な隣接点を見つける
                    let candidates = [
                        Vec2::new(8.0, 0.0),
                        Vec2::new(-8.0, 0.0),
                        Vec2::new(0.0, 6.0),
                        Vec2::new(0.0, -6.0),
                        Vec2::new(4.0, 3.0),
                        Vec2::new(-4.0, -3.0),
                        Vec2::new(4.0, -3.0),
                        Vec2::new(-4.0, 3.0),
                    ];

                    let mut best_wrap = Vec2::ZERO;
                    let mut min_len = f32::MAX;

                    for wrap in candidates {
                        let len = (diff - wrap).length();
                        if len < min_len {
                            min_len = len;
                            best_wrap = wrap;
                        }
                    }

                    let wrap_vec_grid = best_wrap;

                    let progress = anim.eased_progress();

                    // 1. 去るアニメーション (src -> dst_wrapped)
                    // dst_wrapped = dst - wrap_vec
                    let dst_wrapped_grid = target_grid_pos - wrap_vec_grid;
                    let dst_wrapped_screen = to_screen(dst_wrapped_grid);
                    let src_screen = screen_pos; // 現在位置

                    let pos_out = src_screen + (dst_wrapped_screen - src_screen) * progress;

                    // クリッピング領域（移動元の面内のみ表示）
                    let clip_grid_rect = get_face_grid_rect(i);
                    // グリッド座標(中心)から矩形(左上〜右下)への変換補正
                    let clip_rect_src = Rect::from_min_max(
                        to_screen(clip_grid_rect.min) - Vec2::splat(grid_size * 0.5),
                        to_screen(clip_grid_rect.max) - Vec2::splat(grid_size * 0.5),
                    );

                    let clipped_painter_src = painter.with_clip_rect(clip_rect_src);
                    draw_sticker(
                        &clipped_painter_src,
                        pos_out,
                        sticker_size,
                        sticker,
                        rotation,
                        1.0,
                    );

                    // 2. 来るアニメーション (src_wrapped -> dst)
                    // src_wrapped = src + wrap_vec
                    let src_wrapped_grid = grid_pos + wrap_vec_grid;
                    let src_wrapped_screen = to_screen(src_wrapped_grid);
                    let dst_screen = to_screen(target_grid_pos);

                    let pos_in = src_wrapped_screen + (dst_screen - src_wrapped_screen) * progress;

                    // クリッピング領域（移動先の面内のみ表示）
                    let clip_grid_rect_dst = get_face_grid_rect(*target_idx);
                    let clip_rect_dst = Rect::from_min_max(
                        to_screen(clip_grid_rect_dst.min) - Vec2::splat(grid_size * 0.5),
                        to_screen(clip_grid_rect_dst.max) - Vec2::splat(grid_size * 0.5),
                    );

                    let clipped_painter_dst = painter.with_clip_rect(clip_rect_dst);
                    draw_sticker(
                        &clipped_painter_dst,
                        pos_in,
                        sticker_size,
                        sticker,
                        rotation,
                        1.0,
                    );

                    drawn = true;
                }
            }
        }

        if !drawn {
            draw_sticker(painter, screen_pos, sticker_size, sticker, rotation, 1.0);
        }
    }

    // 編集中の面をハイライト表示
    if let Some(face_idx) = highlight_face_index {
        let start_idx = face_idx * 4;
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
        let padding = grid_size * 0.05;
        let highlight_rect = Rect::from_min_max(
            top_left - Vec2::splat(padding),
            bottom_right + Vec2::splat(padding),
        );

        // 太いオレンジの枠線で囲む
        painter.rect_stroke(
            highlight_rect,
            5.0,
            Stroke::new(4.0, Color32::from_rgb(255, 140, 0)),
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

/// インデックスに対応する面全体のグリッド領域を取得
fn get_face_grid_rect(index: usize) -> Rect {
    let (min_col, min_row) = match index {
        0..=3 => (2.0, 0.0),   // U
        4..=7 => (2.0, 4.0),   // D
        8..=11 => (0.0, 2.0),  // L
        12..=15 => (4.0, 2.0), // R
        16..=19 => (2.0, 2.0), // F
        20..=23 => (6.0, 2.0), // B
        _ => (0.0, 0.0),
    };
    // 2x2なのでサイズは2.0x2.0
    Rect::from_min_size(Pos2::new(min_col, min_row), Vec2::new(2.0, 2.0))
}

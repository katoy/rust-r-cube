use egui::Color32;

/// ステッカーの色の定義
pub const COLOR_WHITE: Color32 = Color32::from_rgb(255, 255, 255);
pub const COLOR_YELLOW: Color32 = Color32::from_rgb(255, 255, 0);
pub const COLOR_GREEN: Color32 = Color32::from_rgb(0, 200, 0);
pub const COLOR_BLUE: Color32 = Color32::from_rgb(0, 100, 255);
pub const COLOR_RED: Color32 = Color32::from_rgb(255, 50, 50);
pub const COLOR_ORANGE: Color32 = Color32::from_rgb(255, 165, 0);
pub const COLOR_GRAY: Color32 = Color32::from_rgb(180, 180, 180);

/// 描画パラメータ
pub const STICKER_SIZE_RATIO: f32 = 0.95; // グリッドに対するステッカーの大きさ
pub const STICKER_ROUNDING: f32 = 3.0;
pub const STICKER_STROKE_WIDTH: f32 = 2.0;

pub const ARROW_LENGTH_RATIO: f32 = 0.3; // ステッカーサイズに対する矢印の長さ
pub const ARROW_WIDTH: f32 = 2.0;
pub const ARROW_ALPHA: u8 = 180;
pub const ARROW_WING_RATIO: f32 = 0.4;
pub const ARROW_WING_ANGLE_DEG: f32 = 30.0;
pub const ARROW_BASE_RATIO: f32 = 0.3;

/// 2Dグリッド設定
pub const GRID_COLS: f32 = 8.0;
pub const GRID_ROWS: f32 = 6.0;
pub const GRID_PADDING_RATIO: f32 = 0.95;
pub const STICKER_SIZE_IN_GRID: f32 = 0.85;

/// アニメーション定数
pub const ANIMATION_FACE_HIGHLIGHT_ALPHA: u8 = 30;
pub const ANIMATION_BULGE_RATIO: f32 = 0.2; // 近接面移動時の膨らみ
pub const ANIMATION_JUMP_BULGE_RATIO: f32 = 1.5; // ジャンプ時の膨らみ
pub const ANIMATION_SHADOW_OFFSET: f32 = 5.0;
pub const ANIMATION_JUMP_SHADOW_OFFSET: f32 = 10.0;
pub const ANIMATION_TRAIL_ALPHA_FACTOR: f32 = 0.3;

/// ハイライト設定
pub const HIGHLIGHT_STROKE_WIDTH: f32 = 4.0;
pub const HIGHLIGHT_COLOR: Color32 = Color32::from_rgb(255, 140, 0);
pub const HIGHLIGHT_PADDING_RATIO: f32 = 0.05;

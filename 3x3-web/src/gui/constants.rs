use egui::Color32;
use std::f32::consts::PI;

/// ステッカーの色の定義
pub const COLOR_WHITE: Color32 = Color32::from_rgb(255, 255, 255);
pub const COLOR_YELLOW: Color32 = Color32::from_rgb(255, 255, 0);
pub const COLOR_GREEN: Color32 = Color32::from_rgb(0, 200, 0);
pub const COLOR_BLUE: Color32 = Color32::from_rgb(0, 100, 255);
pub const COLOR_RED: Color32 = Color32::from_rgb(255, 50, 50);
pub const COLOR_ORANGE: Color32 = Color32::from_rgb(255, 165, 0);
pub const COLOR_GRAY: Color32 = Color32::from_rgb(180, 180, 180);

/// 2D描画パラメータ
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
pub const GRID_COLS: f32 = 12.0;
pub const GRID_ROWS: f32 = 9.0;
pub const GRID_PADDING_RATIO: f32 = 0.95;
pub const STICKER_SIZE_IN_GRID: f32 = 0.85;

/// 3D描画パラメータ
pub const VIEW3D_DEFAULT_YAW: f32 = PI / 4.0;
pub const VIEW3D_DEFAULT_PITCH: f32 = PI / 6.0;
pub const VIEW3D_DEFAULT_SCALE: f32 = 1.0;

pub const VIEW3D_CAMERA_DISTANCE: f32 = 5.0;
pub const VIEW3D_PROJECTION_SCALE: f32 = 0.3;
pub const VIEW3D_STICKER_SIZE: f32 = 0.45;
pub const VIEW3D_BACKFACE_CULLING_THRESHOLD: f32 = 0.2;
pub const VIEW3D_ARROW_VEC_SCALE: f32 = 0.6;
pub const VIEW3D_ARROW_WIDTH: f32 = 6.0;
pub const VIEW3D_ARROW_HEAD_RATIO: f32 = 0.6;
pub const VIEW3D_PITCH_LIMIT_MARGIN: f32 = 0.1;

pub const MOUSE_SENSITIVITY: f32 = 0.01;
pub const ZOOM_FACTOR: f32 = 1.1;

/// UIレイアウト定数
pub const UI_SPACING_LARGE: f32 = 10.0;
pub const UI_SPACING_SMALL: f32 = 5.0;
pub const UI_HEADING_FONT_SIZE: f32 = 18.0;
pub const UI_BODY_FONT_SIZE: f32 = 14.0;
pub const UI_HELP_TEXT_SIZE: f32 = 12.0;
pub const UI_SOLVE_STEP_FONT_SIZE: f32 = 16.0;
pub const UI_SIDE_PANEL_WIDTH: f32 = 250.0;

pub const INPUT_PALETTE_BUTTON_SIZE: [f32; 2] = [35.0, 30.0];
pub const INPUT_STICKER_BUTTON_SIZE: [f32; 2] = [50.0, 50.0];
pub const INPUT_SELECTED_STROKE_WIDTH: f32 = 3.0;
pub const INPUT_UNSELECTED_STROKE_WIDTH: f32 = 1.0;

/// アニメーション定数
pub const ANIMATION_FACE_HIGHLIGHT_ALPHA: u8 = 30;
pub const ANIMATION_BULGE_RATIO: f32 = 0.2; // 近接面移動時の膨らみ
pub const ANIMATION_JUMP_BULGE_RATIO: f32 = 1.5; // ジャンプ時の膨らみ
pub const ANIMATION_SHADOW_OFFSET: f32 = 5.0;
pub const ANIMATION_JUMP_SHADOW_OFFSET: f32 = 10.0;
pub const ANIMATION_TRAIL_ALPHA_FACTOR: f32 = 0.3;
pub const ANIMATION_BULGE_MID_FACTOR: f32 = 0.1; // 1.0 + 0.1 * mid_p
pub const ANIMATION_JUMP_SIZE_FACTOR: f32 = 0.3; // 1.0 - 0.3 * mid_p
pub const ANIMATION_JUMP_ALPHA_FACTOR: f32 = 0.5; // 1.0 - 0.5 * mid_p
pub const ANIMATION_SPLIT_DURATION_FACTOR: f32 = 0.5;

/// ハイライト設定
pub const HIGHLIGHT_STROKE_WIDTH: f32 = 4.0;
pub const HIGHLIGHT_COLOR: Color32 = Color32::from_rgb(255, 140, 0);
pub const HIGHLIGHT_PADDING_RATIO: f32 = 0.05;
pub const HIGHLIGHT_EXPAND: f32 = 2.0; // highlight_rect.expand(2.0)
pub const HIGHLIGHT_ROUNDING: f32 = 5.0;

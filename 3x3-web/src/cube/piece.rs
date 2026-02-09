use crate::cube::{Color, Face, Sticker};
use glam::{Mat4, Vec3};

/// ピース（キューブレット）のタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceType {
    Center,
    Edge,
    Corner,
}

/// ピース上の個々のステッカーの色と初期法線
#[derive(Debug, Clone, Copy)]
pub struct CubieSticker {
    pub initial_normal: Vec3,
    pub color: Color,
}

/// ルービックキューブを構成する26個のピース（センター中の1つを除く）
#[derive(Debug, Clone)]
pub struct Cubie {
    pub piece_type: PieceType,
    pub initial_pos: Vec3,
    pub current_pos: Vec3,
    pub current_rot: Mat4,
    pub stickers: Vec<CubieSticker>,
}

impl Cubie {
    pub fn new(pos: Vec3, stickers: Vec<CubieSticker>) -> Self {
        let piece_type = match stickers.len() {
            1 => PieceType::Center,
            2 => PieceType::Edge,
            3 => PieceType::Corner,
            _ => panic!("Invalid number of stickers for a piece: {}", stickers.len()),
        };

        Self {
            piece_type,
            initial_pos: pos,
            current_pos: pos,
            current_rot: Mat4::IDENTITY,
            stickers,
        }
    }

    /// ピースを特定の軸を中心に指定された角度だけ回転させます。
    pub fn rotate(&mut self, axis: Vec3, angle_rad: f32) {
        let rot_mat = Mat4::from_axis_angle(axis, angle_rad);

        // 位置の更新 (浮動小数点の誤差を丸める)
        let new_pos = rot_mat.transform_point3(self.current_pos);
        self.current_pos = Vec3::new(new_pos.x.round(), new_pos.y.round(), new_pos.z.round());

        // 回転状態の更新
        self.current_rot = rot_mat * self.current_rot;
    }

    /// このピースが持つステッカーを、現在の回転状態に基づいて Facelet 配列（54枚）に投影します。
    pub fn project_to_stickers(&self, target: &mut [Sticker; 54]) {
        for cubie_sticker in &self.stickers {
            // 現在の法線を計算
            let current_normal = self
                .current_rot
                .transform_vector3(cubie_sticker.initial_normal);
            let n = Vec3::new(
                current_normal.x.round(),
                current_normal.y.round(),
                current_normal.z.round(),
            );

            // どの面に属するか特定
            let face = if n.y > 0.5 {
                Face::Up
            } else if n.y < -0.5 {
                Face::Down
            } else if n.x < -0.5 {
                Face::Left
            } else if n.x > 0.5 {
                Face::Right
            } else if n.z > 0.5 {
                Face::Front
            } else if n.z < -0.5 {
                Face::Back
            } else {
                continue; // 内部に向いている面（通常はないはず）
            };

            // その面内でのインデックスを計算
            let face_idx = face_to_local_index(face, self.current_pos);
            let abs_idx = face.start_index() + face_idx;

            // ステッカーの「向き (orientation)」を計算
            // Piece の rotation 行列から、どの程度自転しているかを導出
            let ori = self.calculate_orientation(cubie_sticker.initial_normal, n);

            target[abs_idx] = Sticker {
                color: cubie_sticker.color,
                orientation: ori,
            };
        }
    }

    /// ステッカーの piece 内での向きを 0-3 で計算します。
    pub fn calculate_orientation(&self, initial_normal: Vec3, current_normal: Vec3) -> u8 {
        calculate_orientation_with_rot(initial_normal, current_normal, self.current_rot)
    }

    /// 与えられた色（順序不同）をすべて持っているピースかどうか判定します。
    pub fn matches_colors(&self, target_colors: &[Color]) -> bool {
        if self.stickers.len() != target_colors.len() {
            return false;
        }
        let mut available_colors: Vec<Color> = self.stickers.iter().map(|s| s.color).collect();
        for &tc in target_colors {
            if let Some(pos) = available_colors.iter().position(|&c| c == tc) {
                available_colors.remove(pos);
            } else {
                return false;
            }
        }
        true
    }
}

/// 指定された回転行列に基づいて向きを計算します（アニメーション用）。
pub fn calculate_orientation_with_rot(
    _initial_normal: Vec3,
    current_normal: Vec3,
    rot: Mat4,
) -> u8 {
    let current_v_base = get_face_up_axis(current_normal);

    // ピースの初期「上」方向（Whiteステッカーなら -Z等）が、今の回転行列でどこを向いているか
    // すべてのステッカーについて、初期状態 (rot=IDENTITY) での initial_normal 方向の面において
    // get_face_up_axis(initial_normal) が「上 (orientation=0)」を指していると仮定する。
    // しかし、Cubie には複数のステッカーがあるので、各ステッカーごとに初期の上方向が異なる。

    // 正しいアプローチ: 各ステッカーの initial_normal に対応する initial_v_base を求める。
    let initial_v_base = get_face_up_axis(_initial_normal);
    let actual_v = rot.transform_vector3(initial_v_base);

    // actual_v と、現在の面の基準上方向 current_v_base のズレを計算する。
    let dot = actual_v.dot(current_v_base);
    if dot > 0.9 {
        0
    } else if dot < -0.9 {
        2
    } else {
        let cross = current_v_base.cross(actual_v);
        if cross.dot(current_normal) > 0.9 {
            3 // CCW
        } else {
            1 // CW
        }
    }
}

/// 面と空間座標から、面内の 0-8 のインデックスを返します。
pub fn face_to_local_index(face: Face, pos: Vec3) -> usize {
    match face {
        Face::Up => {
            let col = (pos.x + 1.0) as usize;
            let row = (pos.z + 1.0) as usize;
            row * 3 + col
        }
        Face::Down => {
            let col = (pos.x + 1.0) as usize;
            let row = (1.0 - pos.z) as usize;
            row * 3 + col
        }
        Face::Left => {
            let col = (pos.z + 1.0) as usize;
            let row = (1.0 - pos.y) as usize;
            row * 3 + col
        }
        Face::Right => {
            let col = (1.0 - pos.z) as usize;
            let row = (1.0 - pos.y) as usize;
            row * 3 + col
        }
        Face::Front => {
            let col = (pos.x + 1.0) as usize;
            let row = (1.0 - pos.y) as usize;
            row * 3 + col
        }
        Face::Back => {
            let col = (1.0 - pos.x) as usize;
            let row = (1.0 - pos.y) as usize;
            row * 3 + col
        }
    }
}

/// 各面の「上」方向（orientation=0 の基準）を定義します。
/// 展開図の接続 (U <-> B, B <-> D) がスムーズになるように定義。
fn get_face_up_axis(normal: Vec3) -> Vec3 {
    let n = Vec3::new(normal.x.round(), normal.y.round(), normal.z.round());
    if n == Vec3::Y {
        -Vec3::Z // U
    } else if n == -Vec3::Y {
        Vec3::Z // D
    } else {
        Vec3::Y // L, R, F, B
    }
}

/// 全26個のピースを初期完成状態で生成します。
pub fn get_initial_pieces() -> [Cubie; 26] {
    let mut pieces = Vec::new();

    // -1, 0, 1 の全組み合わせ（(0,0,0)を除く）
    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                if x == 0 && y == 0 && z == 0 {
                    continue;
                }

                let pos = Vec3::new(x as f32, y as f32, z as f32);
                let mut stickers = Vec::new();

                // 各座標が面の境界にある場合、ステッカーを追加
                if x == 1 {
                    stickers.push(CubieSticker {
                        initial_normal: Vec3::X,
                        color: Color::Blue,
                    });
                }
                if x == -1 {
                    stickers.push(CubieSticker {
                        initial_normal: -Vec3::X,
                        color: Color::Green,
                    });
                }
                if y == 1 {
                    stickers.push(CubieSticker {
                        initial_normal: Vec3::Y,
                        color: Color::White,
                    });
                }
                if y == -1 {
                    stickers.push(CubieSticker {
                        initial_normal: -Vec3::Y,
                        color: Color::Yellow,
                    });
                }
                if z == 1 {
                    stickers.push(CubieSticker {
                        initial_normal: Vec3::Z,
                        color: Color::Red,
                    });
                }
                if z == -1 {
                    stickers.push(CubieSticker {
                        initial_normal: -Vec3::Z,
                        color: Color::Orange,
                    });
                }

                pieces.push(Cubie::new(pos, stickers));
            }
        }
    }

    pieces.try_into().expect("Must have 26 pieces")
}

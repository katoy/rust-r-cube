use glam::Vec3;

/// 2x2ルービックキューブの6つの面を表します。
///
/// 内部的には0から5の整数値にマップされており、
/// 各面のステッカー配列の開始インデックスを計算するために使用されます。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Face {
    /// 上面 (Whiteが標準)
    Up = 0,
    /// 下面 (Yellowが標準)
    Down = 1,
    /// 左面 (Greenが標準)
    Left = 2,
    /// 右面 (Blueが標準)
    Right = 3,
    /// 前面 (Redが標準)
    Front = 4,
    /// 背面 (Orangeが標準)
    Back = 5,
}

impl Face {
    /// この面のステッカー配列における開始インデックスを取得します。
    ///
    /// 2x2キューブでは各面4枚のステッカーがあるため、
    /// インデックスは (面の番号 * 4) となります。
    #[must_use]
    pub const fn start_index(self) -> usize {
        (self as usize) * STICKERS_PER_FACE
    }

    /// 対面を取得します。
    #[must_use]
    pub fn opposite(self) -> Face {
        match self {
            Face::Up => Face::Down,
            Face::Down => Face::Up,
            Face::Left => Face::Right,
            Face::Right => Face::Left,
            Face::Front => Face::Back,
            Face::Back => Face::Front,
        }
    }

    /// 適当な隣接面を1つ取得します。
    #[must_use]
    pub fn any_adjacent(self) -> Face {
        match self {
            Face::Up => Face::Front,
            Face::Down => Face::Front,
            Face::Left => Face::Up,
            Face::Right => Face::Up,
            Face::Front => Face::Up,
            Face::Back => Face::Up,
        }
    }

    /// すべての面を列挙した配列を返します。
    ///
    /// # 例
    ///
    /// ```
    /// use rubiks_cube_3x3::cube::Face;
    /// for face in Face::all() {
    ///     println!("{:?}", face);
    /// }
    /// ```
    #[must_use]
    pub fn all() -> [Face; 6] {
        [
            Face::Up,
            Face::Down,
            Face::Left,
            Face::Right,
            Face::Front,
            Face::Back,
        ]
    }

    /// インデックスから面を取得します。
    pub fn from_index(index: usize) -> Self {
        match index / 9 {
            0 => Face::Up,
            1 => Face::Down,
            2 => Face::Left,
            3 => Face::Right,
            4 => Face::Front,
            5 => Face::Back,
            _ => Face::Up,
        }
    }

    /// 面内の 0-8 のインデックスから空間座標を返します。
    pub fn to_pos_for_local_index(&self, local_idx: usize) -> Vec3 {
        let row = (local_idx / 3) as f32;
        let col = (local_idx % 3) as f32;
        match self {
            Face::Up => Vec3::new(col - 1.0, 1.0, row - 1.0),
            Face::Down => Vec3::new(col - 1.0, -1.0, 1.0 - row),
            Face::Left => Vec3::new(-1.0, 1.0 - row, col - 1.0),
            Face::Right => Vec3::new(1.0, 1.0 - row, 1.0 - col),
            Face::Front => Vec3::new(col - 1.0, 1.0 - row, 1.0),
            Face::Back => Vec3::new(1.0 - col, 1.0 - row, -1.0),
        }
    }
}

/// ステッカーの総数
pub const NUM_STICKERS: usize = 54;
/// 1面あたりのステッカー数
pub const STICKERS_PER_FACE: usize = 9;

/// ステッカーの色の定義です。
///
/// 標準的なルービックキューブの6色に加えて、
/// 未設定状態を表す `Gray` を含みます。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    /// 白 (通常は上面)
    White,
    /// 黄 (通常は下面)
    Yellow,
    /// 緑 (通常は左面)
    Green,
    /// 青 (通常は右面)
    Blue,
    /// 赤 (通常は前面)
    Red,
    /// 橙 (通常は背面)
    Orange,
    /// 灰色 (未設定・無効な色)
    Gray,
}

/// 1枚のステッカーを表す構造体。
///
/// 色情報（[`Color`]）と、そのステッカーの現在の向き（[`orientation`](Self::orientation)）を保持します。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sticker {
    /// ステッカーの色
    pub color: Color,
    /// 向き（0-3の値で、90度単位の物理的な回転状態を表す。0が初期状態）
    pub orientation: u8,
}

impl Sticker {
    /// 指定された色で、向きが0（初期状態）のステッカーを作成します。
    #[must_use]
    pub fn new(color: Color) -> Self {
        Self {
            color,
            orientation: 0,
        }
    }

    /// ステッカーを時計回りに90度回転させます（向き情報を更新）。
    pub fn rotate_cw(&mut self) {
        self.orientation = (self.orientation + 1) % 4;
    }

    /// ステッカーを反時計回りに90度回転させます（向き情報を更新）。
    pub fn rotate_ccw(&mut self) {
        self.orientation = (self.orientation + 3) % 4;
    }
}

/// キューブに対する単一の回転操作を定義します。
///
/// 各文字（R, L, U, D, F, B）は操作する面を表し、
/// 接尾辞（なし, p, 2）は回転の量と方向を表します。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Move {
    /// 右面 時計回り 90度
    R,
    /// 右面 反時計回り 90度 (Prime)
    Rp,
    /// 右面 180度
    R2,
    /// 左面 時計回り 90度
    L,
    /// 左面 反時計回り 90度
    Lp,
    /// 左面 180度
    L2,
    /// 上面 時計回り 90度
    U,
    /// 上面 反時計回り 90度
    Up,
    /// 上面 180度
    U2,
    /// 下面 時計回り 90度
    D,
    /// 下面 反時計回り 90度
    Dp,
    /// 下面 180度
    D2,
    /// 前面 時計回り 90度
    F,
    /// 前面 反時計回り 90度
    Fp,
    /// 前面 180度
    F2,
    /// 背面 時計回り 90度
    B,
    /// 背面 反時計回り 90度
    Bp,
    /// 背面 180度
    B2,
    /// 中層(M) 時計回り 90度
    M,
    /// 中層(M) 反時計回り 90度
    Mp,
    /// 中層(M) 180度
    M2,
    /// 中層(E) 時計回り 90度
    E,
    /// 中層(E) 反時計回り 90度
    Ep,
    /// 中層(E) 180度
    E2,
    /// 中層(S) 時計回り 90度
    S,
    /// 中層(S) 反時計回り 90度
    Sp,
    /// 中層(S) 180度
    S2,
    /// 全体(X) 時計回り 90度
    X,
    /// 全体(X) 反時計回り 90度
    Xp,
    /// 全体(X) 180度
    X2,
    /// 全体(Y) 時計回り 90度
    Y,
    /// 全体(Y) 反時計回り 90度
    Yp,
    /// 全体(Y) 180度
    Y2,
    /// 全体(Z) 時計回り 90度
    Z,
    /// 全体(Z) 反時計回り 90度
    Zp,
    /// 全体(Z) 180度
    Z2,
}

impl Move {
    /// 全体回転（X, Y, Z）であるかを判定します。
    #[must_use]
    pub fn is_global(self) -> bool {
        matches!(
            self,
            Move::X
                | Move::Xp
                | Move::X2
                | Move::Y
                | Move::Yp
                | Move::Y2
                | Move::Z
                | Move::Zp
                | Move::Z2
        )
    }

    /// 中層回転（M, E, S）であるかを判定します。
    #[must_use]
    pub fn is_middle_layer(self) -> bool {
        matches!(
            self,
            Move::M
                | Move::Mp
                | Move::M2
                | Move::E
                | Move::Ep
                | Move::E2
                | Move::S
                | Move::Sp
                | Move::S2
        )
    }

    /// 基本面回転（U, D, L, R, F, B）であるかを判定します。
    #[must_use]
    pub fn is_face_move(self) -> bool {
        !self.is_global() && !self.is_middle_layer()
    }

    /// 利用可能なすべての回転操作（36種類）を一覧したベクタを返します。
    #[must_use]
    pub fn all_moves() -> Vec<Move> {
        vec![
            Move::R,
            Move::Rp,
            Move::R2,
            Move::L,
            Move::Lp,
            Move::L2,
            Move::U,
            Move::Up,
            Move::U2,
            Move::D,
            Move::Dp,
            Move::D2,
            Move::F,
            Move::Fp,
            Move::F2,
            Move::B,
            Move::Bp,
            Move::B2,
            Move::M,
            Move::Mp,
            Move::M2,
            Move::E,
            Move::Ep,
            Move::E2,
            Move::S,
            Move::Sp,
            Move::S2,
            Move::X,
            Move::Xp,
            Move::X2,
            Move::Y,
            Move::Yp,
            Move::Y2,
            Move::Z,
            Move::Zp,
            Move::Z2,
        ]
    }

    /// 指定された操作の逆操作（反対方向の回転）を返します。
    ///
    /// 180度回転（2）の逆操作は、便宜上同じ180度回転を返します。
    #[must_use]
    pub fn inverse(self) -> Move {
        match self {
            Move::R => Move::Rp,
            Move::Rp => Move::R,
            Move::R2 => Move::R2,
            Move::L => Move::Lp,
            Move::Lp => Move::L,
            Move::L2 => Move::L2,
            Move::U => Move::Up,
            Move::Up => Move::U,
            Move::U2 => Move::U2,
            Move::D => Move::Dp,
            Move::Dp => Move::D,
            Move::D2 => Move::D2,
            Move::F => Move::Fp,
            Move::Fp => Move::F,
            Move::F2 => Move::F2,
            Move::B => Move::Bp,
            Move::Bp => Move::B,
            Move::B2 => Move::B2,
            Move::M => Move::Mp,
            Move::Mp => Move::M,
            Move::M2 => Move::M2,
            Move::E => Move::Ep,
            Move::Ep => Move::E,
            Move::E2 => Move::E2,
            Move::S => Move::Sp,
            Move::Sp => Move::S,
            Move::S2 => Move::S2,
            Move::X => Move::Xp,
            Move::Xp => Move::X,
            Move::X2 => Move::X2,
            Move::Y => Move::Yp,
            Move::Yp => Move::Y,
            Move::Y2 => Move::Y2,
            Move::Z => Move::Zp,
            Move::Zp => Move::Z,
            Move::Z2 => Move::Z2,
        }
    }

    /// 180度回転操作を90度回転操作2回に分割するための、1回分の操作を取得します。
    ///
    /// 90度回転（なし/p）の場合は `None` を返します。
    #[must_use]
    pub fn split_to_single(self) -> Option<Move> {
        match self {
            Move::R2 => Some(Move::R),
            Move::L2 => Some(Move::L),
            Move::U2 => Some(Move::U),
            Move::D2 => Some(Move::D),
            Move::F2 => Some(Move::F),
            Move::B2 => Some(Move::B),
            Move::M2 => Some(Move::M),
            Move::E2 => Some(Move::E),
            Move::S2 => Some(Move::S),
            Move::X2 => Some(Move::X),
            Move::Y2 => Some(Move::Y),
            Move::Z2 => Some(Move::Z),
            _ => None,
        }
    }

    /// 操作を幾何学的なパラメータ（回転軸、角度）に変換します。
    pub fn geometric_params(self) -> (Vec3, f32) {
        let pi_2 = std::f32::consts::FRAC_PI_2;
        match self {
            Move::R => (Vec3::X, -pi_2),
            Move::Rp => (Vec3::X, pi_2),
            Move::R2 => (Vec3::X, pi_2 * 2.0),
            Move::L => (Vec3::X, pi_2),
            Move::Lp => (Vec3::X, -pi_2),
            Move::L2 => (Vec3::X, pi_2 * 2.0),
            Move::U => (Vec3::Y, -pi_2),
            Move::Up => (Vec3::Y, pi_2),
            Move::U2 => (Vec3::Y, pi_2 * 2.0),
            Move::D => (Vec3::Y, pi_2),
            Move::Dp => (Vec3::Y, -pi_2),
            Move::D2 => (Vec3::Y, pi_2 * 2.0),
            Move::F => (Vec3::Z, -pi_2),
            Move::Fp => (Vec3::Z, pi_2),
            Move::F2 => (Vec3::Z, pi_2 * 2.0),
            Move::B => (Vec3::Z, pi_2),
            Move::Bp => (Vec3::Z, -pi_2),
            Move::B2 => (Vec3::Z, pi_2 * 2.0),
            Move::M => (Vec3::X, pi_2),
            Move::Mp => (Vec3::X, -pi_2),
            Move::M2 => (Vec3::X, pi_2 * 2.0),
            Move::E => (Vec3::Y, pi_2),
            Move::Ep => (Vec3::Y, -pi_2),
            Move::E2 => (Vec3::Y, pi_2 * 2.0),
            Move::S => (Vec3::Z, -pi_2),
            Move::Sp => (Vec3::Z, pi_2),
            Move::S2 => (Vec3::Z, pi_2 * 2.0),
            Move::X => (Vec3::X, -pi_2),
            Move::Xp => (Vec3::X, pi_2),
            Move::X2 => (Vec3::X, pi_2 * 2.0),
            Move::Y => (Vec3::Y, -pi_2),
            Move::Yp => (Vec3::Y, pi_2),
            Move::Y2 => (Vec3::Y, pi_2 * 2.0),
            Move::Z => (Vec3::Z, -pi_2),
            Move::Zp => (Vec3::Z, pi_2),
            Move::Z2 => (Vec3::Z, pi_2 * 2.0),
        }
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Move::R => "R",
            Move::Rp => "R'",
            Move::R2 => "R2",
            Move::L => "L",
            Move::Lp => "L'",
            Move::L2 => "L2",
            Move::U => "U",
            Move::Up => "U'",
            Move::U2 => "U2",
            Move::D => "D",
            Move::Dp => "D'",
            Move::D2 => "D2",
            Move::F => "F",
            Move::Fp => "F'",
            Move::F2 => "F2",
            Move::B => "B",
            Move::Bp => "B'",
            Move::B2 => "B2",
            Move::M => "M",
            Move::Mp => "M'",
            Move::M2 => "M2",
            Move::E => "E",
            Move::Ep => "E'",
            Move::E2 => "E2",
            Move::S => "S",
            Move::Sp => "S'",
            Move::S2 => "S2",
            Move::X => "X",
            Move::Xp => "X'",
            Move::X2 => "X2",
            Move::Y => "Y",
            Move::Yp => "Y'",
            Move::Y2 => "Y2",
            Move::Z => "Z",
            Move::Zp => "Z'",
            Move::Z2 => "Z2",
        };
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_face_opposite() {
        assert_eq!(Face::Up.opposite(), Face::Down);
        assert_eq!(Face::Down.opposite(), Face::Up);
        assert_eq!(Face::Left.opposite(), Face::Right);
        assert_eq!(Face::Right.opposite(), Face::Left);
        assert_eq!(Face::Front.opposite(), Face::Back);
        assert_eq!(Face::Back.opposite(), Face::Front);
    }

    #[test]
    fn test_face_any_adjacent() {
        for f in Face::all() {
            let adj = f.any_adjacent();
            assert_ne!(f, adj);
            assert_ne!(f.opposite(), adj);
        }
    }

    #[test]
    fn test_face_to_pos_for_local_index() {
        for f in Face::all() {
            for i in 0..9 {
                let pos = f.to_pos_for_local_index(i);
                assert!(pos.length() > 0.0);
            }
        }
    }

    #[test]
    fn test_face_from_index() {
        assert_eq!(Face::from_index(0), Face::Up);
        assert_eq!(Face::from_index(9), Face::Down);
        assert_eq!(Face::from_index(18), Face::Left);
        assert_eq!(Face::from_index(27), Face::Right);
        assert_eq!(Face::from_index(36), Face::Front);
        assert_eq!(Face::from_index(45), Face::Back);
        assert_eq!(Face::from_index(54), Face::Up); // Default case
    }

    #[test]
    fn test_move_properties() {
        assert!(!Move::R.is_global());
        assert!(!Move::R.is_middle_layer());
        assert!(Move::R.is_face_move());

        assert!(Move::X.is_global());
        assert!(!Move::X.is_middle_layer());
        assert!(!Move::X.is_face_move());

        assert!(!Move::M.is_global());
        assert!(Move::M.is_middle_layer());
        assert!(!Move::M.is_face_move());
    }

    #[test]
    fn test_move_split() {
        assert_eq!(Move::R2.split_to_single(), Some(Move::R));
        assert_eq!(Move::R.split_to_single(), None);
    }

    #[test]
    fn test_move_geometric_params() {
        for m in Move::all_moves() {
            let (axis, angle) = m.geometric_params();
            assert!(axis.length() > 0.0);
            assert!(angle != 0.0);
        }
    }

    #[test]
    fn test_move_display() {
        assert_eq!(format!("{}", Move::R), "R");
        assert_eq!(format!("{}", Move::Rp), "R'");
        assert_eq!(format!("{}", Move::R2), "R2");
    }

    #[test]
    fn test_sticker_rotate() {
        let mut s = Sticker::new(crate::cube::Color::White);
        s.rotate_cw();
        assert_eq!(s.orientation, 1);
        s.rotate_ccw();
        assert_eq!(s.orientation, 0);
    }
}

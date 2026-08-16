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

    /// すべての面を列挙した配列を返します。
    ///
    /// # 例
    ///
    /// ```
    /// use rubiks_cube_2x2::cube::Face;
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
}

/// ステッカーの総数
pub const NUM_STICKERS: usize = 24;
/// 1面あたりのステッカー数
pub const STICKERS_PER_FACE: usize = 4;

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

impl Color {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Color::White,
            1 => Color::Yellow,
            2 => Color::Green,
            3 => Color::Blue,
            4 => Color::Red,
            5 => Color::Orange,
            _ => Color::Gray,
        }
    }
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
    /// 上面 時計回り 90度
    U,
    /// 上面 180度
    U2,
    /// 上面 反時計回り 90度 (Prime)
    Up,
    /// 下面 時計回り 90度
    D,
    /// 下面 180度
    D2,
    /// 下面 反時計回り 90度 (Prime)
    Dp,
    /// 左面 時計回り 90度
    L,
    /// 左面 180度
    L2,
    /// 左面 反時計回り 90度 (Prime)
    Lp,
    /// 右面 時計回り 90度
    R,
    /// 右面 180度
    R2,
    /// 右面 反時計回り 90度 (Prime)
    Rp,
    /// 前面 時計回り 90度
    F,
    /// 前面 180度
    F2,
    /// 前面 反時計回り 90度 (Prime)
    Fp,
    /// 背面 時計回り 90度
    B,
    /// 背面 180度
    B2,
    /// 背面 反時計回り 90度 (Prime)
    Bp,
}

impl Move {
    /// 利用可能なすべての回転操作（18種類）を一覧したベクタを返します。
    #[must_use]
    pub fn all_moves() -> Vec<Move> {
        vec![
            Move::U,
            Move::U2,
            Move::Up,
            Move::D,
            Move::D2,
            Move::Dp,
            Move::L,
            Move::L2,
            Move::Lp,
            Move::R,
            Move::R2,
            Move::Rp,
            Move::F,
            Move::F2,
            Move::Fp,
            Move::B,
            Move::B2,
            Move::Bp,
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
            _ => None,
        }
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Move::U => "U",
            Move::U2 => "U2",
            Move::Up => "U'",
            Move::D => "D",
            Move::D2 => "D2",
            Move::Dp => "D'",
            Move::L => "L",
            Move::L2 => "L2",
            Move::Lp => "L'",
            Move::R => "R",
            Move::R2 => "R2",
            Move::Rp => "R'",
            Move::F => "F",
            Move::F2 => "F2",
            Move::Fp => "F'",
            Move::B => "B",
            Move::B2 => "B2",
            Move::Bp => "B'",
        };
        write!(f, "{s}")
    }
}

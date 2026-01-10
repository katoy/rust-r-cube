/// キューブの面を表す列挙型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Face {
    Up,
    Down,
    Left,
    Right,
    Front,
    Back,
}

/// ステッカーの色
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Yellow,
    Green,
    Blue,
    Red,
    Orange,
    Gray, // 未設定のセル用
}

/// ステッカー（色と向き情報を持つ）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sticker {
    pub color: Color,
    /// 向き（0-3の値で、90度単位の回転を表す）
    pub orientation: u8,
}

impl Sticker {
    #[must_use]
    pub fn new(color: Color) -> Self {
        Self {
            color,
            orientation: 0,
        }
    }

    /// 時計回りに90度回転
    pub fn rotate_cw(&mut self) {
        self.orientation = (self.orientation + 1) % 4;
    }

    /// 反時計回りに90度回転
    pub fn rotate_ccw(&mut self) {
        self.orientation = (self.orientation + 3) % 4;
    }
}

/// 回転操作
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Move {
    R,  // Right face clockwise
    Rp, // Right face counter-clockwise
    L,  // Left face clockwise
    Lp, // Left face counter-clockwise
    U,  // Up face clockwise
    Up, // Up face counter-clockwise
    D,  // Down face clockwise
    Dp, // Down face counter-clockwise
    F,  // Front face clockwise
    Fp, // Front face counter-clockwise
    B,  // Back face clockwise
    Bp, // Back face counter-clockwise
}

impl Move {
    /// すべての回転操作を取得
    #[must_use]
    pub fn all_moves() -> Vec<Move> {
        vec![
            Move::R,
            Move::Rp,
            Move::L,
            Move::Lp,
            Move::U,
            Move::Up,
            Move::D,
            Move::Dp,
            Move::F,
            Move::Fp,
            Move::B,
            Move::Bp,
        ]
    }

    /// 逆操作を取得
    #[must_use]
    pub fn inverse(self) -> Move {
        match self {
            Move::R => Move::Rp,
            Move::Rp => Move::R,
            Move::L => Move::Lp,
            Move::Lp => Move::L,
            Move::U => Move::Up,
            Move::Up => Move::U,
            Move::D => Move::Dp,
            Move::Dp => Move::D,
            Move::F => Move::Fp,
            Move::Fp => Move::F,
            Move::B => Move::Bp,
            Move::Bp => Move::B,
        }
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Move::R => "R",
            Move::Rp => "R'",
            Move::L => "L",
            Move::Lp => "L'",
            Move::U => "U",
            Move::Up => "U'",
            Move::D => "D",
            Move::Dp => "D'",
            Move::F => "F",
            Move::Fp => "F'",
            Move::B => "B",
            Move::Bp => "B'",
        };
        write!(f, "{s}")
    }
}

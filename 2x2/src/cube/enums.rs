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
    R2, // Right face 180 degrees
    L,  // Left face clockwise
    Lp, // Left face counter-clockwise
    L2, // Left face 180 degrees
    U,  // Up face clockwise
    Up, // Up face counter-clockwise
    U2, // Up face 180 degrees
    D,  // Down face clockwise
    Dp, // Down face counter-clockwise
    D2, // Down face 180 degrees
    F,  // Front face clockwise
    Fp, // Front face counter-clockwise
    F2, // Front face 180 degrees
    B,  // Back face clockwise
    Bp, // Back face counter-clockwise
    B2, // Back face 180 degrees
}

impl Move {
    /// すべての回転操作を取得
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
        ]
    }

    /// 逆操作を取得
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
        };
        write!(f, "{s}")
    }
}

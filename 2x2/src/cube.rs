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
}

/// ステッカー（色と向き情報を持つ）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sticker {
    pub color: Color,
    /// 向き（0-3の値で、90度単位の回転を表す）
    pub orientation: u8,
}

impl Sticker {
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
        write!(f, "{}", s)
    }
}

/// 2x2 ルービックキューブ
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cube {
    /// 各面のステッカー（各面4つ）
    /// インデックス: 0-3 (Up), 4-7 (Down), 8-11 (Left), 12-15 (Right), 16-19 (Front), 20-23 (Back)
    stickers: [Sticker; 24],
}

impl Cube {
    /// 完成状態のキューブを作成
    pub fn new() -> Self {
        let mut stickers = [Sticker::new(Color::White); 24];

        // Up face (White)
        for sticker in stickers.iter_mut().take(4) {
            *sticker = Sticker::new(Color::White);
        }
        // Down face (Yellow)
        for sticker in stickers.iter_mut().take(8).skip(4) {
            *sticker = Sticker::new(Color::Yellow);
        }
        // Left face (Green)
        for sticker in stickers.iter_mut().take(12).skip(8) {
            *sticker = Sticker::new(Color::Green);
        }
        // Right face (Blue)
        for sticker in stickers.iter_mut().take(16).skip(12) {
            *sticker = Sticker::new(Color::Blue);
        }
        // Front face (Red)
        for sticker in stickers.iter_mut().take(20).skip(16) {
            *sticker = Sticker::new(Color::Red);
        }
        // Back face (Orange)
        for sticker in stickers.iter_mut().skip(20) {
            *sticker = Sticker::new(Color::Orange);
        }

        Self { stickers }
    }

    /// キューブが完成しているか判定
    pub fn is_solved(&self) -> bool {
        for face_start in (0..24).step_by(4) {
            let color = self.stickers[face_start].color;
            for i in 1..4 {
                if self.stickers[face_start + i].color != color {
                    return false;
                }
            }
        }
        true
    }

    /// ステッカーを取得
    pub fn get_sticker(&self, index: usize) -> Sticker {
        self.stickers[index]
    }

    /// 回転操作を実行
    pub fn apply_move(&mut self, mv: Move) {
        match mv {
            Move::R => self.rotate_r(),
            Move::Rp => self.rotate_rp(),
            Move::L => self.rotate_l(),
            Move::Lp => self.rotate_lp(),
            Move::U => self.rotate_u(),
            Move::Up => self.rotate_up(),
            Move::D => self.rotate_d(),
            Move::Dp => self.rotate_dp(),
            Move::F => self.rotate_f(),
            Move::Fp => self.rotate_fp(),
            Move::B => self.rotate_b(),
            Move::Bp => self.rotate_bp(),
        }
    }

    /// 面を時計回りに90度回転（内部ヘルパー）
    fn rotate_face_cw(&mut self, face_start: usize) {
        let temp = self.stickers[face_start];
        self.stickers[face_start] = self.stickers[face_start + 2];
        self.stickers[face_start + 2] = self.stickers[face_start + 3];
        self.stickers[face_start + 3] = self.stickers[face_start + 1];
        self.stickers[face_start + 1] = temp;

        // 面上のステッカーの向きを更新
        for i in 0..4 {
            self.stickers[face_start + i].rotate_cw();
        }
    }

    /// 面を反時計回りに90度回転（内部ヘルパー）
    fn rotate_face_ccw(&mut self, face_start: usize) {
        let temp = self.stickers[face_start];
        self.stickers[face_start] = self.stickers[face_start + 1];
        self.stickers[face_start + 1] = self.stickers[face_start + 3];
        self.stickers[face_start + 3] = self.stickers[face_start + 2];
        self.stickers[face_start + 2] = temp;

        // 面上のステッカーの向きを更新
        for i in 0..4 {
            self.stickers[face_start + i].rotate_ccw();
        }
    }

    /// R面を時計回りに回転
    fn rotate_r(&mut self) {
        self.rotate_face_cw(12); // Right face

        let mut temp0 = self.stickers[1];
        temp0.rotate_cw();
        let mut temp1 = self.stickers[3];
        temp1.rotate_cw();

        self.stickers[1] = self.stickers[17];
        self.stickers[1].rotate_cw();
        self.stickers[3] = self.stickers[19];
        self.stickers[3].rotate_cw();

        self.stickers[17] = self.stickers[5];
        self.stickers[17].rotate_cw();
        self.stickers[19] = self.stickers[7];
        self.stickers[19].rotate_cw();

        self.stickers[5] = self.stickers[22];
        self.stickers[5].rotate_cw();
        self.stickers[7] = self.stickers[20];
        self.stickers[7].rotate_cw();

        self.stickers[22] = temp0;
        self.stickers[20] = temp1;
    }

    /// R面を反時計回りに回転
    fn rotate_rp(&mut self) {
        self.rotate_face_ccw(12); // Right face

        let mut temp0 = self.stickers[1];
        temp0.rotate_ccw();
        let mut temp1 = self.stickers[3];
        temp1.rotate_ccw();

        self.stickers[1] = self.stickers[22];
        self.stickers[1].rotate_ccw();
        self.stickers[3] = self.stickers[20];
        self.stickers[3].rotate_ccw();

        self.stickers[22] = self.stickers[5];
        self.stickers[22].rotate_ccw();
        self.stickers[20] = self.stickers[7];
        self.stickers[20].rotate_ccw();

        self.stickers[5] = self.stickers[17];
        self.stickers[5].rotate_ccw();
        self.stickers[7] = self.stickers[19];
        self.stickers[7].rotate_ccw();

        self.stickers[17] = temp0;
        self.stickers[19] = temp1;
    }

    /// L面を時計回りに回転
    fn rotate_l(&mut self) {
        self.rotate_face_cw(8); // Left face

        let mut temp0 = self.stickers[0];
        temp0.rotate_cw();
        let mut temp1 = self.stickers[2];
        temp1.rotate_cw();

        self.stickers[0] = self.stickers[21];
        self.stickers[0].rotate_cw();
        self.stickers[2] = self.stickers[23];
        self.stickers[2].rotate_cw();

        self.stickers[21] = self.stickers[4];
        self.stickers[21].rotate_cw();
        self.stickers[23] = self.stickers[6];
        self.stickers[23].rotate_cw();

        self.stickers[4] = self.stickers[16];
        self.stickers[4].rotate_cw();
        self.stickers[6] = self.stickers[18];
        self.stickers[6].rotate_cw();

        self.stickers[16] = temp0;
        self.stickers[18] = temp1;
    }

    /// L面を反時計回りに回転
    fn rotate_lp(&mut self) {
        self.rotate_face_ccw(8); // Left face

        let mut temp0 = self.stickers[0];
        temp0.rotate_ccw();
        let mut temp1 = self.stickers[2];
        temp1.rotate_ccw();

        self.stickers[0] = self.stickers[16];
        self.stickers[0].rotate_ccw();
        self.stickers[2] = self.stickers[18];
        self.stickers[2].rotate_ccw();

        self.stickers[16] = self.stickers[4];
        self.stickers[16].rotate_ccw();
        self.stickers[18] = self.stickers[6];
        self.stickers[18].rotate_ccw();

        self.stickers[4] = self.stickers[21];
        self.stickers[4].rotate_ccw();
        self.stickers[6] = self.stickers[23];
        self.stickers[6].rotate_ccw();

        self.stickers[21] = temp0;
        self.stickers[23] = temp1;
    }

    /// U面を時計回りに回転
    fn rotate_u(&mut self) {
        self.rotate_face_cw(0); // Up face

        let temp0 = self.stickers[16];
        let temp1 = self.stickers[17];

        self.stickers[16] = self.stickers[12];
        self.stickers[17] = self.stickers[13];

        self.stickers[12] = self.stickers[20];
        self.stickers[13] = self.stickers[21];

        self.stickers[20] = self.stickers[8];
        self.stickers[21] = self.stickers[9];

        self.stickers[8] = temp0;
        self.stickers[9] = temp1;
    }

    /// U面を反時計回りに回転
    fn rotate_up(&mut self) {
        self.rotate_face_ccw(0); // Up face

        let temp0 = self.stickers[16];
        let temp1 = self.stickers[17];

        self.stickers[16] = self.stickers[8];
        self.stickers[17] = self.stickers[9];

        self.stickers[8] = self.stickers[20];
        self.stickers[9] = self.stickers[21];

        self.stickers[20] = self.stickers[12];
        self.stickers[21] = self.stickers[13];

        self.stickers[12] = temp0;
        self.stickers[13] = temp1;
    }

    /// D面を時計回りに回転
    fn rotate_d(&mut self) {
        self.rotate_face_cw(4); // Down face

        let temp0 = self.stickers[18];
        let temp1 = self.stickers[19];

        self.stickers[18] = self.stickers[10];
        self.stickers[19] = self.stickers[11];

        self.stickers[10] = self.stickers[22];
        self.stickers[11] = self.stickers[23];

        self.stickers[22] = self.stickers[14];
        self.stickers[23] = self.stickers[15];

        self.stickers[14] = temp0;
        self.stickers[15] = temp1;
    }

    /// D面を反時計回りに回転
    fn rotate_dp(&mut self) {
        self.rotate_face_ccw(4); // Down face

        let temp0 = self.stickers[18];
        let temp1 = self.stickers[19];

        self.stickers[18] = self.stickers[14];
        self.stickers[19] = self.stickers[15];

        self.stickers[14] = self.stickers[22];
        self.stickers[15] = self.stickers[23];

        self.stickers[22] = self.stickers[10];
        self.stickers[23] = self.stickers[11];

        self.stickers[10] = temp0;
        self.stickers[11] = temp1;
    }

    /// F面を時計回りに回転
    fn rotate_f(&mut self) {
        self.rotate_face_cw(16); // Front face

        let temp0 = self.stickers[2];
        let temp1 = self.stickers[3];

        self.stickers[2] = self.stickers[11];
        self.stickers[3] = self.stickers[9];

        self.stickers[11] = self.stickers[4];
        self.stickers[9] = self.stickers[5];

        self.stickers[4] = self.stickers[12];
        self.stickers[5] = self.stickers[14];

        self.stickers[12] = temp0;
        self.stickers[14] = temp1;
    }

    /// F面を反時計回りに回転
    fn rotate_fp(&mut self) {
        self.rotate_face_ccw(16); // Front face

        let mut temp0 = self.stickers[2];
        temp0.rotate_ccw();
        let mut temp1 = self.stickers[3];
        temp1.rotate_ccw();

        self.stickers[2] = self.stickers[12];
        self.stickers[2].rotate_ccw();
        self.stickers[3] = self.stickers[14];
        self.stickers[3].rotate_ccw();

        self.stickers[12] = self.stickers[4];
        self.stickers[12].rotate_ccw();
        self.stickers[14] = self.stickers[5];
        self.stickers[14].rotate_ccw();

        self.stickers[4] = self.stickers[11];
        self.stickers[4].rotate_ccw();
        self.stickers[5] = self.stickers[9];
        self.stickers[5].rotate_ccw();

        self.stickers[11] = temp0;
        self.stickers[9] = temp1;
    }

    /// B面を時計回りに回転
    fn rotate_b(&mut self) {
        self.rotate_face_cw(20); // Back face

        let mut temp0 = self.stickers[0];
        temp0.rotate_cw();
        let mut temp1 = self.stickers[1];
        temp1.rotate_cw();

        self.stickers[0] = self.stickers[13];
        self.stickers[0].rotate_cw();
        self.stickers[1] = self.stickers[15];
        self.stickers[1].rotate_cw();

        self.stickers[13] = self.stickers[6];
        self.stickers[13].rotate_cw();
        self.stickers[15] = self.stickers[7];
        self.stickers[15].rotate_cw();

        self.stickers[6] = self.stickers[10];
        self.stickers[6].rotate_cw();
        self.stickers[7] = self.stickers[8];
        self.stickers[7].rotate_cw();

        self.stickers[10] = temp0;
        self.stickers[8] = temp1;
    }

    /// B面を反時計回りに回転
    fn rotate_bp(&mut self) {
        self.rotate_face_ccw(20); // Back face

        let mut temp0 = self.stickers[0];
        temp0.rotate_ccw();
        let mut temp1 = self.stickers[1];
        temp1.rotate_ccw();

        self.stickers[0] = self.stickers[10];
        self.stickers[0].rotate_ccw();
        self.stickers[1] = self.stickers[8];
        self.stickers[1].rotate_ccw();

        self.stickers[10] = self.stickers[7];
        self.stickers[10].rotate_ccw();
        self.stickers[8] = self.stickers[6];
        self.stickers[8].rotate_ccw();

        self.stickers[7] = self.stickers[13];
        self.stickers[7].rotate_ccw();
        self.stickers[6] = self.stickers[15];
        self.stickers[6].rotate_ccw();

        self.stickers[13] = temp0;
        self.stickers[15] = temp1;
    }

    /// ランダムなスクランブルを生成
    pub fn scramble(&mut self, moves: usize) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let all_moves = Move::all_moves();

        for _ in 0..moves {
            let mv = all_moves[rng.gen_range(0..all_moves.len())];
            self.apply_move(mv);
        }
    }

    /// 色情報のみ比較するために、向き情報をリセットしたCubeを返す
    pub fn normalized(&self) -> Self {
        let mut new_cube = self.clone();
        for sticker in &mut new_cube.stickers {
            sticker.orientation = 0;
        }
        new_cube
    }
}

impl Default for Cube {
    fn default() -> Self {
        Self::new()
    }
}

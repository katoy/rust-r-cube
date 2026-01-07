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
    /// 完成状態のキューブを作成します。
    ///
    /// # 戻り値
    ///
    /// 各面が単色で揃った初期状態の2x2ルービックキューブ。
    /// - Up面: 白 (White)
    /// - Down面: 黄 (Yellow)
    /// - Left面: 緑 (Green)
    /// - Right面: 青 (Blue)
    /// - Front面: 赤 (Red)
    /// - Back面: オレンジ (Orange)
    ///
    /// # 例
    ///
    /// ```
    /// use rubiks_cube_2x2::cube::Cube;
    ///
    /// let cube = Cube::new();
    /// assert!(cube.is_solved());
    /// ```
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

    /// キューブが完成しているか判定します（色のみ、向きは無視）。
    ///
    /// 各面が単色で揃っていれば `true` を返します。
    /// ステッカーの向き（矢印の方向）は考慮されません。
    ///
    /// # 戻り値
    ///
    /// - `true`: 各面が単色で揃っている
    /// - `false`: 少なくとも1面が揃っていない
    ///
    /// # 例
    ///
    /// ```
    /// use rubiks_cube_2x2::cube::{Cube, Move};
    ///
    /// let mut cube = Cube::new();
    /// assert!(cube.is_solved());
    ///
    /// cube.apply_move(Move::R);
    /// assert!(!cube.is_solved());
    ///
    /// cube.apply_move(Move::Rp);
    /// assert!(cube.is_solved());
    /// ```
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

    /// 指定したインデックスのステッカーを取得します。
    ///
    /// # 引数
    ///
    /// - `index`: ステッカーのインデックス (0-23)
    ///   - 0-3: Up面
    ///   - 4-7: Down面
    ///   - 8-11: Left面
    ///   - 12-15: Right面
    ///   - 16-19: Front面
    ///   - 20-23: Back面
    ///
    /// # 戻り値
    ///
    /// 指定されたインデックスのステッカー（色と向き情報を含む）
    pub fn get_sticker(&self, index: usize) -> Sticker {
        self.stickers[index]
    }

    /// 回転操作を実行
    /// 指定した操作をキューブに適用します。
    ///
    /// # 引数
    ///
    /// - `mv`: 適用する操作（R, L, U, D, F, B およびその逆操作）
    ///
    /// # 例
    ///
    /// ```
    /// use rubiks_cube_2x2::cube::{Cube, Move};
    ///
    /// let mut cube = Cube::new();
    /// cube.apply_move(Move::R);
    /// cube.apply_move(Move::U);
    /// ```
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

    /// 面自体を時計回りに回転
    fn rotate_face_cw(&mut self, start_idx: usize, orient_delta: u8) {
        let temp = self.stickers[start_idx];
        self.stickers[start_idx] = self.stickers[start_idx + 2];
        self.stickers[start_idx + 2] = self.stickers[start_idx + 3];
        self.stickers[start_idx + 3] = self.stickers[start_idx + 1];
        self.stickers[start_idx + 1] = temp;

        for i in 0..4 {
            for _ in 0..orient_delta {
                self.stickers[start_idx + i].rotate_cw();
            }
        }
    }

    /// 面自体を反時計回りに回転
    fn rotate_face_ccw(&mut self, start_idx: usize, orient_delta: u8) {
        let temp = self.stickers[start_idx];
        self.stickers[start_idx] = self.stickers[start_idx + 1];
        self.stickers[start_idx + 1] = self.stickers[start_idx + 3];
        self.stickers[start_idx + 3] = self.stickers[start_idx + 2];
        self.stickers[start_idx + 2] = temp;

        for i in 0..4 {
            for _ in 0..orient_delta {
                self.stickers[start_idx + i].rotate_ccw();
            }
        }
    }

    /// R面を時計回りに回転
    fn rotate_r(&mut self) {
        self.rotate_face_cw(12, 3); // Right face (orient +3)

        let temp0 = self.stickers[1];
        let temp1 = self.stickers[3];

        // U <- F (R-slice) [0]
        self.stickers[1] = self.stickers[17];
        self.stickers[3] = self.stickers[19];

        // F <- D (R-slice) [0]
        self.stickers[17] = self.stickers[5];
        self.stickers[19] = self.stickers[7];

        // D <- B [+2]
        self.stickers[5] = self.stickers[22];
        self.stickers[5].rotate_cw();
        self.stickers[5].rotate_cw();
        self.stickers[7] = self.stickers[20];
        self.stickers[7].rotate_cw();
        self.stickers[7].rotate_cw();

        // B <- U [+2]
        self.stickers[22] = temp0;
        self.stickers[22].rotate_cw();
        self.stickers[22].rotate_cw();
        self.stickers[20] = temp1;
        self.stickers[20].rotate_cw();
        self.stickers[20].rotate_cw();
    }

    /// R面を反時計回りに回転
    fn rotate_rp(&mut self) {
        self.rotate_face_ccw(12, 3); // Right face (orient +3)

        let temp0 = self.stickers[1];
        let temp1 = self.stickers[3];

        // U <- B (+2)
        self.stickers[1] = self.stickers[22];
        self.stickers[1].rotate_cw();
        self.stickers[1].rotate_cw();
        self.stickers[3] = self.stickers[20];
        self.stickers[3].rotate_cw();
        self.stickers[3].rotate_cw();

        // B <- D (+2)
        self.stickers[22] = self.stickers[5];
        self.stickers[22].rotate_cw();
        self.stickers[22].rotate_cw();
        self.stickers[20] = self.stickers[7];
        self.stickers[20].rotate_cw();
        self.stickers[20].rotate_cw();

        // D <- F (+0)
        self.stickers[5] = self.stickers[17];
        self.stickers[7] = self.stickers[19];

        // F <- U (+0)
        self.stickers[17] = temp0;
        self.stickers[19] = temp1;
    }

    /// L面を時計回りに回転
    fn rotate_l(&mut self) {
        self.rotate_face_cw(8, 3); // Left face (orient +3)

        let temp0 = self.stickers[0];
        let temp1 = self.stickers[2];

        // U <- B [+2]
        self.stickers[0] = self.stickers[23];
        self.stickers[0].rotate_cw();
        self.stickers[0].rotate_cw();
        self.stickers[2] = self.stickers[21];
        self.stickers[2].rotate_cw();
        self.stickers[2].rotate_cw();

        // B <- D [+2]
        self.stickers[23] = self.stickers[4];
        self.stickers[23].rotate_cw();
        self.stickers[23].rotate_cw();
        self.stickers[21] = self.stickers[6];
        self.stickers[21].rotate_cw();
        self.stickers[21].rotate_cw();

        // D <- F [+0]
        self.stickers[4] = self.stickers[16];
        self.stickers[6] = self.stickers[18];

        // F <- U [+0]
        self.stickers[16] = temp0;
        self.stickers[18] = temp1;
    }

    /// L面を反時計回りに回転
    fn rotate_lp(&mut self) {
        self.rotate_face_ccw(8, 3); // Left face (orient +3)

        let temp0 = self.stickers[0];
        let temp1 = self.stickers[2];

        // U <- F [+0]
        self.stickers[0] = self.stickers[16];
        self.stickers[2] = self.stickers[18];

        // F <- D [+0]
        self.stickers[16] = self.stickers[4];
        self.stickers[18] = self.stickers[6];

        // D <- B [+2]
        self.stickers[4] = self.stickers[23];
        self.stickers[4].rotate_cw();
        self.stickers[4].rotate_cw();
        self.stickers[6] = self.stickers[21];
        self.stickers[6].rotate_cw();
        self.stickers[6].rotate_cw();

        // B <- U [+2]
        self.stickers[23] = temp0;
        self.stickers[23].rotate_cw();
        self.stickers[23].rotate_cw();
        self.stickers[21] = temp1;
        self.stickers[21].rotate_cw();
        self.stickers[21].rotate_cw();
    }

    /// U面を時計回りに回転
    fn rotate_u(&mut self) {
        self.rotate_face_cw(0, 1); // Up face (orient +1)

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
        self.rotate_face_ccw(0, 1); // Up face (orient +1)

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
        self.rotate_face_cw(4, 1); // Down face (orient +1)

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
        self.rotate_face_ccw(4, 1); // Down face (orient +1)

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
        self.rotate_face_cw(16, 1); // Front face (orient +1)

        let temp0 = self.stickers[2];
        let temp1 = self.stickers[3];

        // U -> R -> D -> L -> U
        // F CW: idx 11->2 (L->U) orient 1. idx 2->12 (U->R) orient 3. idx 12->5 (R->D) orient 1. idx 5->11 (D->L) orient 3.
        self.stickers[2] = self.stickers[11];
        self.stickers[2].rotate_cw();
        self.stickers[3] = self.stickers[9];
        self.stickers[3].rotate_cw();

        self.stickers[11] = self.stickers[5];
        self.stickers[11].rotate_ccw();
        self.stickers[9] = self.stickers[4];
        self.stickers[9].rotate_ccw();

        self.stickers[5] = self.stickers[12];
        self.stickers[5].rotate_cw();
        self.stickers[4] = self.stickers[14];
        self.stickers[4].rotate_cw();

        let mut t0 = temp0;
        t0.rotate_ccw();
        let mut t1 = temp1;
        t1.rotate_ccw();
        self.stickers[12] = t0;
        self.stickers[14] = t1;
    }

    /// F面を反時計回りに回転
    fn rotate_fp(&mut self) {
        self.rotate_face_ccw(16, 1); // Front face (orient +1)

        let temp0 = self.stickers[2];
        let temp1 = self.stickers[3];

        // U -> L -> D -> R -> U
        // Fp CCW: idx 12->2 (R->U) orient 1. idx 2->11 (U->L) orient 3. idx 11->5 (L->D) orient 1. idx 5->12 (D->R) orient 3.
        self.stickers[2] = self.stickers[12];
        self.stickers[2].rotate_cw();
        self.stickers[3] = self.stickers[14];
        self.stickers[3].rotate_cw();

        self.stickers[12] = self.stickers[5];
        self.stickers[12].rotate_ccw();
        self.stickers[14] = self.stickers[4];
        self.stickers[14].rotate_ccw();

        self.stickers[5] = self.stickers[11];
        self.stickers[5].rotate_cw();
        self.stickers[4] = self.stickers[9];
        self.stickers[4].rotate_cw();

        let mut t0 = temp0;
        t0.rotate_ccw();
        let mut t1 = temp1;
        t1.rotate_ccw();
        self.stickers[11] = t1; // Correct index: U[3] moves to L-BL=11
        self.stickers[9] = t0; // Correct index: U[2] moves to L-TL=9
    }

    /// B面を時計回りに回転
    fn rotate_b(&mut self) {
        self.rotate_face_cw(20, 1); // Back face (orient +1)

        let mut temp0 = self.stickers[0];
        let mut temp1 = self.stickers[1];

        // U -> L -> D -> R -> U (CW from back)
        // B CW: idx 13->0 (R->U) orient 3. idx 0->10 (U->L) orient 1. idx 10->7 (L->D) orient 3. idx 7->13 (D->R) orient 1.
        self.stickers[0] = self.stickers[13];
        self.stickers[0].rotate_ccw(); // U <- R (+3)
        self.stickers[1] = self.stickers[15];
        self.stickers[1].rotate_ccw(); // U <- R (+3)

        self.stickers[13] = self.stickers[7];
        self.stickers[13].rotate_cw(); // R <- D (+1)
        self.stickers[15] = self.stickers[6];
        self.stickers[15].rotate_cw(); // R <- D (+1)

        self.stickers[7] = self.stickers[10];
        self.stickers[7].rotate_ccw(); // D <- L (+3)
        self.stickers[6] = self.stickers[8];
        self.stickers[6].rotate_ccw(); // D <- L (+3)

        temp0.rotate_cw(); // L <- U (+1)
        temp1.rotate_cw(); // L <- U (+1)
        self.stickers[10] = temp0;
        self.stickers[8] = temp1;
    }

    /// B面を反時計回りに回転
    fn rotate_bp(&mut self) {
        self.rotate_face_ccw(20, 1); // Back face (orient +1)

        let temp0 = self.stickers[0];
        let temp1 = self.stickers[1];

        // U -> R -> D -> L -> U (CCW from back)
        // B CW:  U<-R(3), R<-D(1), D<-L(3), L<-U(1)
        // Bp CCW: U<-L(3), L<-D(1), D<-R(3), R<-U(1)

        // U <- L (+3)
        self.stickers[0] = self.stickers[10];
        self.stickers[0].rotate_ccw();
        self.stickers[1] = self.stickers[8];
        self.stickers[1].rotate_ccw();

        // L <- D (+1)
        self.stickers[10] = self.stickers[7];
        self.stickers[10].rotate_cw();
        self.stickers[8] = self.stickers[6];
        self.stickers[8].rotate_cw();

        // D <- R (+3)
        self.stickers[7] = self.stickers[13];
        self.stickers[7].rotate_ccw();
        self.stickers[6] = self.stickers[15];
        self.stickers[6].rotate_ccw();

        // R <- U (+1)
        self.stickers[13] = temp0;
        self.stickers[13].rotate_cw();
        self.stickers[15] = temp1;
        self.stickers[15].rotate_cw();
    }

    /// ランダムなスクランブルを生成します。
    ///
    /// # 引数
    ///
    /// - `moves`: 適用するランダム操作の回数
    ///
    /// # 例
    ///
    /// ```
    /// use rubiks_cube_2x2::cube::Cube;
    ///
    /// let mut cube = Cube::new();
    /// cube.scramble(20);
    /// assert!(!cube.is_solved()); // ほぼ確実に未完成状態になる
    /// ```
    pub fn scramble(&mut self, moves: usize) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let all_moves = Move::all_moves();

        for _ in 0..moves {
            let mv = all_moves[rng.gen_range(0..all_moves.len())];
            self.apply_move(mv);
        }
    }

    /// 色情報のみ比較するために、向き情報をリセットしたキューブを返します。
    ///
    /// すべてのステッカーの向き（orientation）を0にリセットした新しいキューブを返します。
    /// 色の配置は元のキューブと同じです。
    ///
    /// # 戻り値
    ///
    /// 向き情報がリセットされた新しいキューブ
    ///
    /// # 例
    ///
    /// ```
    /// use rubiks_cube_2x2::cube::{Cube, Move};
    ///
    /// let mut cube = Cube::new();
    /// cube.apply_move(Move::R);
    /// let normalized = cube.normalized();
    /// // normalized は色配置は同じだが、すべての向きが0
    /// ```
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

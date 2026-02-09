use crate::cube::{Color, Cube, Face, STICKERS_PER_FACE};
use std::sync::OnceLock;

/// Kociemba の FaceCube 表現
pub struct FaceCube {
    pub f: [Color; 54],
}

/// コーナーピース (Kociemba順: 0:UFR, 1:UFL, 2:ULB, 3:UBR, 4:DFR, 5:DLF, 6:DBL, 7:DRB)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Corner {
    UFR = 0,
    UFL,
    ULB,
    UBR,
    DFR,
    DLF,
    DBL,
    DRB,
}

/// エッジピース (Kociemba順: 0:UR, 1:UF, 2:UL, 3:UB, 4:DR, 5:DF, 6:DL, 7:DB, 8:FR, 9:FL, 10:BL, 11:BR)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    UR = 0,
    UF,
    UL,
    UB,
    DR,
    DF,
    DL,
    DB,
    FR,
    FL,
    BL,
    BR,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawCube {
    pub cp: [Corner; 8],
    pub co: [u8; 8],
    pub ep: [Edge; 12],
    pub eo: [u8; 12],
}

/// Kociemba アルゴリズムで使用する座標表現
#[derive(Debug, Clone, Default)]
pub struct CoordCube {
    pub twist: u16,    // コーナーの向き (0..2186)
    pub flip: u16,     // エッジの向き (0..2047)
    pub ud_slice: u16, // 中層エッジの位置 (0..494)
    pub cp: u16,       // コーナーの置換 (0..40319)
    pub ep8: u16,      // U/D面エッジの置換 (0..40319)
    pub slice_p: u16,  // 中層エッジの置換 (0..23)
}

impl RawCube {
    /// 2つの RawCube を合成する (A.multiply(B) は、状態 A に操作 B を適用した新しい状態を返す)
    pub fn multiply(&self, other: &RawCube) -> Self {
        let mut res = RawCube::default();
        for i in 0..8 {
            res.cp[i] = self.cp[other.cp[i] as usize];
            res.co[i] = (self.co[other.cp[i] as usize] + other.co[i]) % 3;
        }
        for i in 0..12 {
            res.ep[i] = self.ep[other.ep[i] as usize];
            res.eo[i] = (self.eo[other.ep[i] as usize] + other.eo[i]) % 2;
        }
        res
    }

    /// 基本操作（U, R, F, D, L, B）の RawCube 表現を取得する
    /// cp[i] = ピースがどこから来たか (流入元スロット)
    pub fn move_cube(mv: usize) -> &'static RawCube {
        static MOVE_CUBES: OnceLock<[RawCube; 6]> = OnceLock::new();
        &MOVE_CUBES.get_or_init(|| {
            [
                // 0: U CW (B->R->F->L)
                RawCube {
                    cp: [
                        Corner::UBR,
                        Corner::UFR,
                        Corner::UFL,
                        Corner::ULB,
                        Corner::DFR,
                        Corner::DLF,
                        Corner::DBL,
                        Corner::DRB,
                    ],
                    co: [0, 0, 0, 0, 0, 0, 0, 0],
                    ep: [
                        Edge::UB,
                        Edge::UR,
                        Edge::UF,
                        Edge::UL,
                        Edge::DR,
                        Edge::DF,
                        Edge::DL,
                        Edge::DB,
                        Edge::FR,
                        Edge::FL,
                        Edge::BL,
                        Edge::BR,
                    ],
                    eo: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                },
                // 1: R CW (U->B->D->F)
                RawCube {
                    cp: [
                        Corner::DFR,
                        Corner::UFL,
                        Corner::ULB,
                        Corner::UFR,
                        Corner::DRB,
                        Corner::DLF,
                        Corner::DBL,
                        Corner::UBR,
                    ],
                    co: [1, 0, 0, 2, 2, 0, 0, 1],
                    ep: [
                        Edge::FR,
                        Edge::UF,
                        Edge::UL,
                        Edge::UB,
                        Edge::BR,
                        Edge::DF,
                        Edge::DL,
                        Edge::DB,
                        Edge::DR,
                        Edge::FL,
                        Edge::BL,
                        Edge::UR,
                    ],
                    eo: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                },
                // 2: F CW (U->R->D->L)
                RawCube {
                    cp: [
                        Corner::UFL,
                        Corner::DLF,
                        Corner::ULB,
                        Corner::UBR,
                        Corner::UFR,
                        Corner::DFR,
                        Corner::DBL,
                        Corner::DRB,
                    ],
                    co: [2, 1, 0, 0, 1, 2, 0, 0],
                    ep: [
                        Edge::UR,
                        Edge::FL,
                        Edge::UL,
                        Edge::UB,
                        Edge::DR,
                        Edge::FR,
                        Edge::DL,
                        Edge::DB,
                        Edge::UF,
                        Edge::DF,
                        Edge::BL,
                        Edge::BR,
                    ],
                    eo: [0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0],
                },
                // 3: D CW (F->R->B->L)
                RawCube {
                    cp: [
                        Corner::UFR,
                        Corner::UFL,
                        Corner::ULB,
                        Corner::UBR,
                        Corner::DLF,
                        Corner::DBL,
                        Corner::DRB,
                        Corner::DFR,
                    ],
                    co: [0, 0, 0, 0, 0, 0, 0, 0],
                    ep: [
                        Edge::UR,
                        Edge::UF,
                        Edge::UL,
                        Edge::UB,
                        Edge::DF,
                        Edge::DL,
                        Edge::DB,
                        Edge::DR,
                        Edge::FR,
                        Edge::FL,
                        Edge::BL,
                        Edge::BR,
                    ],
                    eo: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                },
                // 4: L CW (U->F->D->B)
                RawCube {
                    cp: [
                        Corner::UFR,
                        Corner::ULB,
                        Corner::DBL,
                        Corner::UBR,
                        Corner::DFR,
                        Corner::UFL,
                        Corner::DLF,
                        Corner::DRB,
                    ],
                    co: [0, 2, 1, 0, 0, 1, 2, 0],
                    ep: [
                        Edge::UR,
                        Edge::UF,
                        Edge::BL,
                        Edge::UB,
                        Edge::DR,
                        Edge::DF,
                        Edge::FL,
                        Edge::DB,
                        Edge::FR,
                        Edge::UL,
                        Edge::DL,
                        Edge::BR,
                    ],
                    eo: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                },
                // 5: B CW (U->L->D->R)
                RawCube {
                    cp: [
                        Corner::UFR,
                        Corner::UFL,
                        Corner::UBR,
                        Corner::DRB,
                        Corner::DFR,
                        Corner::DLF,
                        Corner::ULB,
                        Corner::DBL,
                    ],
                    co: [0, 0, 2, 1, 0, 0, 1, 2],
                    ep: [
                        Edge::UR,
                        Edge::UF,
                        Edge::UL,
                        Edge::BR,
                        Edge::DR,
                        Edge::DF,
                        Edge::DL,
                        Edge::BL,
                        Edge::FR,
                        Edge::FL,
                        Edge::UB,
                        Edge::DB,
                    ],
                    eo: [0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1],
                },
            ]
        })[mv]
    }

    pub fn from_cube(cube: &Cube) -> Result<RawCube, String> {
        let u_color = cube.stickers[4].color;
        let d_color = cube.stickers[13].color;
        let l_color = cube.stickers[22].color;
        let r_color = cube.stickers[31].color;
        let f_color = cube.stickers[40].color;
        let b_color = cube.stickers[49].color;

        let mut rc = RawCube::default();
        use crate::cube::validation::{CORNER_STICKERS, EDGE_STICKERS};
        for (i, &facelets) in CORNER_STICKERS.iter().enumerate() {
            let c1 = cube.stickers[facelets[0]].color; // Primary (U/D)
            let c2 = cube.stickers[facelets[1]].color; // Side 1
            let c3 = cube.stickers[facelets[2]].color; // Side 2

            // Corner Enum: 0:UFR, 1:UFL, 2:ULB, 3:UBR, 4:DFR, 5:DLF, 6:DBL, 7:DRB
            rc.cp[i] = match (c1, c2, c3) {
                // slot i にあるピースの色の組み合わせ (U/D, Side1, Side2)

                // U-Corners (Primary: White/U)
                (c, f, r) if c == u_color && f == f_color && r == r_color => Corner::UFR,
                (c, l, f) if c == u_color && l == l_color && f == f_color => Corner::UFL,
                (c, b, l) if c == u_color && b == b_color && l == l_color => Corner::ULB,
                (c, r, b) if c == u_color && r == r_color && b == b_color => Corner::UBR,

                // D-Corners (Primary: Yellow/D)
                (c, r, f) if c == d_color && r == r_color && f == f_color => Corner::DFR,
                (c, f, l) if c == d_color && f == f_color && l == l_color => Corner::DLF,
                (c, l, b) if c == d_color && l == l_color && b == b_color => Corner::DBL,
                (c, b, r) if c == d_color && b == b_color && r == r_color => Corner::DRB,

                // Twisted Ori 1
                (f, r, c) if c == u_color && f == f_color && r == r_color => Corner::UFR,
                (l, f, c) if c == u_color && l == l_color && f == f_color => Corner::UFL,
                (b, l, c) if c == u_color && b == b_color && l == l_color => Corner::ULB,
                (r, b, c) if c == u_color && r == r_color && b == b_color => Corner::UBR,
                (r, f, c) if c == d_color && r == r_color && f == f_color => Corner::DFR,
                (f, l, c) if c == d_color && f == f_color && l == l_color => Corner::DLF,
                (l, b, c) if c == d_color && l == l_color && b == b_color => Corner::DBL,
                (b, r, c) if c == d_color && b == b_color && r == r_color => Corner::DRB,

                // Twisted Ori 2
                (r, c, f) if c == u_color && f == f_color && r == r_color => Corner::UFR,
                (f, c, l) if c == u_color && l == l_color && f == f_color => Corner::UFL,
                (l, c, b) if c == u_color && b == b_color && l == l_color => Corner::ULB,
                (b, c, r) if c == u_color && r == r_color && b == b_color => Corner::UBR,
                (f, c, r) if c == d_color && r == r_color && f == f_color => Corner::DFR,
                (l, c, f) if c == d_color && f == f_color && l == l_color => Corner::DLF,
                (b, c, l) if c == d_color && l == l_color && b == b_color => Corner::DBL,
                (r, c, b) if c == d_color && b == b_color && r == r_color => Corner::DRB,

                _ => {
                    return Err(format!(
                        "Invalid corner colors at index {}: c1={:?}, c2={:?}, c3={:?}, centers(U={:?}, D={:?}, L={:?}, R={:?}, F={:?}, B={:?})",
                        i, c1, c2, c3, u_color, d_color, l_color, r_color, f_color, b_color
                    ))
                }
            };

            // Orientation calculation
            rc.co[i] = if c1 == u_color || c1 == d_color {
                0
            } else if c2 == u_color || c2 == d_color {
                1
            } else {
                2
            };
        }

        for (i, &facelets) in EDGE_STICKERS.iter().enumerate() {
            let color0 = cube.stickers[facelets[0]].color;
            let color1 = cube.stickers[facelets[1]].color;

            let ori = if color0 == u_color || color0 == d_color {
                0
            } else if color1 == u_color || color1 == d_color {
                1
            } else if color0 == f_color || color0 == b_color {
                0
            } else if color1 == f_color || color1 == b_color {
                1
            } else {
                0
            };

            let c1 = cube.stickers[facelets[ori as usize]].color;
            let c2 = cube.stickers[facelets[1 - ori as usize]].color;
            rc.ep[i] = match (c1, c2) {
                (c, r) if (c == u_color && r == r_color) || (c == r_color && r == u_color) => {
                    Edge::UR
                }
                (c, f) if (c == u_color && f == f_color) || (c == f_color && f == u_color) => {
                    Edge::UF
                }
                (c, l) if (c == u_color && l == l_color) || (c == l_color && l == u_color) => {
                    Edge::UL
                }
                (c, b) if (c == u_color && b == b_color) || (c == b_color && b == u_color) => {
                    Edge::UB
                }
                (c, r) if (c == d_color && r == r_color) || (c == r_color && r == d_color) => {
                    Edge::DR
                }
                (c, f) if (c == d_color && f == f_color) || (c == f_color && f == d_color) => {
                    Edge::DF
                }
                (c, l) if (c == d_color && l == l_color) || (c == l_color && l == d_color) => {
                    Edge::DL
                }
                (c, b) if (c == d_color && b == b_color) || (c == b_color && b == d_color) => {
                    Edge::DB
                }
                (r, f) if (r == r_color && f == f_color) || (r == f_color && f == r_color) => {
                    Edge::FR
                }
                (l, f) if (l == l_color && f == f_color) || (l == f_color && f == l_color) => {
                    Edge::FL
                }
                (l, b) if (l == l_color && b == b_color) || (l == b_color && b == l_color) => {
                    Edge::BL
                }
                (r, b) if (r == r_color && b == b_color) || (r == b_color && b == r_color) => {
                    Edge::BR
                }
                _ => {
                    tracing::warn!(
                        "Edge identification failure at index {}: c1={:?}, c2={:?}, expected U={:?}, D={:?}, L={:?}, R={:?}, F={:?}, B={:?}",
                        i, c1, c2, u_color, d_color, l_color, r_color, f_color, b_color
                    );
                    return Err(format!(
                        "Invalid edge colors at index {}: {:?}, {:?}",
                        i, c1, c2
                    ));
                }
            };
            rc.eo[i] = ori;
        }

        Ok(rc)
    }

    // --- 座標変換メソッド ---

    /// Twist (コーナーの向き) を取得 (0..2186)
    pub fn get_twist(&self) -> u16 {
        let mut twist = 0u16;
        for i in 0..7 {
            twist = twist * 3 + self.co[i] as u16;
        }
        twist
    }

    /// Twist を設定 (最後のコーナーの向きはパリティで決定)
    pub fn set_twist(&mut self, mut twist: u16) {
        let mut twist_parity = 0;
        for i in (0..7).rev() {
            self.co[i] = (twist % 3) as u8;
            twist_parity += self.co[i];
            twist /= 3;
        }
        self.co[7] = (3 - (twist_parity % 3)) % 3;
    }

    /// Flip (エッジの向き) を取得 (0..2047)
    pub fn get_flip(&self) -> u16 {
        let mut flip = 0u16;
        for i in 0..11 {
            flip = flip * 2 + self.eo[i] as u16;
        }
        flip
    }

    /// Flip を設定 (最後のエッジの向きはパリティで決定)
    pub fn set_flip(&mut self, mut flip: u16) {
        let mut flip_parity = 0;
        for i in (0..11).rev() {
            self.eo[i] = (flip % 2) as u8;
            flip_parity += self.eo[i];
            flip /= 2;
        }
        self.eo[11] = (2 - (flip_parity % 2)) % 2;
    }

    /// UDSlice (中層エッジの位置) を取得 (0..494)
    /// 12個のエッジのうち、8(FR), 9(FL), 10(BL), 11(BR) がどの位置にあるかの組合せ (Solved=0)
    pub fn get_ud_slice(&self) -> u16 {
        let mut res = 0;
        let mut k = 4;
        for i in (0..12).rev() {
            if (self.ep[i] as u8) >= 8 {
                k -= 1;
            } else if k > 0 {
                res += n_choose_k(i as i16, (k - 1) as i16);
            }
        }
        res
    }

    /// UDSlice を設定 (中層エッジの位置のみ設定、具体的なエッジの種類は任意)
    pub fn set_ud_slice(&mut self, ud_slice: u16) {
        let mut k = 4;
        let mut s = ud_slice;
        for i in (0..12).rev() {
            if k > 0 && s >= n_choose_k(i as i16, (k - 1) as i16) {
                s -= n_choose_k(i as i16, (k - 1) as i16);
                self.ep[i] = Edge::UR; // placeholder
            } else if k > 0 {
                self.ep[i] = Edge::FR; // placeholder
                k -= 1;
            } else {
                self.ep[i] = Edge::UR;
            }
        }
    }

    /// コーナーの置換をインデックス化 (0..40319)
    pub fn get_cp(&self) -> u16 {
        let mut cp = 0u32;
        let p = self.cp.map(|c| c as u8).to_vec();
        for i in 0..7 {
            let mut k = 0;
            for j in (i + 1)..8 {
                if p[j] < p[i] {
                    k += 1;
                }
            }
            cp = cp * (8 - i as u32) + k as u32;
        }
        cp as u16
    }

    /// コーナーの置換を設定
    pub fn set_cp(&mut self, cp: u16) {
        let mut available = (0..8).collect::<Vec<u8>>();
        let mut cp_u32 = cp as u32;
        let mut res = [0u8; 8];
        for (i, item) in res.iter_mut().enumerate().take(7) {
            let fact = factorial(7 - i as u8);
            let idx = (cp_u32 / fact) as usize;
            *item = available.remove(idx);
            cp_u32 %= fact;
        }
        res[7] = available[0];
        for (i, &item) in res.iter().enumerate() {
            // SAFETY: res[i] は 0..7 の範囲内の値で、Corner enum の判別値と一致するため、transmute は安全
            self.cp[i] = unsafe { std::mem::transmute::<u8, Corner>(item) };
        }
    }

    /// U/D面エッジの置換をインデックス化 (Phase 2, 0..40319)
    pub fn get_ep8(&self) -> u16 {
        let mut ep8 = 0u32;
        let mut p = [0u8; 8];
        let mut count = 0;
        for i in 0..12 {
            if (self.ep[i] as u8) < 8 {
                p[count] = self.ep[i] as u8;
                count += 1;
            }
        }
        for i in 0..7 {
            let mut k = 0;
            for j in (i + 1)..8 {
                if p[j] < p[i] {
                    k += 1;
                }
            }
            ep8 = ep8 * (8 - i as u32) + k as u32;
        }
        ep8 as u16
    }

    /// U/D面エッジの置換を設定
    pub fn set_ep8(&mut self, ep8: u16) {
        let mut available = (0..8).collect::<Vec<u8>>();
        let mut ep8_u32 = ep8 as u32;
        let mut res = [0u8; 8];
        for (i, item) in res.iter_mut().enumerate().take(7) {
            let fact = factorial(7 - i as u8);
            let idx = (ep8_u32 / fact) as usize;
            *item = available.remove(idx);
            ep8_u32 %= fact;
        }
        res[7] = available[0];
        // U/D面の8箇所に配置
        let mut count = 0;
        for i in 0..12 {
            if (self.ep[i] as u8) < 8 {
                // SAFETY: res[count] は 0..7 の範囲内の値で、Edge enum の判別値 (0..7) と一致するため、transmute は安全
                self.ep[i] = unsafe { std::mem::transmute::<u8, Edge>(res[count]) };
                count += 1;
            }
        }
    }

    /// 中層エッジの置換をインデックス化 (Phase 2, 0..23)
    pub fn get_slice_p(&self) -> u16 {
        let mut slice_p = 0u32;
        let mut p = [0u8; 4];
        let mut count = 0;
        for i in 0..12 {
            if (self.ep[i] as u8) >= 8 {
                p[count] = self.ep[i] as u8 - 8;
                count += 1;
            }
        }
        for i in 0..3 {
            let mut k = 0;
            for j in (i + 1)..4 {
                if p[j] < p[i] {
                    k += 1;
                }
            }
            slice_p = slice_p * (4 - i as u32) + k as u32;
        }
        slice_p as u16
    }

    /// 中層エッジの置換を設定
    pub fn set_slice_p(&mut self, slice_p: u16) {
        let mut available = (0..4).collect::<Vec<u8>>();
        let mut slice_p_u32 = slice_p as u32;
        let mut res = [0u8; 4];
        for (i, item) in res.iter_mut().enumerate().take(3) {
            let fact = factorial(3 - i as u8);
            let idx = (slice_p_u32 / fact) as usize;
            *item = available.remove(idx);
            slice_p_u32 %= fact;
        }
        res[3] = available[0];
        let mut count = 0;
        for i in 0..12 {
            if (self.ep[i] as u8) >= 8 {
                // SAFETY: res[count] + 8 は 8..11 の範囲内の値で、Edge enum の判別値 (8:FR, 9:FL, 10:BL, 11:BR) と一致するため、transmute は安全
                self.ep[i] = unsafe { std::mem::transmute::<u8, Edge>(res[count] + 8) };
                count += 1;
            }
        }
    }
}

// --- 数学ユーティリティ ---

fn n_choose_k(n: i16, mut k: i16) -> u16 {
    if n < k || k < 0 {
        return 0;
    }
    if k > n / 2 {
        k = n - k;
    }
    let mut res = 1u32;
    for i in 1..=k {
        res = res * (n - i + 1) as u32 / i as u32;
    }
    res as u16
}

fn factorial(n: u8) -> u32 {
    let mut res = 1u32;
    for i in 2..=n {
        res *= i as u32;
    }
    res
}

pub fn move_cube_18(mv_idx: usize) -> &'static RawCube {
    static MOVE_CUBES_18: OnceLock<[RawCube; 18]> = OnceLock::new();
    &MOVE_CUBES_18.get_or_init(|| {
        let mut moves = [RawCube::default(); 18];
        for mv in 0..6 {
            let base = RawCube::move_cube(mv);
            moves[mv * 3] = *base; // CW (Copy)
            moves[mv * 3 + 1] = base.multiply(base); // 2
            moves[mv * 3 + 2] = base.multiply(base).multiply(base); // CCW
        }
        moves
    })[mv_idx]
}

impl Default for RawCube {
    fn default() -> Self {
        Self {
            cp: [
                Corner::UFR,
                Corner::UFL,
                Corner::ULB,
                Corner::UBR,
                Corner::DFR,
                Corner::DLF,
                Corner::DBL,
                Corner::DRB,
            ],
            co: [0; 8],
            ep: [
                Edge::UR,
                Edge::UF,
                Edge::UL,
                Edge::UB,
                Edge::DR,
                Edge::DF,
                Edge::DL,
                Edge::DB,
                Edge::FR,
                Edge::FL,
                Edge::BL,
                Edge::BR,
            ],
            eo: [0; 12],
        }
    }
}

impl FaceCube {
    pub fn from_cube(cube: &Cube) -> Self {
        let mut f = [Color::White; 54];
        let face_map = [
            (Face::Up, 0),
            (Face::Right, 9),
            (Face::Front, 18),
            (Face::Down, 27),
            (Face::Left, 36),
            (Face::Back, 45),
        ];
        for (src_face, dst_offset) in face_map {
            let src_start = src_face.start_index();
            for i in 0..STICKERS_PER_FACE {
                f[dst_offset + i] = cube.stickers[src_start + i].color;
            }
        }
        Self { f }
    }
}

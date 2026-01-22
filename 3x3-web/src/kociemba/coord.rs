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
                // 0: U CW
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
                // 1: R CW
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
                    eo: [1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1],
                },
                // 2: F CW
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
                    eo: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                },
                // 3: D CW
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
                // 4: L CW
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
                    eo: [0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0],
                },
                // 5: B CW
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
                    eo: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                },
            ]
        })[mv]
    }

    pub fn from_cube(cube: &Cube) -> Result<RawCube, String> {
        let mut rc = RawCube::default();
        use crate::cube::validation::{CORNER_STICKERS, EDGE_STICKERS};
        for i in 0..8 {
            let facelets = CORNER_STICKERS[i];
            let mut ori = 0;
            let mut found = false;
            for (o, &f) in facelets.iter().enumerate() {
                let color = cube.stickers[f].color;
                if color == Color::White || color == Color::Yellow {
                    ori = o as u8;
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(format!("No primary color at corner {}", i));
            }

            let c1 = cube.stickers[facelets[ori as usize]].color;
            let c2 = cube.stickers[facelets[(ori as usize + 1) % 3]].color;
            let c3 = cube.stickers[facelets[(ori as usize + 2) % 3]].color;
            rc.cp[i] = match (c1, c2, c3) {
                (Color::White, Color::Green, Color::Red) => Corner::UFR,
                (Color::White, Color::Orange, Color::Green) => Corner::UFL,
                (Color::White, Color::Blue, Color::Orange) => Corner::ULB,
                (Color::White, Color::Red, Color::Blue) => Corner::UBR,
                (Color::Yellow, Color::Red, Color::Green) => Corner::DFR,
                (Color::Yellow, Color::Green, Color::Orange) => Corner::DLF,
                (Color::Yellow, Color::Orange, Color::Blue) => Corner::DBL,
                (Color::Yellow, Color::Blue, Color::Red) => Corner::DRB,
                _ => {
                    return Err(format!(
                        "Invalid corner colors at index {}: {:?}, {:?}, {:?}",
                        i, c1, c2, c3
                    ))
                }
            };
            rc.co[i] = ori;
        }

        for i in 0..12 {
            let facelets = EDGE_STICKERS[i];
            let color0 = cube.stickers[facelets[0]].color;
            let color1 = cube.stickers[facelets[1]].color;

            let ori = if color0 == Color::White || color0 == Color::Yellow {
                0
            } else if color1 == Color::White || color1 == Color::Yellow {
                1
            } else if color0 == Color::Red || color0 == Color::Orange {
                0
            } else if color1 == Color::Red || color1 == Color::Orange {
                1
            } else {
                0
            };

            let c1 = cube.stickers[facelets[ori as usize]].color;
            let c2 = cube.stickers[facelets[1 - ori as usize]].color;
            rc.ep[i] = match (c1, c2) {
                (Color::White, Color::Red) | (Color::Red, Color::White) => Edge::UR,
                (Color::White, Color::Green) | (Color::Green, Color::White) => Edge::UF,
                (Color::White, Color::Orange) | (Color::Orange, Color::White) => Edge::UL,
                (Color::White, Color::Blue) | (Color::Blue, Color::White) => Edge::UB,
                (Color::Yellow, Color::Red) | (Color::Red, Color::Yellow) => Edge::DR,
                (Color::Yellow, Color::Green) | (Color::Green, Color::Yellow) => Edge::DF,
                (Color::Yellow, Color::Orange) | (Color::Orange, Color::Yellow) => Edge::DL,
                (Color::Yellow, Color::Blue) | (Color::Blue, Color::Yellow) => Edge::DB,
                (Color::Red, Color::Green) | (Color::Green, Color::Red) => Edge::FR,
                (Color::Orange, Color::Green) | (Color::Green, Color::Orange) => Edge::FL,
                (Color::Orange, Color::Blue) | (Color::Blue, Color::Orange) => Edge::BL,
                (Color::Red, Color::Blue) | (Color::Blue, Color::Red) => Edge::BR,
                _ => {
                    return Err(format!(
                        "Invalid edge colors at index {}: {:?}, {:?}",
                        i, c1, c2
                    ))
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
        for i in 0..7 {
            let fact = factorial(7 - i as u8);
            let idx = (cp_u32 / fact) as usize;
            res[i] = available.remove(idx);
            cp_u32 %= fact;
        }
        res[7] = available[0];
        for i in 0..8 {
            self.cp[i] = unsafe { std::mem::transmute(res[i]) };
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
        for i in 0..7 {
            let fact = factorial(7 - i as u8);
            let idx = (ep8_u32 / fact) as usize;
            res[i] = available.remove(idx);
            ep8_u32 %= fact;
        }
        res[7] = available[0];
        // U/D面の8箇所に配置
        let mut count = 0;
        for i in 0..12 {
            if (self.ep[i] as u8) < 8 {
                self.ep[i] = unsafe { std::mem::transmute(res[count]) };
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
        for i in 0..3 {
            let fact = factorial(3 - i as u8);
            let idx = (slice_p_u32 / fact) as usize;
            res[i] = available.remove(idx);
            slice_p_u32 %= fact;
        }
        res[3] = available[0];
        let mut count = 0;
        for i in 0..12 {
            if (self.ep[i] as u8) >= 8 {
                self.ep[i] = unsafe { std::mem::transmute(res[count] + 8) };
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
            moves[mv * 3] = base.clone(); // CW
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cube::Move;

    #[test]
    fn test_to_raw_cube_initial() {
        let cube = Cube::new();
        let rc = RawCube::from_cube(&cube).expect("Should convert");
        assert_eq!(rc.cp[0], Corner::UFR);
        assert_eq!(rc.ep[0], Edge::UR);
    }

    #[test]
    fn test_raw_cube_move_u() {
        let mut cube = Cube::new();
        crate::cube::rotation::apply_move(&mut cube, Move::U);
        let rc_from_cube = RawCube::from_cube(&cube).unwrap();

        let identity = RawCube::default();
        let rc_from_move = identity.multiply(RawCube::move_cube(0));

        assert_eq!(rc_from_cube.cp, rc_from_move.cp, "CP mismatch for U");
        assert_eq!(rc_from_cube.co, rc_from_move.co, "CO mismatch for U");
        assert_eq!(rc_from_cube.ep, rc_from_move.ep, "EP mismatch for U");
        assert_eq!(rc_from_cube.eo, rc_from_move.eo, "EO mismatch for U");
    }

    #[test]
    fn test_coordinates_initial() {
        let rc = RawCube::default();
        assert_eq!(rc.get_twist(), 0);
        assert_eq!(rc.get_flip(), 0);
        assert_eq!(rc.get_ud_slice(), 0); // Corrected to 0
        assert_eq!(rc.get_cp(), 0);
        assert_eq!(rc.get_ep8(), 0);
        assert_eq!(rc.get_slice_p(), 0);
    }

    #[test]
    fn test_twist_symmetry() {
        let mut rc = RawCube::default();
        for twist in 0..2187 {
            rc.set_twist(twist);
            assert_eq!(rc.get_twist(), twist, "Twist failed at {}", twist);
        }
    }

    #[test]
    fn test_flip_symmetry() {
        let mut rc = RawCube::default();
        for flip in 0..2048 {
            rc.set_flip(flip);
            assert_eq!(rc.get_flip(), flip, "Flip failed at {}", flip);
        }
    }

    #[test]
    fn test_slice_symmetry() {
        let mut rc = RawCube::default();
        for slice in 0..495 {
            rc.set_ud_slice(slice);
            assert_eq!(rc.get_ud_slice(), slice, "Slice failed at {}", slice);
        }
    }

    #[test]
    fn test_raw_cube_all_basic_moves() {
        let moves = [Move::U, Move::R, Move::F, Move::D, Move::L, Move::B];
        for (i, &mv) in moves.iter().enumerate() {
            let mut cube = Cube::new();
            crate::cube::rotation::apply_move(&mut cube, mv);
            let rc_from_cube =
                RawCube::from_cube(&cube).expect(&format!("Convert fail for {:?}", mv));

            let identity = RawCube::default();
            let rc_from_move = identity.multiply(RawCube::move_cube(i));

            assert_eq!(rc_from_cube.cp, rc_from_move.cp, "CP mismatch for {:?}", mv);
            assert_eq!(rc_from_cube.co, rc_from_move.co, "CO mismatch for {:?}", mv);
            assert_eq!(rc_from_cube.ep, rc_from_move.ep, "EP mismatch for {:?}", mv);
            assert_eq!(rc_from_cube.eo, rc_from_move.eo, "EO mismatch for {:?}", mv);
        }
    }
}

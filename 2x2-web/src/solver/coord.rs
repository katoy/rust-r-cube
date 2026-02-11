use crate::cube::{Color, Cube, Move};
use std::sync::OnceLock;

/// コーナーピース (2x2)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Corner {
    UFL = 0,
    UFR,
    UBR,
    UBL,
    DFL,
    DFR,
    DBR,
    DBL,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawCube {
    pub cp: [Corner; 8],
    pub co: [u8; 8],
}

impl RawCube {
    pub fn multiply(&self, other: &RawCube) -> Self {
        let mut res = RawCube::default();
        for i in 0..8 {
            res.cp[i] = self.cp[other.cp[i] as usize];
            res.co[i] = (self.co[other.cp[i] as usize] + other.co[i]) % 3;
        }
        res
    }

    pub fn move_cube(mv: usize) -> &'static RawCube {
        static MOVE_CUBES: OnceLock<[RawCube; 6]> = OnceLock::new();
        &MOVE_CUBES.get_or_init(|| {
            let mut res = [RawCube::default(); 6];
            let moves = [Move::U, Move::D, Move::L, Move::R, Move::F, Move::B];
            for (i, &m) in moves.iter().enumerate() {
                let mut cube = Cube::new();
                crate::cube::rotation::apply_move(&mut cube, m);
                res[i] = RawCube::from_cube(&cube, &[0, 1, 2, 3, 4, 5]).expect("Move generation failed");
            }
            res
        })[mv]
    }

    pub fn from_cube(cube: &Cube, face_map: &[u8; 6]) -> Result<RawCube, String> {
        let u = Color::from_u8(face_map[0]);
        let d = Color::from_u8(face_map[1]);
        let l = Color::from_u8(face_map[2]);
        let r = Color::from_u8(face_map[3]);
        let f = Color::from_u8(face_map[4]);
        let b = Color::from_u8(face_map[5]);

        let mut rc = RawCube::default();
        use crate::cube::validation::CORNER_STICKERS;

        for (i, &facelets) in CORNER_STICKERS.iter().enumerate() {
            let colors = [
                cube.stickers[facelets[0]].color,
                cube.stickers[facelets[1]].color,
                cube.stickers[facelets[2]].color,
            ];

            let (piece, ori) =
                if let Some(p) = colors.iter().position(|&c| c == u || c == d) {
                    // Determine piece by color set
                    let piece = {
                        let mut sorted = colors;
                        sorted.sort_by_key(|c| *c as u8);

                        if set_match(sorted, [u, l, f]) {
                            Corner::UFL
                        } else if set_match(sorted, [u, f, r]) {
                            Corner::UFR
                        } else if set_match(sorted, [u, r, b]) {
                            Corner::UBR
                        } else if set_match(sorted, [u, b, l]) {
                            Corner::UBL
                        } else if set_match(sorted, [d, f, l]) {
                            Corner::DFL
                        } else if set_match(sorted, [d, r, f]) {
                            Corner::DFR
                        } else if set_match(sorted, [d, b, r]) {
                            Corner::DBR
                        } else if set_match(sorted, [d, l, b]) {
                            Corner::DBL
                        } else {
                            return Err(format!("Invalid corner set at {}: {:?}", i, colors));
                        }
                    };
                    (piece, p as u8)
                } else {
                    return Err(format!("No U/D color ({:?}/{:?}) at corner {}: {:?}", u, d, i, colors));
                };

            rc.cp[i] = piece;
            rc.co[i] = ori;
        }
        Ok(rc)
    }

    pub fn get_twist(&self) -> u16 {
        let mut twist = 0u16;
        for i in 0..7 {
            twist = twist * 3 + self.co[i] as u16;
        }
        twist
    }

    pub fn set_twist(&mut self, mut twist: u16) {
        let mut twist_parity = 0;
        for i in (0..7).rev() {
            self.co[i] = (twist % 3) as u8;
            twist_parity += self.co[i];
            twist /= 3;
        }
        self.co[7] = (3 - (twist_parity % 3)) % 3;
    }

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
            self.cp[i] = match item {
                0 => Corner::UFL,
                1 => Corner::UFR,
                2 => Corner::UBR,
                3 => Corner::UBL,
                4 => Corner::DFL,
                5 => Corner::DFR,
                6 => Corner::DBR,
                _ => Corner::DBL,
            };
        }
    }
}

fn set_match(a: [Color; 3], mut b: [Color; 3]) -> bool {
    b.sort_by_key(|c| *c as u8);
    a == b
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
            let r2 = base.multiply(base);
            let r3 = r2.multiply(base);
            moves[mv * 3] = *base;
            moves[mv * 3 + 1] = r2;
            moves[mv * 3 + 2] = r3;
        }
        moves
    })[mv_idx]
}

impl Default for RawCube {
    fn default() -> Self {
        Self {
            cp: [
                Corner::UFL,
                Corner::UFR,
                Corner::UBR,
                Corner::UBL,
                Corner::DFL,
                Corner::DFR,
                Corner::DBR,
                Corner::DBL,
            ],
            co: [0; 8],
        }
    }
}

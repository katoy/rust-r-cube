use std::sync::OnceLock;

/// 2x2 コーナーピース (Kociemba順準拠)
/// 0: UFR, 1: UFL, 2: ULB, 3: UBR, 4: DFR, 5: DLF, 6: DBL, 7: DRB
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Corner {
    UFR = 0,
    UFL = 1,
    ULB = 2,
    UBR = 3,
    DFR = 4,
    DLF = 5,
    DBL = 6,
    DRB = 7,
}

impl Corner {
    pub const ALL: [Corner; 8] = [
        Corner::UFR,
        Corner::UFL,
        Corner::ULB,
        Corner::UBR,
        Corner::DFR,
        Corner::DLF,
        Corner::DBL,
        Corner::DRB,
    ];

    #[inline]
    pub fn from_u8(v: u8) -> Self {
        match v & 7 {
            0 => Corner::UFR,
            1 => Corner::UFL,
            2 => Corner::ULB,
            3 => Corner::UBR,
            4 => Corner::DFR,
            5 => Corner::DLF,
            6 => Corner::DBL,
            _ => Corner::DRB,
        }
    }
}

/// 2x2 ルービックキューブの内部生表現
/// - cp: コーナーの配置 (Permutation: 0..7)
/// - co: コーナーの向き (Orientation: 0, 1, 2)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RawCube {
    pub cp: [Corner; 8],
    pub co: [u8; 8],
}

impl Default for RawCube {
    fn default() -> Self {
        Self {
            cp: Corner::ALL,
            co: [0; 8],
        }
    }
}

impl RawCube {
    /// 2つのキューブ操作を合成します。
    pub fn multiply(&self, other: &RawCube) -> Self {
        let mut res = RawCube::default();
        for i in 0..8 {
            res.cp[i] = self.cp[other.cp[i] as usize];
            res.co[i] = (self.co[other.cp[i] as usize] + other.co[i]) % 3;
        }
        res
    }

    /// CP (Corner Permutation: 0..40319) を取得します。
    pub fn get_cp(&self) -> u16 {
        let mut idx = 0;
        for i in 0..7 {
            idx *= 8 - i;
            for j in (i + 1)..8 {
                if (self.cp[i] as u8) > (self.cp[j] as u8) {
                    idx += 1;
                }
            }
        }
        idx as u16
    }

    /// CP (0..40319) からコーナー配置を設定します。
    pub fn set_cp(&mut self, idx: u16) {
        let mut val = idx as usize;
        let mut available = [0, 1, 2, 3, 4, 5, 6, 7];
        for i in 0..8 {
            let fact = (1..=(7 - i)).product::<usize>().max(1);
            let p = val / fact;
            val %= fact;
            self.cp[i] = Corner::from_u8(available[p]);
            for k in p..7 - i {
                available[k] = available[k + 1];
            }
        }
    }

    /// Twist (Corner Orientation: 0..2186) を取得します。
    pub fn get_twist(&self) -> u16 {
        let mut idx = 0;
        for i in 0..7 {
            idx = idx * 3 + self.co[i] as usize;
        }
        idx as u16
    }

    /// Twist (0..2186) からコーナー向きを設定します。
    pub fn set_twist(&mut self, idx: u16) {
        let mut val = idx as usize;
        let mut sum = 0;
        for i in (0..7).rev() {
            self.co[i] = (val % 3) as u8;
            sum += self.co[i];
            val /= 3;
        }
        self.co[7] = (3 - (sum % 3)) % 3;
    }
}

// ----------------------------------------------------------------------------
// 基本回転の定義 (18手)
// ----------------------------------------------------------------------------

/// 基本6面の90°時計回り回転 (U, R, F, D, L, B)
pub fn basic_move_cube(face: usize) -> RawCube {
    let mut rc = RawCube::default();
    match face {
        0 => {
            // U面: UFR(0) <- UBR(3) <- ULB(2) <- UFL(1) (時計回りサイクル: 0->3->2->1)
            rc.cp[0] = Corner::UBR;
            rc.cp[1] = Corner::UFR;
            rc.cp[2] = Corner::UFL;
            rc.cp[3] = Corner::ULB;
            // U面回転では各ピースのU面ステッカーは上面のままなので向き変化は0
        }
        1 => {
            // R面: UFR(0), UBR(3), DRB(7), DFR(4) のサイクル (0->4->7->3)
            rc.cp[0] = Corner::DFR;
            rc.co[0] = 2;
            rc.cp[3] = Corner::UFR;
            rc.co[3] = 1;
            rc.cp[4] = Corner::DRB;
            rc.co[4] = 1;
            rc.cp[7] = Corner::UBR;
            rc.co[7] = 2;
        }
        2 => {
            // F面: UFL(1), UFR(0), DFR(4), DLF(5) のサイクル (1->0->4->5)
            rc.cp[0] = Corner::UFL;
            rc.co[0] = 1;
            rc.cp[1] = Corner::DLF;
            rc.co[1] = 2;
            rc.cp[4] = Corner::UFR;
            rc.co[4] = 2;
            rc.cp[5] = Corner::DFR;
            rc.co[5] = 1;
        }
        3 => {
            // D面: DFR(4) <- DLF(5) <- DBL(6) <- DRB(7) (時計回りサイクル: 4->5->6->7)
            rc.cp[4] = Corner::DLF;
            rc.cp[5] = Corner::DBL;
            rc.cp[6] = Corner::DRB;
            rc.cp[7] = Corner::DFR;
            // D面回転では各ピースのD面ステッカーは下面のままなので向き変化は0
        }
        4 => {
            // L面: ULB(2), UFL(1), DLF(5), DBL(6) のサイクル (2->1->5->6)
            rc.cp[1] = Corner::ULB;
            rc.co[1] = 1;
            rc.cp[2] = Corner::DBL;
            rc.co[2] = 2;
            rc.cp[5] = Corner::UFL;
            rc.co[5] = 2;
            rc.cp[6] = Corner::DLF;
            rc.co[6] = 1;
        }
        5 => {
            // B面: UBR(3), ULB(2), DBL(6), DRB(7) のサイクル (3->2->6->7)
            rc.cp[2] = Corner::UBR;
            rc.co[2] = 1;
            rc.cp[3] = Corner::DRB;
            rc.co[3] = 2;
            rc.cp[6] = Corner::ULB;
            rc.co[6] = 2;
            rc.cp[7] = Corner::DBL;
            rc.co[7] = 1;
        }
        _ => unreachable!(),
    }
    rc
}

/// 18種類の回転操作 (0..17) に対応する RawCube
pub fn move_cube_18(mv: usize) -> &'static RawCube {
    static MOVES: OnceLock<[RawCube; 18]> = OnceLock::new();
    &MOVES.get_or_init(|| {
        let mut table = [RawCube::default(); 18];
        for face in 0..6 {
            let m1 = basic_move_cube(face);
            let m2 = m1.multiply(&m1);
            let m3 = m2.multiply(&m1);
            table[face * 3] = m1;
            table[face * 3 + 1] = m2;
            table[face * 3 + 2] = m3;
        }
        table
    })[mv]
}

// ----------------------------------------------------------------------------
// 4グループ統合 Pruning Table 用のインデックス計算
// コーナー8個を4つの重複グループ（各4コーナー）に分け、各グループの
// 配置（8P4 = 1680通り）と向き（3^4 = 81通り）を統合したインデックス (1680 * 81 = 136,080) を算出します。
// ----------------------------------------------------------------------------

#[inline]
fn get_group_idx(rc: &RawCube, corners: [Corner; 4]) -> usize {
    // 1. corners の各ピースが現在のどの位置にあるかを探す
    let mut pos = [0usize; 4];
    let mut ori = [0usize; 4];
    for (idx, &c) in corners.iter().enumerate() {
        let p = rc.cp.iter().position(|&x| x == c).unwrap();
        pos[idx] = p;
        ori[idx] = rc.co[p] as usize;
    }

    // 2. 8P4 の配置インデックス (0..1679)
    let mut perm_idx = 0;
    for i in 0..4 {
        let mut k = pos[i];
        for j in 0..i {
            if pos[j] < pos[i] {
                k -= 1;
            }
        }
        perm_idx = perm_idx * (8 - i) + k;
    }

    // 3. 3^4 の向きインデックス (0..80)
    let twist_idx = ori[0] * 27 + ori[1] * 9 + ori[2] * 3 + ori[3];

    perm_idx * 81 + twist_idx
}

/// Group A: UFR, UFL, ULB, UBR (上面4コーナー)
pub fn get_group_a_idx(rc: &RawCube) -> usize {
    get_group_idx(rc, [Corner::UFR, Corner::UFL, Corner::ULB, Corner::UBR])
}

/// Group B: DFR, DLF, DBL, DRB (下面4コーナー)
pub fn get_group_b_idx(rc: &RawCube) -> usize {
    get_group_idx(rc, [Corner::DFR, Corner::DLF, Corner::DBL, Corner::DRB])
}

/// Group C: UFL, DFR, DLF, UBR (斜向4コーナー)
pub fn get_group_c_idx(rc: &RawCube) -> usize {
    get_group_idx(rc, [Corner::UFL, Corner::DFR, Corner::DLF, Corner::UBR])
}

/// Group D: UFR, ULB, DBL, DRB (対角4コーナー)
pub fn get_group_d_idx(rc: &RawCube) -> usize {
    get_group_idx(rc, [Corner::UFR, Corner::ULB, Corner::DBL, Corner::DRB])
}

use super::coord::{
    move_cube_18, Corner, GROUP_A_CORNERS, GROUP_B_CORNERS, GROUP_C_CORNERS, GROUP_D_CORNERS,
};
use std::collections::VecDeque;
use std::sync::OnceLock;

/// 4グループ統合 Pruning Table
pub struct PruningTable {
    pub group_a: Box<[u8]>,
    pub group_b: &'static [u8],
    pub group_c: Box<[u8]>,
    pub group_d: Box<[u8]>,
}

impl PruningTable {
    /// Group B（下面4コーナー）のみを単体で遅延初期化して取得する（LBL 第1層用）
    pub fn group_b() -> &'static [u8] {
        static GROUP_B: OnceLock<Box<[u8]>> = OnceLock::new();
        GROUP_B
            .get_or_init(|| generate_group_pruning_table(GROUP_B_CORNERS))
            .as_ref()
    }

    pub fn get() -> &'static PruningTable {
        static TABLE: OnceLock<PruningTable> = OnceLock::new();
        TABLE.get_or_init(|| PruningTable {
            group_a: generate_group_pruning_table(GROUP_A_CORNERS),
            group_b: Self::group_b(),
            group_c: generate_group_pruning_table(GROUP_C_CORNERS),
            group_d: generate_group_pruning_table(GROUP_D_CORNERS),
        })
    }
}

#[inline(always)]
fn group_idx(pos: &[u8; 4], ori: &[u8; 4]) -> usize {
    let p0 = pos[0] as usize;
    let p1 = pos[1] as usize;
    let p2 = pos[2] as usize;
    let p3 = pos[3] as usize;

    let k0 = p0;
    let k1 = p1 - usize::from(p0 < p1);
    let k2 = p2 - usize::from(p0 < p2) - usize::from(p1 < p2);
    let k3 = p3 - usize::from(p0 < p3) - usize::from(p1 < p3) - usize::from(p2 < p3);
    let perm_idx = ((k0 * 7 + k1) * 6 + k2) * 5 + k3;

    let twist_idx =
        ori[0] as usize * 27 + ori[1] as usize * 9 + ori[2] as usize * 3 + ori[3] as usize;

    perm_idx * 81 + twist_idx
}

fn generate_group_pruning_table(corners: [Corner; 4]) -> Box<[u8]> {
    // 18 手それぞれについて、元の位置 src にいたコーナーが移動する先 dst と向き変化量 delta_co を事前計算する
    let mut next_pos = [[0u8; 8]; 18];
    let mut delta_co = [[0u8; 8]; 18];
    for m in 0..18 {
        let mv = move_cube_18(m);
        for (dst, &src_corner) in mv.cp.iter().enumerate() {
            let src = src_corner as usize;
            next_pos[m][src] = dst as u8;
            delta_co[m][src] = mv.co[dst];
        }
    }

    let start_pos = [
        corners[0] as u8,
        corners[1] as u8,
        corners[2] as u8,
        corners[3] as u8,
    ];
    let start_ori = [0u8; 4];

    let mut table = vec![255u8; 136080];
    table[group_idx(&start_pos, &start_ori)] = 0;

    let mut q = VecDeque::with_capacity(136080);
    q.push_back((start_pos, start_ori, 0u8));
    let mut visited_count = 1;

    while let Some((pos, ori, dist)) = q.pop_front() {
        let next_dist = dist + 1;
        for m in 0..18 {
            let np = [
                next_pos[m][pos[0] as usize],
                next_pos[m][pos[1] as usize],
                next_pos[m][pos[2] as usize],
                next_pos[m][pos[3] as usize],
            ];
            let no = std::array::from_fn(|k| {
                let mut o = ori[k] + delta_co[m][pos[k] as usize];
                if o >= 3 {
                    o -= 3;
                }
                o
            });
            let next_idx = group_idx(&np, &no);
            if table[next_idx] == 255 {
                table[next_idx] = next_dist;
                visited_count += 1;
                q.push_back((np, no, next_dist));
            }
        }
    }
    assert_eq!(
        visited_count, 136080,
        "2x2 pruning table generation incomplete"
    );
    table.into_boxed_slice()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::c2x2::coord::{corner_positions, group_idx_from_pos, RawCube};
    use crate::c2x2::cube::{apply, scramble};

    #[test]
    fn test_fast_4corner_bfs_matches_rawcube_group_indices() {
        let pt = PruningTable::get();
        let solved = RawCube::default();
        let pos_of = corner_positions(&solved);
        assert_eq!(
            pt.group_a[group_idx_from_pos(&solved, &pos_of, GROUP_A_CORNERS)],
            0
        );
        assert_eq!(
            pt.group_b[group_idx_from_pos(&solved, &pos_of, GROUP_B_CORNERS)],
            0
        );
        assert_eq!(
            pt.group_c[group_idx_from_pos(&solved, &pos_of, GROUP_C_CORNERS)],
            0
        );
        assert_eq!(
            pt.group_d[group_idx_from_pos(&solved, &pos_of, GROUP_D_CORNERS)],
            0
        );

        // 各手適用後も距離差は <= 1（一貫性 / アドミッシビリティ）
        for seed in 1..=10 {
            let rc = apply(&RawCube::default(), &scramble(seed));
            let pos = corner_positions(&rc);
            let da = pt.group_a[group_idx_from_pos(&rc, &pos, GROUP_A_CORNERS)];
            let db = pt.group_b[group_idx_from_pos(&rc, &pos, GROUP_B_CORNERS)];
            let dc = pt.group_c[group_idx_from_pos(&rc, &pos, GROUP_C_CORNERS)];
            let dd = pt.group_d[group_idx_from_pos(&rc, &pos, GROUP_D_CORNERS)];
            for m in 0..18 {
                let next = rc.multiply(move_cube_18(m));
                let npos = corner_positions(&next);
                let nda = pt.group_a[group_idx_from_pos(&next, &npos, GROUP_A_CORNERS)];
                let ndb = pt.group_b[group_idx_from_pos(&next, &npos, GROUP_B_CORNERS)];
                let ndc = pt.group_c[group_idx_from_pos(&next, &npos, GROUP_C_CORNERS)];
                let ndd = pt.group_d[group_idx_from_pos(&next, &npos, GROUP_D_CORNERS)];
                assert!(da.abs_diff(nda) <= 1);
                assert!(db.abs_diff(ndb) <= 1);
                assert!(dc.abs_diff(ndc) <= 1);
                assert!(dd.abs_diff(ndd) <= 1);
            }
        }
    }
}

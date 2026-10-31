//! 全コーナーをその場でひねった局面（3x3 のスーパーフリップのコーナー版）のテスト

use rubiks_cube_2x2::cube::validation::CORNER_STICKERS;
use rubiks_cube_2x2::cube::{Color, Cube, NUM_STICKERS};
use rubiks_cube_2x2::solver;

/// 2x2x2 の神の数（HTM）
const GODS_NUMBER_HTM: usize = 11;

/// 全コーナーひねり局面の最短手数（HTM）
const ALL_CORNERS_TWISTED_OPTIMAL: usize = 8;

/// 上段の 4 コーナーを時計回り、下段の 4 コーナーを反時計回りにひねった局面を作る
///
/// ひねりの合計は 4 × 1 + 4 × 2 = 12 ≡ 0 (mod 3) なので、実際に到達可能な局面になる。
/// 各コーナーの位置は動かさず、向きだけを変える。
fn all_corners_twisted() -> Cube {
    let solved = Cube::new();
    let mut colors: [Color; NUM_STICKERS] = std::array::from_fn(|i| solved.get_sticker(i).color);
    for (corner, stickers) in CORNER_STICKERS.iter().enumerate() {
        let [a, b, c] = stickers.map(|i| colors[i]);
        // 上段（0-3）は U/D 色を 1 つ先へ、下段（4-7）は 2 つ先へずらす
        let twisted = if corner < 4 { [c, a, b] } else { [b, c, a] };
        for (&index, color) in stickers.iter().zip(twisted) {
            colors[index] = color;
        }
    }
    // from_colors はステッカーの向き情報も物理的に正しく復元する
    Cube::from_colors(&colors).expect("到達可能な局面のはず")
}

#[test]
fn test_all_corners_twisted_is_valid_state() {
    let cube = all_corners_twisted();

    assert!(cube.is_valid_state().is_ok(), "到達可能な局面のはず");
    assert!(!cube.is_solved(), "完成状態ではないはず");
}

#[test]
fn test_all_corners_twisted_solved_within_gods_number() {
    let cube = all_corners_twisted();

    let solution = solver::solve(&cube, GODS_NUMBER_HTM, false);
    println!(
        "全コーナーひねり局面の解（{} 手）: {:?}",
        solution.moves.len(),
        solution.moves
    );

    assert!(
        solution.found,
        "神の数（{GODS_NUMBER_HTM} 手）以内で解けるはず"
    );
    assert_eq!(solution.moves.len(), ALL_CORNERS_TWISTED_OPTIMAL);

    let mut check = cube.clone();
    for &mv in &solution.moves {
        check.apply_move(mv);
    }
    assert!(
        check.is_solved_with_orientation(),
        "解を適用すると完成状態に戻るはず"
    );
}

#[test]
fn test_all_corners_twisted_not_solvable_in_fewer_moves() {
    let cube = all_corners_twisted();

    // 最短手数より 1 手少ない深さでは解が見つからない（8 手が最短であることの確認）
    let solution = solver::solve(&cube, ALL_CORNERS_TWISTED_OPTIMAL - 1, false);
    assert!(
        !solution.found,
        "{ALL_CORNERS_TWISTED_OPTIMAL} 手未満では解けないはず"
    );
}

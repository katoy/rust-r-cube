use cube_studio::cfop;
use cube_studio::coord::{move_cube_18, RawCube};
use cube_studio::cube::{apply, parse_moves};

fn main() {
    println!("=== Probe 1: CFOP First Layer Corner Search Analysis ===");

    // クロスが揃った状態から、D面回転（D: m=9）を行ったキューブを作成
    let solved = RawCube::default();
    let d_turn = move_cube_18(9); // D面 90度回転
    let d_cube = solved.multiply(d_turn);

    // クロスエッジの壊れ数をカウント
    let broken_cross = (4..8)
        .filter(|&i| d_cube.ep[i] as usize != i || d_cube.eo[i] != 0)
        .count();
    println!("D面回転直後のクロス破壊エッジ数: {}", broken_cross);

    // D' (m=11) で戻せば 1 手で 4 個のクロスエッジが修復される
    let d_prime = move_cube_18(11);
    let restored = d_cube.multiply(d_prime);
    let restored_broken = (4..8)
        .filter(|&i| restored.ep[i] as usize != i || restored.eo[i] != 0)
        .count();
    println!("D' を 1 手適用した後のクロス破壊エッジ数: {}", restored_broken);

    println!("\n【数理的帰結】");
    println!("D面回転を許容する場合、1手で修復できるクロスエッジ数は最大4個。");
    println!("したがって、アドミッシブル（決して真の距離を過大評価しない）下界は broken_cross.div_ceil(4) であるべき。");
    println!("現在の実装 `broken_cross > depth` は、D面回転を含む手順においてアドミッシビリティが破綻している。");
    println!("また、第1層コーナー解法においてD面回転は不要であるため、最初から face == 3 を除外すべきである。");
}

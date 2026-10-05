use cube_studio::coord::RawCube;
use cube_studio::cube::{apply, parse_moves};

fn main() {
    let alg = "U R L U2 R' L' U R L U2 R' L'";
    let moves = parse_moves(alg).unwrap();
    let solved = RawCube::default();
    let res = apply(&solved, &moves);
    println!("キューブは完成状態のままか？: {}", res == solved);

    // センター回転の計算
    let mut centers = [0i32; 6];
    for &m in &moves {
        let f = m / 3;
        let t = match m % 3 {
            0 => 1,
            1 => 2,
            2 => -1,
            _ => 0,
        };
        centers[f] = (centers[f] + t).rem_euclid(4);
    }
    println!("各面のセンター正味回転: {:?}", centers);
}

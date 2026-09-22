use cube_studio::supercube::solve_center_orientations;

fn main() {
    println!("=== Probe 2: Supercube Center Orientations Parity Analysis ===");

    // 1つのセンターだけが90度回転（奇数回転面が1個）の入力（物理的に不可能・パリティ矛盾）
    let invalid_centers = [1, 0, 0, 0, 0, 0];
    let solution = solve_center_orientations(invalid_centers);
    println!("不正な入力（1つのセンターのみ90度回転）: {:?}", invalid_centers);
    println!("返された手順の手数: {}", solution.len());
    println!("返された手順: {:?}", solution);

    // 手順を適用した後のセンター回転を追跡
    let mut centers = invalid_centers;
    for &m in &solution {
        let f = m / 3;
        let t = match m % 3 {
            0 => 1,
            1 => 2,
            2 => -1,
            _ => unreachable!(),
        };
        centers[f] = (centers[f] + t).rem_euclid(4);
    }
    println!("手順適用後のセンター回転: {:?}", centers);
    let is_solved = centers.iter().all(|&c| c == 0);
    println!("センターは揃ったか？: {}", is_solved);

    // 3つのセンターが90度回転（奇数回転面が3個）の入力
    let three_centers = [1, 1, 1, 0, 0, 0];
    let sol3 = solve_center_orientations(three_centers);
    println!("\n入力（3つのセンターが90度回転）: {:?}", three_centers);
    println!("返された手順の手数: {}", sol3.len());
    let mut centers3 = three_centers;
    for &m in &sol3 {
        let f = m / 3;
        let t = match m % 3 {
            0 => 1,
            1 => 2,
            2 => -1,
            _ => unreachable!(),
        };
        centers3[f] = (centers3[f] + t).rem_euclid(4);
    }
    println!("手順適用後のセンター回転: {:?}", centers3);
    println!("センターは揃ったか？: {}", centers3.iter().all(|&c| c == 0));

    println!("\n=> 【実証成功】奇数個の奇数回転が存在する場合、最後のセンターが未解決のまま無視され、サイレントに不完全な手順が返される！");
}

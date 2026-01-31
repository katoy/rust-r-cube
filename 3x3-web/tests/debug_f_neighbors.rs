use rubiks_cube_3x3::cube::{Cube, Move, NUM_STICKERS};
use rubiks_cube_3x3::gui::mapping::{get_oris_delta, get_source_index};

#[test]
fn debug_f_neighbors() {
    let mv = Move::F;
    let initial_cube = Cube::new();
    let mut after_cube = initial_cube.clone();
    after_cube.apply_move(mv);

    // F 面の隣接ステッカーおよび F 面自体のステッカー (36-44)
    let targets = [
        6, 7, 8, 27, 30, 33, 9, 10, 11, 20, 23, 26, // Neighbors
        36, 37, 38, 39, 40, 41, 42, 43, 44, // Front face itself
    ];

    for &src_idx in &targets {
        // dst_idx を逆引き
        let dst_idx = (0..NUM_STICKERS)
            .find(|&d| get_source_index(mv, d) == src_idx)
            .unwrap();

        let initial_ori = initial_cube.get_sticker(src_idx).orientation;
        let after_ori = after_cube.get_sticker(dst_idx).orientation;
        let delta = get_oris_delta(mv, src_idx);

        println!(
            "Sticker {}: from_face={}, initial_ori={}, after_ori (dst={})={}, delta={}",
            src_idx,
            src_idx / 9,
            initial_ori,
            dst_idx,
            after_ori,
            delta
        );

        let expected_after = (initial_ori + delta) % 4;
        if after_ori != expected_after {
            println!(
                "  !!! MISMATCH: after_ori {} != expected {}",
                after_ori, expected_after
            );
        }
    }
}

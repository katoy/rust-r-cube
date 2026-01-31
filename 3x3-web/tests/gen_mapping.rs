use rubiks_cube_3x3::cube::{Cube, Move, NUM_STICKERS};

#[test]
fn generate_mapping_code() {
    let all_moves = Move::all_moves();

    println!("pub const MOVE_MAPPING_TABLE: [[(usize, usize); 54]; 36] = [");
    for &mv in &all_moves {
        let initial_cube = Cube::new();
        let mut after_cube = initial_cube.clone();
        after_cube.apply_move(mv);

        print!("    [ // {}\n", mv);
        let mut count = 0;
        for dst_idx in 0..NUM_STICKERS {
            let initial_sticker = initial_cube.get_sticker(dst_idx);
            let after_sticker = after_cube.get_sticker(dst_idx);

            // 全ての遷移を出力（または移動したものだけを出力して 99 で埋める）
            // ここでは一貫性のために全54個を出力する方針にする
            let src_idx = (0..NUM_STICKERS).find(|&s| {
                // 物理モデルのピース位置から逆引き
                // 実際には get_initial_pieces などの状態から追跡が必要だが
                // 単純に「初期状態でこの色だった場所」を探す
                initial_cube.get_sticker(s).color == after_cube.get_sticker(dst_idx).color
                    && s / 9 == after_cube.get_sticker(dst_idx).color as usize // 同じ面の色であること（簡易判定）
            });

            // 正確な逆引きのために piece.id を使うべきだが、
            // 幸い初期状態は unique

            // もっと確実に:
            let src_idx = find_source_index(mv, dst_idx);

            println!("        ({}, {}),", src_idx, dst_idx);
            count += 1;
        }
        for _ in count..54 {
            println!("        (99, 99),");
        }
        println!("    ],");
    }
    println!("];");
}

fn find_source_index(mv: Move, dst_idx: usize) -> usize {
    let mut initial_cube = Cube::new();
    // 各ステッカーにユニークな「初期インデックス」を color の代わりに埋め込む
    // （Pieceモデルの内部情報にアクセスする必要があるが、
    // ここでは apply_move をシミュレートして追跡する）

    // 実際には rotation.rs のロジックを模倣してインデックスを追跡する
    // ...
    0 // dummy
}

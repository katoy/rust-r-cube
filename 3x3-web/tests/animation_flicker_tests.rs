use rubiks_cube_3x3::cube::{Color, Cube, Move};
use rubiks_cube_3x3::gui::mapping::{get_source_index, get_oris_delta};

/// アニメーション開始時の「瞬き（書き換え）」バグを検出するテスト
/// 
/// 理論:
/// アニメーション開始時(t=0)において、ステッカーが「移動元(Source)の向き」と
/// 一致していない場合、瞬間的に矢印が跳ねる「瞬き」が発生する。
/// 
/// レンダラー内での開始向き計算: (Current_Orientation - oris_delta) % 4
/// これが Source_Orientation と一致しなければならない。

#[test]
fn test_m_move_animation_continuity() {
    let mut cube = Cube::new();
    
    // M 操作前の状態 (Source)
    // センター(4)と、中列(1, 4, 7 ...) はすべて orientation = 0
    let source_ori_u1 = cube.get_sticker(1).orientation; // 0
    
    // M 操作適用
    cube.apply_move(Move::M);
    
    // 操作後の U1 (Target)
    let target_ori_u1 = cube.get_sticker(1).orientation;
    
    // U1 のソースインデックスを取得 (M 操作で 1 に来たのはどこか？)
    let src_idx = get_source_index(Move::M, 1);
    
    // oris_delta を取得
    let delta = get_oris_delta(Move::M, src_idx);
    
    // レンダラーが計算する開始時の向き
    let start_ori = (target_ori_u1 as i16 - delta as i16).rem_euclid(4) as u8;
    
    println!("M move continuity check for U1:");
    println!("  Source Ori: {}", source_ori_u1);
    println!("  Target Ori: {}", target_ori_u1);
    println!("  oris_delta: {}", delta);
    println!("  Animation Start Ori: {}", start_ori);
    
    assert_eq!(
        start_ori, source_ori_u1,
        "Flicker detected! Animation starts at orientation {} but source was {}. (Move: M, Target Index: 1)",
        start_ori, source_ori_u1
    );
}

#[test]
fn test_u_move_animation_continuity() {
    let mut cube = Cube::new();
    let source_ori_u4 = cube.get_sticker(4).orientation; // 0
    
    cube.apply_move(Move::U);
    
    let target_ori_u4 = cube.get_sticker(4).orientation;
    let src_idx = get_source_index(Move::U, 4); // 4
    let delta = get_oris_delta(Move::U, src_idx);
    
    let start_ori = (target_ori_u4 as i16 - delta as i16).rem_euclid(4) as u8;
    
    assert_eq!(
        start_ori, source_ori_u4,
        "Flicker detected in U move! Start: {}, Source: {}", 
        start_ori, source_ori_u4
    );
}

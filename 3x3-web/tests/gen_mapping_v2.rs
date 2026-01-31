use glam::Vec3;
use rubiks_cube_3x3::cube::{Cube, Face, Move, NUM_STICKERS};

#[test]
fn dump_mapping_v2() {
    let all_moves = Move::all_moves();
    let initial_cube = Cube::new();

    // (dst_idx) -> (piece_initial_pos, sticker_initial_normal) の対応を初期状態で作成
    let mut sticker_to_piece = Vec::new();
    for dst_idx in 0..NUM_STICKERS {
        let mut found = false;
        for piece in &initial_cube.pieces {
            for target in &piece.stickers {
                let normal = piece.current_rot.transform_vector3(target.initial_normal);
                let pos = piece.current_pos;

                if let Some(face) = get_face_of_normal(normal) {
                    let local_idx = rubiks_cube_3x3::cube::piece::face_to_local_index(face, pos);
                    if (face as usize) * 9 + local_idx == dst_idx {
                        sticker_to_piece.push((piece.initial_pos, target.initial_normal));
                        found = true;
                        break;
                    }
                }
            }
            if found {
                break;
            }
        }
    }

    println!("pub const MOVE_MAPPING_TABLE: [[(usize, usize); 54]; 36] = [");
    for &mv in &all_moves {
        let mut cube = initial_cube.clone();
        cube.apply_move(mv);

        println!("    [ // {}", mv);
        for (src_idx, &(target_initial_pos, target_initial_normal)) in
            sticker_to_piece.iter().enumerate().take(NUM_STICKERS)
        {
            let mut found_dst = 99;
            for piece in &cube.pieces {
                if piece.initial_pos == target_initial_pos {
                    for target in &piece.stickers {
                        if target.initial_normal == target_initial_normal {
                            let normal = piece.current_rot.transform_vector3(target.initial_normal);
                            if let Some(face) = get_face_of_normal(normal) {
                                let local_idx = rubiks_cube_3x3::cube::piece::face_to_local_index(
                                    face,
                                    piece.current_pos,
                                );
                                found_dst = (face as usize) * 9 + local_idx;
                            }
                            break;
                        }
                    }
                    break;
                }
            }
            println!("        ({}, {}),", src_idx, found_dst);
        }
        println!("    ],");
    }
    println!("];");
}

fn get_face_of_normal(n: Vec3) -> Option<Face> {
    let nx = n.x.round();
    let ny = n.y.round();
    let nz = n.z.round();
    if ny > 0.9 {
        Some(Face::Up)
    } else if ny < -0.9 {
        Some(Face::Down)
    } else if nx < -0.9 {
        Some(Face::Left)
    } else if nx > 0.9 {
        Some(Face::Right)
    } else if nz > 0.9 {
        Some(Face::Front)
    } else if nz < -0.9 {
        Some(Face::Back)
    } else {
        None
    }
}

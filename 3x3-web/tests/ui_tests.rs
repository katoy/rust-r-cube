use rubiks_cube_3x3::cube::{Cube, Move};
use rubiks_cube_3x3::gui::mapping::{get_oris_delta, get_source_index};

// ==================== Animation Continuity Tests ====================

#[test]
fn test_m_operations_oris_delta() {
    assert_eq!(
        get_oris_delta(Move::M, 46),
        2,
        "B->U should have oris_delta=2"
    );
    assert_eq!(
        get_oris_delta(Move::M, 10),
        2,
        "D->B should have oris_delta=2"
    );
    assert_eq!(
        get_oris_delta(Move::M, 37),
        0,
        "U->F should have oris_delta=0"
    );
}

#[test]
fn test_l_operations_oris_delta() {
    assert_eq!(
        get_oris_delta(Move::L, 53),
        2,
        "B->U should have oris_delta=2"
    );
    assert_eq!(
        get_oris_delta(Move::L, 9),
        2,
        "D->B should have oris_delta=2"
    );
}

#[test]
fn test_face_rotation_oris_delta() {
    assert_eq!(
        get_oris_delta(Move::U, 4),
        1,
        "U-center should have oris_delta=1"
    );
    assert_eq!(
        get_oris_delta(Move::F, 40),
        1,
        "F-center should have oris_delta=1"
    );
    assert_eq!(
        get_oris_delta(Move::B, 49),
        1,
        "B-center should have oris_delta=1"
    );
}

#[test]
fn test_frontal_adjacent_oris_delta() {
    assert_eq!(
        get_oris_delta(Move::F, 7),
        1,
        "U-Front-Mid should have oris_delta=1"
    );
    assert_eq!(
        get_oris_delta(Move::B, 1),
        3,
        "U-Back-Mid should have oris_delta=3"
    );
}

// ==================== Animation Flicker Tests ====================

#[test]
fn test_m_move_animation_continuity() {
    let mut cube = Cube::new();
    let source_ori_u1 = cube.get_sticker(1).orientation;
    cube.apply_move(Move::M);
    let target_ori_u1 = cube.get_sticker(1).orientation;
    let src_idx = get_source_index(Move::M, 1);
    let delta = get_oris_delta(Move::M, src_idx);
    let start_ori = (target_ori_u1 as i16 - delta as i16).rem_euclid(4) as u8;

    assert_eq!(start_ori, source_ori_u1);
}

#[test]
fn test_u_move_animation_continuity() {
    let mut cube = Cube::new();
    let source_ori_u4 = cube.get_sticker(4).orientation;
    cube.apply_move(Move::U);
    let target_ori_u4 = cube.get_sticker(4).orientation;
    let src_idx = get_source_index(Move::U, 4);
    let delta = get_oris_delta(Move::U, src_idx);
    let start_ori = (target_ori_u4 as i16 - delta as i16).rem_euclid(4) as u8;

    assert_eq!(start_ori, source_ori_u4);
}

// ==================== Web UI Tests (WASM specific) ====================

#[cfg(target_arch = "wasm32")]
mod wasm_ui_tests {
    use super::*;
    use rubiks_cube_3x3::gui::app::CubeApp;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_wasm_solver_flow() {
        let mut app = CubeApp::default();
        assert!(!app.solving);
        app.solve_without_confirm();
        assert!(app.solving);
        assert_eq!(app.solution_text, "探索中...");

        let ctx = egui::Context::default();
        app.update_logic(&ctx);
        assert!(app.solving);
        assert!(app.solver_progress < 0.01);

        app.update_logic(&ctx);
        app.update_logic(&ctx);
        assert!(app.solving);
    }

    #[wasm_bindgen_test]
    fn test_wasm_solver_cancel() {
        let mut app = CubeApp::default();
        app.solve_without_confirm();
        assert!(app.solving);
        app.cancel_solve();
        assert!(!app.solving);
        assert!(app.solution.is_none());
    }

    #[wasm_bindgen_test]
    fn test_wasm_solver_incremental_progress() {
        let mut app = CubeApp::default();
        app.solve_without_confirm();
        let ctx = egui::Context::default();
        for _ in 0..5 {
            app.update_logic(&ctx);
        }
        for _ in 0..10 {
            app.update_logic(&ctx);
        }
    }
}

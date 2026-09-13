#[allow(clippy::upper_case_acronyms)]
pub mod coord;
pub mod cube;
pub mod search;
pub mod supercube;
mod tables;

const TABLE_BYTES: Option<&[u8]> = Some(include_bytes!(concat!(env!("OUT_DIR"), "/tables.bin")));

use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Debug, Serialize, serde::Deserialize)]
pub struct ResultData {
    pub state: String,
    pub moves: Vec<String>,
    pub states: Vec<String>,
    pub elapsed_ms: f64,
    pub nodes: u64,
}
fn result(state: &str, moves: &[usize], elapsed_ms: f64, nodes: u64) -> Result<ResultData, String> {
    let mut cube = cube::parse_state(state)?;
    let mut states = vec![state.to_owned()];
    for m in moves {
        cube = cube::apply(&cube, &[*m]);
        states.push(cube::facelets(&cube));
    }
    Ok(ResultData {
        state: cube::facelets(&cube),
        moves: moves.iter().map(|m| cube::notation(*m)).collect(),
        states,
        elapsed_ms,
        nodes,
    })
}
pub fn solve_state(
    state: &str,
    budget_ms: u32,
    include_orientation: bool,
) -> Result<ResultData, String> {
    solve_state_with_centers(state, budget_ms, include_orientation, None)
}

pub fn solve_state_with_centers(
    state: &str,
    budget_ms: u32,
    include_orientation: bool,
    initial_centers: Option<[i32; 6]>,
) -> Result<ResultData, String> {
    let cube = cube::parse_state(state)?;
    let start = web_time::Instant::now();
    let mut total_nodes = 0u64;
    let moves_opt = if let (true, Some(centers)) = (include_orientation, initial_centers) {
        let mut search_oriented =
            search::Search::new((budget_ms / 2).min(15000)).with_target_centers(centers);
        let res = search_oriented.solve(&cube);
        total_nodes += search_oriented.nodes;
        res
    } else {
        None
    };

    let mut moves = if let Some(m) = moves_opt {
        m
    } else {
        let elapsed_ms = start.elapsed().as_millis() as u32;
        let remaining_budget = budget_ms.saturating_sub(elapsed_ms).min(30000);
        let mut search = search::Search::new(remaining_budget);
        let m = search.solve(&cube).ok_or_else(|| {
            "探索時間の上限に達しました。30秒の延長探索を試してください。".to_owned()
        })?;
        total_nodes += search.nodes;
        m
    };

    if include_orientation {
        if let Some(initial) = initial_centers {
            // センターの向き（Supercube仕様）を解く
            let mut centers = initial;
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
            let center_fixes = supercube::solve_center_orientations(centers);
            moves.extend(center_fixes);
        }
    }

    // 完成状態を確認
    let result_cube = cube::apply(&cube, &moves);
    let is_solved = if include_orientation {
        // 向き情報を含める: 完全に解けているか確認
        result_cube == coord::RawCube::default()
    } else {
        // 色だけを確認: 向き情報を無視して色だけが揃っているか確認
        cube::facelets(&result_cube) == cube::SOLVED
    };

    if !is_solved {
        return Err("解法の検証に失敗しました。".into());
    }

    result(
        state,
        &moves,
        start.elapsed().as_secs_f64() * 1000.0,
        total_nodes,
    )
}
fn json(value: Result<ResultData, String>) -> Result<String, JsValue> {
    value
        .and_then(|v| serde_json::to_string(&v).map_err(|e| e.to_string()))
        .map_err(|e| JsValue::from_str(&e))
}
#[wasm_bindgen]
pub fn initialize() {
    let _ = tables::MoveTable::get();
    let _ = tables::PruningTable::get();
}
#[wasm_bindgen]
pub fn validate(state: &str) -> Result<bool, JsValue> {
    cube::parse_state(state)
        .map(|c| c == coord::RawCube::default())
        .map_err(|e| JsValue::from_str(&e))
}
#[wasm_bindgen]
pub fn apply_moves(state: &str, moves: &str) -> Result<String, JsValue> {
    json(cube::parse_moves(moves).and_then(|m| result(state, &m, 0.0, 0)))
}
#[wasm_bindgen]
pub fn scramble(seed: u32) -> String {
    cube::scramble(seed)
        .iter()
        .map(|m| cube::notation(*m))
        .collect::<Vec<_>>()
        .join(" ")
}
#[wasm_bindgen]
pub fn solve(state: &str, budget_ms: u32) -> Result<String, JsValue> {
    json(solve_state(state, budget_ms, true))
}

#[wasm_bindgen]
pub fn solve_with_orientation(
    state: &str,
    budget_ms: u32,
    include_orientation: bool,
    centers_str: Option<String>,
) -> Result<String, JsValue> {
    let initial_centers = centers_str.and_then(|s| {
        let nums: Vec<i32> = s.split(',').filter_map(|p| p.trim().parse().ok()).collect();
        if nums.len() == 6 {
            Some([nums[0], nums[1], nums[2], nums[3], nums[4], nums[5]])
        } else {
            None
        }
    });
    json(solve_state_with_centers(
        state,
        budget_ms,
        include_orientation,
        initial_centers,
    ))
}

/// キューブのピース向き情報を JSON で返す
/// コーナーの向き: 0=正常, 1=時計回り90°, 2=反時計回り90°
/// エッジの向き: 0=正常, 1=反転
#[wasm_bindgen]
pub fn get_orientations(state: &str) -> Result<String, JsValue> {
    let raw_cube = cube::parse_state(state).map_err(|e| JsValue::from_str(&e))?;

    let corner_orientations: Vec<usize> = raw_cube.co.iter().map(|&o| o as usize).collect();
    let edge_orientations: Vec<usize> = raw_cube.eo.iter().map(|&o| o as usize).collect();

    let result = serde_json::json!({
        "corners": corner_orientations,
        "edges": edge_orientations,
    });

    serde_json::to_string(&result)
        .map_err(|e| JsValue::from_str(&format!("JSON serialization error: {}", e)))
}

#[cfg(test)]
mod tests;

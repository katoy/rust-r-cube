#[allow(clippy::upper_case_acronyms)]
pub mod coord;
pub mod cube;
pub mod search;
mod tables;

const TABLE_BYTES: Option<&[u8]> = Some(include_bytes!(concat!(env!("OUT_DIR"), "/tables.bin")));

use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
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
pub fn solve_state(state: &str, budget_ms: u32) -> Result<ResultData, String> {
    let cube = cube::parse_state(state)?;
    let start = web_time::Instant::now();
    let mut search = search::Search::new(budget_ms.min(30000));
    let moves = search
        .solve(&cube)
        .ok_or_else(|| "探索時間の上限に達しました。30秒の延長探索を試してください。".to_owned())?;
    if cube::apply(&cube, &moves) != coord::RawCube::default() {
        return Err("解法の検証に失敗しました。".into());
    }
    result(
        state,
        &moves,
        start.elapsed().as_secs_f64() * 1000.0,
        search.nodes,
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
    json(solve_state(state, budget_ms))
}

#[cfg(test)]
mod tests;

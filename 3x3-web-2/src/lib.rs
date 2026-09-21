pub mod cfop;
#[allow(clippy::upper_case_acronyms)]
pub mod coord;
pub mod cube;
pub mod korf;
pub mod search;
pub mod supercube;
mod tables;
pub mod thistlethwaite;

const TABLE_BYTES: Option<&[u8]> = Some(include_bytes!(concat!(env!("OUT_DIR"), "/tables.bin")));

use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct PhaseInfo {
    pub name: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Serialize, serde::Deserialize)]
pub struct ResultData {
    pub state: String,
    pub moves: Vec<String>,
    pub states: Vec<String>,
    pub elapsed_ms: f64,
    pub nodes: u64,
    #[serde(default)]
    pub algorithm: String,
    #[serde(default)]
    pub phases: Vec<PhaseInfo>,
}
fn result(
    state: &str,
    moves: &[usize],
    elapsed_ms: f64,
    nodes: u64,
    algorithm: &str,
    phases: Vec<PhaseInfo>,
) -> Result<ResultData, String> {
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
        algorithm: algorithm.to_string(),
        phases,
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
    solve_state_with_algorithm(
        state,
        budget_ms,
        include_orientation,
        initial_centers,
        "kociemba",
    )
}

pub fn solve_state_with_algorithm(
    state: &str,
    budget_ms: u32,
    include_orientation: bool,
    initial_centers: Option<[i32; 6]>,
    algorithm: &str,
) -> Result<ResultData, String> {
    let cube = cube::parse_state(state)?;
    if let (true, Some(centers)) = (include_orientation, initial_centers) {
        let center_parity = centers.iter().map(|c| c.rem_euclid(4)).sum::<i32>() % 2;
        if center_parity as usize != cube::parity(&cube.cp.map(|c| c as u8)) {
            return Err(
                "センターの向きと配色が整合しません。向きを確認するか、自動設定してください。"
                    .into(),
            );
        }
    }
    let start = web_time::Instant::now();
    let mut total_nodes = 0u64;
    let mut phase_infos = Vec::new();

    let mut moves = match algorithm {
        "cfop" => {
            let res = cfop::solve(&cube, budget_ms)?;
            let mut offset = 0;
            for p in res.phases {
                let len = p.moves.len();
                phase_infos.push(PhaseInfo {
                    name: p.name.to_string(),
                    start: offset,
                    end: offset + len,
                });
                offset += len;
            }
            res.moves
        }
        "thistlethwaite" => {
            let mut th = thistlethwaite::ThistlethwaiteSearch::new(budget_ms);
            let res = th.solve(&cube)?;
            total_nodes += th.nodes;
            let mut offset = 0;
            for p in res.phases {
                let len = p.moves.len();
                phase_infos.push(PhaseInfo {
                    name: p.name.to_string(),
                    start: offset,
                    end: offset + len,
                });
                offset += len;
            }
            res.moves
        }
        "korf" => {
            let mut korf = korf::KorfSearch::new(budget_ms);
            let sol = korf
                .solve(&cube)
                .ok_or_else(|| "探索時間の上限に達しました。".to_owned())?;
            total_nodes += korf.nodes;
            let phase_name = if sol.is_optimal {
                "Korf 最短探索 (IDA*)"
            } else {
                "Kociemba フォールバック"
            };
            phase_infos.push(PhaseInfo {
                name: phase_name.to_string(),
                start: 0,
                end: sol.moves.len(),
            });
            sol.moves
        }
        _ => {
            // Kociemba (Two-Phase)
            let moves_opt = if let (true, Some(centers)) = (include_orientation, initial_centers) {
                let oriented_budget = budget_ms
                    .saturating_sub(1000)
                    .max(budget_ms * 4 / 5)
                    .min(25000);
                let mut search_oriented =
                    search::Search::new(oriented_budget).with_target_centers(centers);
                let res = search_oriented.solve(&cube);
                total_nodes += search_oriented.nodes;
                res
            } else {
                None
            };

            if let Some(m) = moves_opt {
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
            }
        }
    };

    if include_orientation {
        if let Some(initial) = initial_centers {
            let mut centers = initial;
            for &m in &moves {
                let f = m / 3;
                let t = match m % 3 {
                    0 => 1,
                    1 => 2,
                    2 => -1,
                    _ => unreachable!("m % 3 は 0, 1, 2 のみ"),
                };
                centers[f] = (centers[f] + t).rem_euclid(4);
            }
            let center_fixes = supercube::solve_center_orientations(centers);
            if !center_fixes.is_empty() {
                let start_idx = moves.len();
                if phase_infos.is_empty() && start_idx > 0 {
                    phase_infos.push(PhaseInfo {
                        name: "色解法 (Kociemba)".to_string(),
                        start: 0,
                        end: start_idx,
                    });
                }
                moves.extend(center_fixes);
                phase_infos.push(PhaseInfo {
                    name: "センター向き解決".to_string(),
                    start: start_idx,
                    end: moves.len(),
                });
            }
        }
    }

    // 完成状態を確認
    let result_cube = cube::apply(&cube, &moves);
    let is_pieces_solved = if include_orientation {
        result_cube == coord::RawCube::default()
    } else {
        cube::facelets(&result_cube) == cube::SOLVED
    };

    if !is_pieces_solved {
        return Err("解法の検証に失敗しました。".into());
    }

    if include_orientation {
        if let Some(initial) = initial_centers {
            let mut final_centers = initial;
            for &m in &moves {
                let f = m / 3;
                let t = match m % 3 {
                    0 => 1,
                    1 => 2,
                    2 => -1,
                    _ => unreachable!("m % 3 は 0, 1, 2 のみ"),
                };
                final_centers[f] = (final_centers[f] + t).rem_euclid(4);
            }
            if final_centers.iter().any(|&c| c != 0) {
                return Err("センター向きの検証に失敗しました。".into());
            }
        }
    }

    result(
        state,
        &moves,
        start.elapsed().as_secs_f64() * 1000.0,
        total_nodes,
        algorithm,
        phase_infos,
    )
}

#[inline]
fn to_js_error(err: impl std::fmt::Display) -> JsValue {
    #[cfg(target_arch = "wasm32")]
    {
        JsValue::from_str(&err.to_string())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = err;
        JsValue::UNDEFINED
    }
}

fn json(value: Result<ResultData, String>) -> Result<String, JsValue> {
    match value {
        Ok(v) => serde_json::to_string(&v).map_err(to_js_error),
        Err(e) => Err(to_js_error(e)),
    }
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
        .map_err(to_js_error)
}
/// Required parity of the sum of center quarter turns. Each face quarter turn
/// changes both this sum's parity and the corner permutation's parity.
#[wasm_bindgen]
pub fn center_parity(state: &str) -> Result<u8, JsValue> {
    cube::parse_state(state)
        .map(|c| cube::parity(&c.cp.map(|p| p as u8)) as u8)
        .map_err(to_js_error)
}
#[wasm_bindgen]
pub fn apply_moves(state: &str, moves: &str) -> Result<String, JsValue> {
    json(cube::parse_moves(moves).and_then(|m| result(state, &m, 0.0, 0, "apply", Vec::new())))
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
    solve_with_algorithm(state, budget_ms, include_orientation, centers_str, None)
}

pub(crate) fn parse_initial_centers(centers_str: Option<&str>) -> Result<Option<[i32; 6]>, String> {
    match centers_str {
        Some(s) if !s.trim().is_empty() => {
            let tokens: Vec<&str> = s.split(',').map(|p| p.trim()).collect();
            if tokens.len() != 6 {
                return Err("センター入力は6個のカンマ区切り数値である必要があります。".to_owned());
            }
            let mut nums = [0i32; 6];
            for (i, tok) in tokens.iter().enumerate() {
                let n: i32 = tok
                    .parse()
                    .map_err(|_| format!("不正なセンタートークンです: '{}'", tok))?;
                if !(0..=3).contains(&n) {
                    return Err(format!(
                        "センター回転は0から3の範囲である必要があります: {}",
                        n
                    ));
                }
                nums[i] = n;
            }
            Ok(Some(nums))
        }
        _ => Ok(None),
    }
}

#[wasm_bindgen]
pub fn solve_with_algorithm(
    state: &str,
    budget_ms: u32,
    include_orientation: bool,
    centers_str: Option<String>,
    algorithm: Option<String>,
) -> Result<String, JsValue> {
    let initial_centers = match parse_initial_centers(centers_str.as_deref()) {
        Ok(c) => c,
        Err(e) => return json(Err(e)),
    };
    let alg = algorithm.as_deref().unwrap_or("kociemba");
    json(solve_state_with_algorithm(
        state,
        budget_ms,
        include_orientation,
        initial_centers,
        alg,
    ))
}

/// キューブのピース向き情報を JSON で返す
/// コーナーの向き: 0=正常, 1=時計回り90°, 2=反時計回り90°
/// エッジの向き: 0=正常, 1=反転
#[wasm_bindgen]
pub fn get_orientations(state: &str) -> Result<String, JsValue> {
    let raw_cube = cube::parse_state(state).map_err(to_js_error)?;

    let corner_orientations: Vec<usize> = raw_cube.co.iter().map(|&o| o as usize).collect();
    let edge_orientations: Vec<usize> = raw_cube.eo.iter().map(|&o| o as usize).collect();

    let result = serde_json::json!({
        "corners": corner_orientations,
        "edges": edge_orientations,
    });

    serde_json::to_string(&result)
        .map_err(|e| to_js_error(format!("JSON serialization error: {}", e)))
}

#[cfg(test)]
mod tests;

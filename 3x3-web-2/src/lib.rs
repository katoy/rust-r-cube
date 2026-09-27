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
    cube: &coord::RawCube,
    state: &str,
    moves: &[usize],
    elapsed_ms: f64,
    nodes: u64,
    algorithm: &str,
    phases: Vec<PhaseInfo>,
) -> ResultData {
    let mut current_cube = *cube;
    let mut states = Vec::with_capacity(moves.len() + 1);
    states.push(state.to_owned());
    for m in moves {
        current_cube = cube::apply(&current_cube, &[*m]);
        states.push(cube::facelets(&current_cube));
    }
    let final_state = states.last().cloned().unwrap_or_else(|| state.to_owned());
    ResultData {
        state: final_state,
        moves: moves.iter().map(|m| cube::notation(*m)).collect(),
        states,
        elapsed_ms,
        nodes,
        algorithm: algorithm.to_string(),
        phases,
    }
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
            total_nodes += res.nodes;
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
                let oriented_budget = if budget_ms <= 1000 {
                    budget_ms / 2
                } else {
                    budget_ms.saturating_sub(1000).max(budget_ms * 4 / 5)
                };
                let mut search_oriented =
                    search::Search::new(oriented_budget).with_target_centers(centers);
                let res = search_oriented.solve(&cube);
                total_nodes += search_oriented.nodes;
                res
            } else {
                None
            };

            if let Some(m) = moves_opt {
                phase_infos.push(PhaseInfo {
                    name: "同時最適化 (色＆センター)".to_string(),
                    start: 0,
                    end: m.len(),
                });
                m
            } else {
                let elapsed_ms = start.elapsed().as_millis() as u32;
                let remaining_budget = budget_ms
                    .saturating_sub(elapsed_ms)
                    .max(if budget_ms > 5000 { 3000 } else { 0 });
                let mut search = search::Search::new(remaining_budget);
                let m = search.solve(&cube).ok_or_else(|| {
                    "探索時間の上限に達しました。30秒の延長探索を試してください。".to_owned()
                })?;
                let p1 = search.best_phase1_len;
                total_nodes += search.nodes;
                let total_len = m.len();
                if p1 > 0 && p1 < total_len {
                    phase_infos.push(PhaseInfo {
                        name: "Kociemba Phase 1 (G1縮約)".to_string(),
                        start: 0,
                        end: p1,
                    });
                    phase_infos.push(PhaseInfo {
                        name: "Kociemba Phase 2 (群解決)".to_string(),
                        start: p1,
                        end: total_len,
                    });
                } else if total_len > 0 {
                    phase_infos.push(PhaseInfo {
                        name: "Kociemba 直接解決".to_string(),
                        start: 0,
                        end: total_len,
                    });
                }
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
                    _ => -1,
                };
                centers[f] = (centers[f] + t).rem_euclid(4);
            }
            let center_fixes = supercube::solve_center_orientations(centers)?;
            if !center_fixes.is_empty() {
                let start_idx = moves.len();
                moves.extend(center_fixes);
                phase_infos.push(PhaseInfo {
                    name: "センター向き解決".to_string(),
                    start: start_idx,
                    end: moves.len(),
                });
            }
        }
    }

    // 解法適用後の完成状態を無条件に検証
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
                    _ => -1,
                };
                final_centers[f] = (final_centers[f] + t).rem_euclid(4);
            }
            if final_centers.iter().any(|&c| c != 0) {
                return Err("センター向きの検証に失敗しました。".into());
            }
        }
    }

    Ok(result(
        &cube,
        state,
        &moves,
        start.elapsed().as_secs_f64() * 1000.0,
        total_nodes,
        algorithm,
        phase_infos,
    ))
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
/// 純粋な Rust 向けの内部・共通ロジック
pub fn apply_moves_core(state: &str, moves: &str) -> Result<ResultData, String> {
    let cube = cube::parse_state(state)?;
    let m = cube::parse_moves(moves)?;
    Ok(result(&cube, state, &m, 0.0, 0, "apply", Vec::new()))
}

/// 状態が合法であるかを検証し、完成状態（SOLVED）であれば true、
/// 合法だが未完成（スクランブル状態）であれば false、
/// 不正な配色・パリティであれば Err を返します。
pub fn validate_core(state: &str) -> Result<bool, String> {
    cube::parse_state(state).map(|c| c == coord::RawCube::default())
}

/// 状態が合法（回転可能で解法が存在する状態）であるかを検証します。
pub fn is_valid_core(state: &str) -> Result<bool, String> {
    cube::parse_state(state).map(|_| true)
}

/// 状態が完成状態（6面すべて揃っている状態）であるかを判定します。
pub fn is_solved_core(state: &str) -> Result<bool, String> {
    cube::parse_state(state).map(|c| c == coord::RawCube::default())
}

/// センタークォーターターンの総和パリティ（0 または 1）を返します。
/// コーナー置換パリティとセンター回転総和パリティの偶奇は常に一致する必要があります。
pub fn center_parity_core(state: &str) -> Result<u8, String> {
    cube::parse_state(state).map(|c| cube::parity(&c.cp.map(|p| p as u8)) as u8)
}

/// テーブルや各種パニックフックを初期化します。
#[wasm_bindgen]
pub fn initialize() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    let _ = tables::MoveTable::get();
    let _ = tables::PruningTable::get();
}

/// 状態が合法であるかを検証し、完成状態（SOLVED）であれば true、
/// 合法だが未完成（スクランブル状態）であれば false、
/// 不正な配色・パリティであれば Err を返します。
#[wasm_bindgen]
pub fn validate(state: &str) -> Result<bool, JsValue> {
    validate_core(state).map_err(to_js_error)
}

/// 状態が合法（回転可能で解法が存在する状態）であるかを検証します。
#[wasm_bindgen]
pub fn is_valid(state: &str) -> Result<bool, JsValue> {
    is_valid_core(state).map_err(to_js_error)
}

/// 状態が完成状態（6面すべて揃っている状態）であるかを判定します。
#[wasm_bindgen]
pub fn is_solved(state: &str) -> Result<bool, JsValue> {
    is_solved_core(state).map_err(to_js_error)
}

/// センタークォーターターンの総和パリティ（0 または 1）を返します。
/// 各面のクォーターターン（90°回転）は、この総和パリティとコーナー置換パリティの双方を反転させます。
#[wasm_bindgen]
pub fn center_parity(state: &str) -> Result<u8, JsValue> {
    center_parity_core(state).map_err(to_js_error)
}

/// 指定した局面に回転手順を適用し、結果の局面および遷移履歴を JSON で返します。
#[wasm_bindgen]
pub fn apply_moves(state: &str, moves: &str) -> Result<String, JsValue> {
    json(apply_moves_core(state, moves))
}

/// 指定した疑似乱数シードに基づき、冗長手を排除した25手のスクランブル文字列を生成します。
#[wasm_bindgen]
pub fn scramble(seed: u32) -> String {
    cube::scramble(seed)
        .iter()
        .map(|m| cube::notation(*m))
        .collect::<Vec<_>>()
        .join(" ")
}

/// 状態を解き、解法手順と各ステップの局面を含む JSON 文字列を返します。
#[wasm_bindgen]
pub fn solve(state: &str, budget_ms: u32) -> Result<String, JsValue> {
    json(solve_state(state, budget_ms, true))
}

/// センターの初期向きを考慮して解法を探索し、結果を JSON 文字列で返します。
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

/// 指定したアルゴリズム（kociemba, cfop, thistlethwaite, korf）を用いて解法を探索し、結果を JSON 文字列で返します。
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

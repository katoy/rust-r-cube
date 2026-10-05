pub mod coord;
pub mod cube;
pub mod lbl;
pub mod ortega;
pub mod search;
pub mod tables;

use crate::{PhaseInfo, ResultData};

/// 2x2 キューブを解き、解法手順と各ステップの盤面を含む `ResultData` を返します。
/// `algorithm`:
/// - "lbl": 初心者向け LBL法 (Layer-by-Layer: 完全1層 → OLL → PLL)
/// - "ortega": スピード解法 Ortega法 (1面色揃え → OLL → PBL)
/// - "optimal" (デフォルト): IDA* 最短探索 (最大11手)
pub fn solve(
    state: &str,
    _budget_ms: u32,
    include_orientation: bool,
    algorithm: &str,
) -> Result<ResultData, String> {
    let rc = cube::parse_state(state)?;
    let start = web_time::Instant::now();

    let (moves, phases, algo_name, nodes) = match algorithm {
        "lbl" => {
            let (m, p) = lbl::solve_lbl(&rc)?;
            (m, p, "lbl".to_string(), 0)
        }
        "ortega" => {
            let (m, p) = ortega::solve_ortega(&rc)?;
            (m, p, "ortega".to_string(), 0)
        }
        _ => {
            let mut searcher = search::Search::new();
            let m = searcher
                .solve(&rc, include_orientation)
                .ok_or_else(|| "解法が見つかりませんでした。".to_string())?;
            let p = vec![PhaseInfo {
                name: "Optimal 最短探索 (IDA*)".to_string(),
                start: 0,
                end: m.len(),
            }];
            (m, p, "optimal".to_string(), searcher.nodes)
        }
    };

    // ガードレール原則: Release ビルドでも無条件に解法を検証する
    let result_cube = cube::apply(&rc, &moves);
    let is_solved = if include_orientation {
        result_cube == coord::RawCube::default()
    } else {
        search::get_all_orientations()
            .iter()
            .any(|rot| rot.multiply(&result_cube) == coord::RawCube::default())
    };

    if !is_solved {
        return Err("解法の検証に失敗しました。".into());
    }

    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

    let mut current_cube = rc;
    let mut states = Vec::with_capacity(moves.len() + 1);
    states.push(state.to_owned());
    for m in &moves {
        current_cube = cube::apply(&current_cube, &[*m]);
        states.push(cube::facelets(&current_cube));
    }
    let final_state = states.last().cloned().unwrap_or_else(|| state.to_owned());

    Ok(ResultData {
        state: final_state,
        moves: moves.iter().map(|m| cube::notation(*m)).collect(),
        states,
        elapsed_ms,
        nodes,
        algorithm: algo_name,
        phases,
    })
}

/// 24文字の盤面が合法であるかを検証します。
pub fn is_valid(state: &str) -> Result<bool, String> {
    cube::parse_state(state).map(|_| true)
}

/// 24文字の盤面が完成状態であるかを判定します。
pub fn is_solved(state: &str, include_orientation: bool) -> Result<bool, String> {
    let rc = cube::parse_state(state)?;
    if include_orientation {
        Ok(rc == coord::RawCube::default())
    } else {
        Ok(search::get_all_orientations()
            .iter()
            .any(|rot| rot.multiply(&rc) == coord::RawCube::default()))
    }
}

/// 指定した局面に回転手順を適用し、結果の局面および遷移履歴を返します。
pub fn apply_moves_core(state: &str, moves_str: &str) -> Result<ResultData, String> {
    let rc = cube::parse_state(state)?;
    let moves = cube::parse_moves(moves_str)?;

    let mut current_cube = rc;
    let mut states = Vec::with_capacity(moves.len() + 1);
    states.push(state.to_owned());
    for m in &moves {
        current_cube = cube::apply(&current_cube, &[*m]);
        states.push(cube::facelets(&current_cube));
    }
    let final_state = states.last().cloned().unwrap_or_else(|| state.to_owned());

    Ok(ResultData {
        state: final_state,
        moves: moves.iter().map(|m| cube::notation(*m)).collect(),
        states,
        elapsed_ms: 0.0,
        nodes: 0,
        algorithm: "apply".to_string(),
        phases: Vec::new(),
    })
}

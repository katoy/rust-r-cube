use crate::cube::{Color, Move};
use crate::gui::app::{CubeApp, InputState};
use crate::gui::constants::*;

/// コントロールパネルを描画
pub fn draw_controls(app: &mut CubeApp, ui: &mut egui::Ui) {
    ui.heading("操作");
    ui.add_space(UI_SPACING_LARGE);

    // 6面スキャン入力モード
    ui.add_enabled_ui(!app.solving, |ui| {
        if let InputState::Scanning { face_index } = app.input_state {
            draw_scanning_ui(app, ui, face_index);
        } else {
            // 通常モード: 6面スキャンボタンを表示
            if ui.button("📸 6面スキャン入力").clicked() {
                app.start_scanning_mode();
            }
            ui.add_space(UI_SPACING_LARGE);
        }
    });

    // 基本操作、回転ボタン、ソルバー（探索中は制限）
    ui.add_enabled_ui(!app.solving, |ui| {
        draw_base_operations(app, ui);
        ui.add_space(UI_SPACING_LARGE);
        draw_rotation_buttons(app, ui);
        ui.add_space(UI_SPACING_LARGE);
    });

    draw_solver_ui(app, ui);

    ui.add_space(UI_SPACING_LARGE);

    // 状態表示
    let is_solved = if app.ignore_orientation {
        app.cube().is_solved()
    } else {
        crate::solver::is_fully_solved(app.cube())
    };

    if is_solved {
        ui.colored_label(egui::Color32::GREEN, "✓ 完成!");
    } else {
        ui.label("未完成");
    }
}

/// 6面スキャン入力モードのUIを描画
fn draw_scanning_ui(app: &mut CubeApp, ui: &mut egui::Ui, face_index: usize) {
    ui.separator();
    ui.heading("🎯 実物のキューブを入力中");
    ui.add_space(UI_SPACING_LARGE);

    // 進捗表示
    let progress = (face_index as f32 + 1.0) / 6.0;
    ui.add(egui::ProgressBar::new(progress).text(format!("{}/6 面", face_index + 1)));
    ui.add_space(UI_SPACING_SMALL);

    // 現在の面
    ui.label(format!("現在の面: {}", app.get_current_face_name()));
    ui.add_space(UI_SPACING_LARGE);

    // 色選択パレット
    ui.label("色を選択:");
    ui.horizontal(|ui| {
        let colors = [
            Color::White,
            Color::Yellow,
            Color::Green,
            Color::Blue,
            Color::Red,
            Color::Orange,
        ];

        for color in colors {
            let is_selected = app.selected_input_color == color;
            let rgb = crate::gui::renderer::color_to_color32(color);
            let label = match color {
                Color::White => "白",
                Color::Yellow => "黄",
                Color::Green => "緑",
                Color::Blue => "青",
                Color::Red => "赤",
                Color::Orange => "橙",
                _ => "?",
            };

            let button = egui::Button::new(label)
                .fill(rgb)
                .stroke(if is_selected {
                    egui::Stroke::new(INPUT_SELECTED_STROKE_WIDTH, egui::Color32::BLACK)
                } else {
                    egui::Stroke::new(INPUT_UNSELECTED_STROKE_WIDTH, egui::Color32::GRAY)
                })
                .min_size(egui::Vec2::from(INPUT_PALETTE_BUTTON_SIZE));

            if ui.add(button).clicked() {
                app.selected_input_color = color;
            }
        }
    });
    ui.add_space(UI_SPACING_LARGE);

    // ステッカーグリッド (3x3)
    ui.label("この面のステッカー:");
    ui.label("(クリックして選択した色を設定)");
    ui.add_space(UI_SPACING_SMALL);

    egui::Grid::new("sticker_grid")
        .spacing([UI_SPACING_SMALL, UI_SPACING_SMALL])
        .show(ui, |ui| {
            for row in 0..3 {
                for col in 0..3 {
                    let position = row * 3 + col;
                    let current_color = app.get_current_face_sticker(position);

                    let button_color = if let Some(color) = current_color {
                        crate::gui::renderer::color_to_color32(color)
                    } else {
                        egui::Color32::from_rgb(200, 200, 200) // 未設定
                    };

                    let button = egui::Button::new("")
                        .fill(button_color)
                        .stroke(egui::Stroke::new(
                            STICKER_STROKE_WIDTH,
                            egui::Color32::BLACK,
                        ))
                        .min_size(egui::Vec2::from(INPUT_STICKER_BUTTON_SIZE));

                    if ui.add(button).clicked() {
                        app.set_current_face_sticker(position, app.selected_input_color);
                    }
                }
                ui.end_row();
            }
        });

    ui.add_space(UI_SPACING_LARGE);

    // エラーメッセージ表示
    if !app.input_error_message.is_empty() {
        ui.colored_label(egui::Color32::RED, &app.input_error_message);
        ui.add_space(UI_SPACING_SMALL);
    }

    // ナビゲーションボタン
    ui.horizontal(|ui| {
        // 前の面へ
        ui.add_enabled_ui(face_index > 0, |ui| {
            if ui.button("◀ 前の面").clicked() {
                app.prev_face();
            }
        });

        // キャンセル
        if ui.button("❌ キャンセル").clicked() {
            app.cancel_scanning_mode();
        }

        // 次の面へ / 完了
        if face_index < 5 {
            let can_proceed = app.is_current_face_complete();
            ui.add_enabled_ui(can_proceed, |ui| {
                if ui.button("次の面 ▶").clicked() {
                    app.next_face();
                }
            });
        } else {
            // 最後の面
            let can_finish = app.is_current_face_complete();
            ui.add_enabled_ui(can_finish, |ui| {
                if ui.button("✅ 完了").clicked() {
                    app.finish_scanning();
                }
            });
        }
    });

    ui.separator();
    ui.add_space(UI_SPACING_LARGE);
}

/// 基本操作（スクランブル、リセット、ファイル）のUIを描画
fn draw_base_operations(app: &mut CubeApp, ui: &mut egui::Ui) {
    ui.label("基本操作:");
    ui.horizontal(|ui| {
        if ui.button("スクランブル").clicked() {
            app.scramble();
        }
        if ui.button("リセット").clicked() {
            app.reset();
        }
    });

    ui.add_space(UI_SPACING_LARGE);

    // ファイル保存・読み込み
    ui.label("ファイル:");
    ui.horizontal(|ui| {
        if ui.button("💾 保存").clicked() {
            app.save_with_dialog();
        }
        if ui.button("📂 読み込み").clicked() {
            app.load_with_dialog();
        }
    });

    ui.add_space(UI_SPACING_LARGE);

    // Undo/Redo
    ui.horizontal(|ui| {
        if ui
            .add_enabled(app.history.can_undo(), egui::Button::new("↶ Undo"))
            .clicked()
        {
            app.undo();
        }
        if ui
            .add_enabled(app.history.can_redo(), egui::Button::new("↷ Redo"))
            .clicked()
        {
            app.redo();
        }
    });

    ui.add_space(UI_SPACING_LARGE);

    // アニメーション制御
    ui.label("アニメーション:");
    ui.horizontal(|ui| {
        ui.label("速度:");
        ui.add(egui::Slider::new(&mut app.animation_speed, 0.0..=5.0).text("秒"));
    });
}

/// 回転操作ボタンの描画
fn draw_rotation_buttons(app: &mut CubeApp, ui: &mut egui::Ui) {
    ui.label("回転操作:");

    let move_groups = [
        (
            vec![Move::R, Move::Rp, Move::R2],
            vec![Move::L, Move::Lp, Move::L2],
        ),
        (
            vec![Move::U, Move::Up, Move::U2],
            vec![Move::D, Move::Dp, Move::D2],
        ),
        (
            vec![Move::F, Move::Fp, Move::F2],
            vec![Move::B, Move::Bp, Move::B2],
        ),
        (
            vec![Move::M, Move::Mp, Move::M2],
            vec![Move::E, Move::Ep, Move::E2],
        ),
        (
            vec![Move::S, Move::Sp, Move::S2],
            vec![Move::X, Move::Xp, Move::X2],
        ),
        (
            vec![Move::Y, Move::Yp, Move::Y2],
            vec![Move::Z, Move::Zp, Move::Z2],
        ),
    ];

    for (group1, group2) in move_groups {
        ui.horizontal(|ui| {
            for mv in group1 {
                if ui.button(format!("{}", mv)).clicked() {
                    app.queue_move(mv);
                }
            }
            ui.add_space(UI_SPACING_LARGE);
            for mv in group2 {
                if ui.button(format!("{}", mv)).clicked() {
                    app.queue_move(mv);
                }
            }
        });
    }
}

/// ソルバー関連のUIを描画
fn draw_solver_ui(app: &mut CubeApp, ui: &mut egui::Ui) {
    ui.label("ソルバー:");
    ui.horizontal(|ui| {
        ui.add_enabled_ui(!app.solving, |ui| {
            ui.radio_value(&mut app.ignore_orientation, true, "向き無視");
            ui.radio_value(&mut app.ignore_orientation, false, "向きも揃える");
        });
    });

    // 「解法を探す」ボタン（探索中は無効化）
    ui.horizontal(|ui| {
        ui.add_enabled_ui(!app.solving, |ui| {
            if ui.button("解法を探す").clicked() {
                app.solve();
            }
        });

        // 探索中のみ「中止」ボタンを表示
        if app.solving && ui.button("中止").clicked() {
            app.cancel_solve();
        }
    });

    // 探索中の進捗表示
    if app.solving {
        ui.label("探索中...");
        ui.add(egui::ProgressBar::new(app.solver_progress));

        if let Some(start_time) = app.solving_start_time {
            let elapsed = start_time.elapsed().as_secs_f32();
            let elapsed_display = (elapsed / 0.2).floor() * 0.2;
            ui.label(format!("経過: {:.1}秒", elapsed_display));
        }
    } else {
        // 探索終了後の結果表示
        if !app.solution_text.is_empty() {
            ui.add_space(UI_SPACING_SMALL);
            ui.label(&app.solution_text);
        }

        if let Some(solution) = app.solution.clone() {
            draw_solution_steps(app, ui, &solution);
        }
    }
}

/// 解法ステップ操作の描画
fn draw_solution_steps(app: &mut CubeApp, ui: &mut egui::Ui, solution: &[Move]) {
    let solution_len = solution.len();
    ui.add_space(UI_SPACING_LARGE);
    ui.label("解法ステップ操作:");

    ui.label(format!("ステップ: {}/{}", app.solution_step, solution_len));

    if !solution.is_empty() {
        ui.add_space(UI_SPACING_SMALL);
        ui.label("操作内容:");

        let moves_per_line = 10;
        for (i, chunk) in solution.chunks(moves_per_line).enumerate() {
            ui.horizontal(|ui| {
                for (j, &mv) in chunk.iter().enumerate() {
                    let global_idx = i * moves_per_line + j;
                    let move_text = format!("{}", mv);

                    if global_idx == app.solution_step && app.solution_step < solution_len {
                        ui.colored_label(
                            egui::Color32::from_rgb(255, 200, 0),
                            format!("[{}]", move_text),
                        );
                    } else if global_idx < app.solution_step {
                        ui.colored_label(egui::Color32::GRAY, move_text);
                    } else {
                        ui.label(move_text);
                    }
                }
            });
        }
    }

    ui.add_space(UI_SPACING_SMALL);
    if app.solution_step < solution_len {
        let next_move = solution[app.solution_step];
        ui.label(format!("次の動き: {}", next_move));
    } else {
        ui.colored_label(egui::Color32::GREEN, "完了!");
    }

    ui.add_space(UI_SPACING_SMALL);
    ui.horizontal(|ui| {
        if ui.button("⏮ 最初へ").clicked() {
            app.solution_step_reset();
        }
        ui.add_enabled_ui(app.solution_step > 0, |ui| {
            if ui.button("◀ 前へ").clicked() {
                app.solution_step_backward();
            }
        });
        ui.add_enabled_ui(app.solution_step < solution_len, |ui| {
            if ui.button("次へ ▶").clicked() {
                app.solution_step_forward();
            }
        });
        if ui.button("最後へ ⏭").clicked() {
            app.solution_step_to_end();
        }
    });

    ui.add_space(5.0);
    let progress = app.solution_step as f32 / solution_len as f32;
    ui.add(
        egui::ProgressBar::new(progress).text(format!("{}/{}", app.solution_step, solution_len)),
    );
}

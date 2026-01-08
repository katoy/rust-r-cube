use crate::cube::Move;
use crate::gui::app::CubeApp;

/// コントロールパネルを描画
pub fn draw_controls(app: &mut CubeApp, ui: &mut egui::Ui) {
    ui.heading("操作");

    ui.add_space(10.0);

    // 基本操作ボタン
    ui.label("基本操作:");
    ui.horizontal(|ui| {
        if ui.button("スクランブル").clicked() {
            app.scramble();
        }
        if ui.button("リセット").clicked() {
            app.reset();
        }
    });

    ui.add_space(10.0);

    // 回転ボタン
    ui.label("回転操作:");

    ui.horizontal(|ui| {
        if ui.button("R").clicked() {
            app.queue_move(Move::R);
        }
        if ui.button("R'").clicked() {
            app.queue_move(Move::Rp);
        }
        if ui.button("L").clicked() {
            app.queue_move(Move::L);
        }
        if ui.button("L'").clicked() {
            app.queue_move(Move::Lp);
        }
    });

    ui.horizontal(|ui| {
        if ui.button("U").clicked() {
            app.queue_move(Move::U);
        }
        if ui.button("U'").clicked() {
            app.queue_move(Move::Up);
        }
        if ui.button("D").clicked() {
            app.queue_move(Move::D);
        }
        if ui.button("D'").clicked() {
            app.queue_move(Move::Dp);
        }
    });

    ui.horizontal(|ui| {
        if ui.button("F").clicked() {
            app.queue_move(Move::F);
        }
        if ui.button("F'").clicked() {
            app.queue_move(Move::Fp);
        }
        if ui.button("B").clicked() {
            app.queue_move(Move::B);
        }
        if ui.button("B'").clicked() {
            app.queue_move(Move::Bp);
        }
    });

    ui.add_space(10.0);

    // アニメーション制御
    ui.label("アニメーション:");
    ui.horizontal(|ui| {
        ui.label("速度:");
        ui.add(egui::Slider::new(&mut app.animation_speed, 0.0..=5.0).text("秒"));
    });

    ui.add_space(10.0);

    // ソルバー
    ui.label("ソルバー:");
    ui.horizontal(|ui| {
        ui.radio_value(&mut app.ignore_orientation, true, "向き無視");
        ui.radio_value(&mut app.ignore_orientation, false, "向きも揃える");
    });

    if app.solving {
        // 探索中: プログレスバーと経過時間を表示
        ui.add(egui::ProgressBar::new(app.solver_progress));

        // 経過時間を表示（0.2秒ごとに更新）
        if let Some(start_time) = app.solving_start_time {
            let elapsed = start_time.elapsed().as_secs_f32();
            // 0.2秒単位で切り捨て
            let elapsed_display = (elapsed / 0.2).floor() * 0.2;
            ui.label(format!("経過: {:.1}秒", elapsed_display));
        }
    } else {
        // 探索中でない: ボタンを表示
        if ui.button("解法を探す").clicked() {
            app.solve();
        }
    }

    if !app.solution_text.is_empty() {
        ui.add_space(5.0);
        ui.label(&app.solution_text);
    }

    // 解法ステップ操作（探索中は非表示）
    if !app.solving && app.solution.is_some() {
        let solution = app.solution.as_ref().unwrap();
        let solution_len = solution.len();
        ui.add_space(10.0);
        ui.label("解法ステップ操作:");

        // 現在のステップ表示
        ui.label(format!("ステップ: {}/{}", app.solution_step, solution_len));

        // 全操作内容を表示
        if !solution.is_empty() {
            ui.add_space(5.0);
            ui.label("操作内容:");

            // 操作内容を複数行で表示（1行あたり最大10個）
            let moves_per_line = 10;
            for (i, chunk) in solution.chunks(moves_per_line).enumerate() {
                ui.horizontal(|ui| {
                    for (j, &mv) in chunk.iter().enumerate() {
                        let global_idx = i * moves_per_line + j;
                        let move_text = format!("{}", mv);

                        // 現在のステップを強調表示
                        if global_idx == app.solution_step && app.solution_step < solution_len {
                            ui.colored_label(
                                egui::Color32::from_rgb(255, 200, 0),
                                format!("[{}]", move_text),
                            );
                        } else if global_idx < app.solution_step {
                            // 実行済みのステップは薄く表示
                            ui.colored_label(egui::Color32::GRAY, move_text);
                        } else {
                            // 未実行のステップは通常表示
                            ui.label(move_text);
                        }
                    }
                });
            }
        }

        // 現在のステップの動き表示
        ui.add_space(5.0);
        if app.solution_step < solution_len {
            let next_move = solution[app.solution_step];
            ui.label(format!("次の動き: {}", next_move));
        } else if app.solution_step == solution_len {
            ui.colored_label(egui::Color32::GREEN, "完了!");
        }

        ui.add_space(5.0);

        // ステップ操作ボタン
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

        // プログレスバー
        let progress = app.solution_step as f32 / solution_len as f32;
        ui.add(
            egui::ProgressBar::new(progress)
                .text(format!("{}/{}", app.solution_step, solution_len)),
        );
    }

    ui.add_space(10.0);

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

use crate::cube::{Color, Move};
use crate::gui::app::{CubeApp, InputState};

/// コントロールパネルを描画
pub fn draw_controls(app: &mut CubeApp, ui: &mut egui::Ui) {
    ui.heading("操作");

    ui.add_space(10.0);

    // 6面スキャン入力モード
    ui.add_enabled_ui(!app.solving, |ui| {
        if let InputState::Scanning { face_index } = app.input_state {
            // スキャンモード中
            ui.separator();
            ui.heading("🎯 実物のキューブを入力中");
            ui.add_space(10.0);

            // 進捗表示
            let progress = (face_index as f32 + 1.0) / 6.0;
            ui.add(egui::ProgressBar::new(progress).text(format!("{}/6 面", face_index + 1)));
            ui.add_space(5.0);

            // 現在の面
            ui.label(format!("現在の面: {}", app.get_current_face_name()));
            ui.add_space(10.0);

            // 色選択パレット
            ui.label("色を選択:");
            ui.horizontal(|ui| {
                let colors = [
                    (Color::White, "白", egui::Color32::from_rgb(255, 255, 255)),
                    (Color::Yellow, "黄", egui::Color32::from_rgb(255, 255, 0)),
                    (Color::Green, "緑", egui::Color32::from_rgb(0, 200, 0)),
                    (Color::Blue, "青", egui::Color32::from_rgb(0, 100, 255)),
                    (Color::Red, "赤", egui::Color32::from_rgb(255, 0, 0)),
                    (Color::Orange, "橙", egui::Color32::from_rgb(255, 140, 0)),
                ];

                for (color, label, rgb) in colors {
                    let is_selected = app.selected_input_color == color;
                    let button = egui::Button::new(label)
                        .fill(rgb)
                        .stroke(if is_selected {
                            egui::Stroke::new(3.0, egui::Color32::BLACK)
                        } else {
                            egui::Stroke::new(1.0, egui::Color32::GRAY)
                        })
                        .min_size(egui::vec2(35.0, 30.0));

                    if ui.add(button).clicked() {
                        app.selected_input_color = color;
                    }
                }
            });
            ui.add_space(10.0);

            // ステッカーグリッド (2x2)
            ui.label("この面のステッカー:");
            ui.label("(クリックして選択した色を設定)");
            ui.add_space(5.0);

            egui::Grid::new("sticker_grid")
                .spacing([5.0, 5.0])
                .show(ui, |ui| {
                    for row in 0..2 {
                        for col in 0..2 {
                            let position = row * 2 + col;
                            let current_color = app.get_current_face_sticker(position);

                            let button_color = if let Some(color) = current_color {
                                match color {
                                    Color::White => egui::Color32::from_rgb(255, 255, 255),
                                    Color::Yellow => egui::Color32::from_rgb(255, 255, 0),
                                    Color::Green => egui::Color32::from_rgb(0, 200, 0),
                                    Color::Blue => egui::Color32::from_rgb(0, 100, 255),
                                    Color::Red => egui::Color32::from_rgb(255, 0, 0),
                                    Color::Orange => egui::Color32::from_rgb(255, 140, 0),
                                    Color::Gray => egui::Color32::from_rgb(180, 180, 180),
                                }
                            } else {
                                egui::Color32::from_rgb(200, 200, 200) // 未設定
                            };

                            let button = egui::Button::new("")
                                .fill(button_color)
                                .stroke(egui::Stroke::new(2.0, egui::Color32::BLACK))
                                .min_size(egui::vec2(50.0, 50.0));

                            if ui.add(button).clicked() {
                                app.set_current_face_sticker(position, app.selected_input_color);
                            }
                        }
                        ui.end_row();
                    }
                });

            ui.add_space(10.0);

            // エラーメッセージ表示
            if !app.input_error_message.is_empty() {
                ui.colored_label(egui::Color32::RED, &app.input_error_message);
                ui.add_space(5.0);
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
            ui.add_space(10.0);
        } else {
            // 通常モード: 6面スキャンボタンを表示
            if ui.button("📸 6面スキャン入力").clicked() {
                app.start_scanning_mode();
            }
            ui.add_space(10.0);
        }
    });

    // 基本操作ボタンなど（探索中は無効化）
    ui.add_enabled_ui(!app.solving, |ui| {
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

        ui.add_space(10.0);

        // 回転ボタン
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
        ];

        for (group1, group2) in move_groups {
            ui.horizontal(|ui| {
                for mv in group1 {
                    if ui.button(format!("{}", mv)).clicked() {
                        app.queue_move(mv);
                    }
                }
                ui.add_space(10.0);
                for mv in group2 {
                    if ui.button(format!("{}", mv)).clicked() {
                        app.queue_move(mv);
                    }
                }
            });
        }

        ui.add_space(5.0);

        // Undo/Redo ボタン
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

        ui.add_space(10.0);

        // アニメーション制御
        ui.label("アニメーション:");
        ui.horizontal(|ui| {
            ui.label("速度:");
            ui.add(egui::Slider::new(&mut app.animation_speed, 0.0..=5.0).text("秒"));
        });
    });

    ui.add_space(10.0);

    // ソルバー
    ui.label("ソルバー:");
    ui.horizontal(|ui| {
        ui.add_enabled_ui(!app.solving, |ui| {
            ui.radio_value(&mut app.ignore_orientation, true, "向き無視");
            ui.radio_value(&mut app.ignore_orientation, false, "向きも揃える");
        });
    });

    if app.solving {
        // 探索中: プログレスバーと経過時間を表示
        ui.horizontal(|ui| {
            ui.label("探索中...");
            if ui.button("中止").clicked() {
                app.cancel_solve();
            }
        });

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

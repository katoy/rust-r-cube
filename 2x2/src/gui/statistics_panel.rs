use crate::gui::app::CubeApp;
use crate::gui::constants::*;

pub fn draw_statistics(app: &CubeApp, ui: &mut egui::Ui) {
    ui.heading("📊 統計情報");
    ui.add_space(UI_SPACING_SMALL);

    egui::Grid::new("statistics_grid")
        .num_columns(2)
        .spacing([UI_SPACING_LARGE, UI_SPACING_SMALL])
        .show(ui, |ui| {
            ui.label("総解法回数:");
            ui.label(format!(
                "{} 回 (成功: {}/失敗: {})",
                app.statistics.total_solves,
                app.statistics.successful_solves,
                app.statistics.total_solves - app.statistics.successful_solves
            ));
            ui.end_row();

            if let Some(avg) = app.statistics.avg_solve_time() {
                ui.label("平均解法時間:");
                ui.label(format!("{:.2}秒", avg.as_secs_f64()));
                ui.end_row();
            }

            if let Some(best) = app.statistics.best_solve_time {
                ui.label("最速解法時間:");
                ui.label(format!("{:.2}秒", best.as_secs_f64()));
                ui.end_row();
            }

            ui.label("手動操作回数:");
            ui.label(format!("{} 回", app.statistics.total_manual_moves));
            ui.end_row();

            let session = app.statistics.session_duration();
            ui.label("セッション時間:");
            ui.label(format!("{:.0}分", session.as_secs_f64() / 60.0));
            ui.end_row();
        });
}

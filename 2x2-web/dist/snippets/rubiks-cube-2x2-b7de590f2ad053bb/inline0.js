
    export function run_solver_delayed(callback) {
        // プログレスバーを表示
        if (window.showSolverProgress) {
            window.showSolverProgress();
        }
        // 50ms遅延してソルバーを実行（DOM更新を保証）
        setTimeout(() => {
            callback();
            // プログレスバーを非表示
            if (window.hideSolverProgress) {
                window.hideSolverProgress();
            }
        }, 50);
    }

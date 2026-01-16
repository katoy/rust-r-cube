
    export function run_solver_delayed(callback) {
        if (window.showSolverProgress) {
            window.showSolverProgress();
        }
        setTimeout(() => {
            callback();
            if (window.hideSolverProgress) {
                window.hideSolverProgress();
            }
        }, 50);
    }
    export function show_solver_progress() {
        if (window.showSolverProgress) {
            window.showSolverProgress();
        }
    }
    export function hide_solver_progress() {
        if (window.hideSolverProgress) {
            window.hideSolverProgress();
        }
    }

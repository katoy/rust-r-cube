import { FACES } from "./model";

export interface KeyboardShortcutsOptions {
  isReady: () => boolean;
  isSolving?: () => boolean;
  getModifier: () => string;
  onMove: (move: string) => void | Promise<void>;
  onPlay: () => void | Promise<void>;
  onStop: () => void;
  onSeek: (step: number, animate?: boolean) => void | Promise<void>;
  getCurrentStep: () => number;
  getSolutionLength: () => number | undefined;
  onSuspend?: () => void;
}

export function clearActivePress(): void {
  document.querySelectorAll(".active-press").forEach((el) => {
    el.classList.remove("active-press");
  });
}

export function setupKeyboardShortcuts(
  options: KeyboardShortcutsOptions,
): () => void {
  const onKeyDown = (event: KeyboardEvent) => {
    if (
      event.ctrlKey ||
      event.metaKey ||
      event.altKey ||
      document.querySelector("dialog[open]") ||
      event.target instanceof HTMLInputElement ||
      event.target instanceof HTMLTextAreaElement ||
      event.target instanceof HTMLSelectElement
    ) {
      return;
    }

    if (!options.isReady() || event.defaultPrevented || options.isSolving?.()) {
      return;
    }

    const face = event.key.toUpperCase();
    if (FACES.includes(face) && face.length === 1) {
      const btn = document.querySelector(`button[data-move="${face}"]`);
      btn?.classList.add("active-press");
    } else if (event.key === "Shift") {
      document.getElementById("prime")?.classList.add("active-press");
    }

    if (FACES.includes(face) && face.length === 1) {
      event.preventDefault();
      if (!event.repeat) {
        void options.onMove(
          face + (event.shiftKey ? "'" : options.getModifier()),
        );
      }
    } else if (
      event.code === "Space" &&
      !(event.target instanceof HTMLButtonElement)
    ) {
      event.preventDefault();
      void options.onPlay();
    } else if (
      (event.key === "ArrowLeft" || event.key === "ArrowRight") &&
      !(event.target instanceof HTMLButtonElement)
    ) {
      event.preventDefault();
      options.onStop();
      void options.onSeek(
        options.getCurrentStep() + (event.key === "ArrowRight" ? 1 : -1),
      );
    } else if (event.key === "Home") {
      const total = options.getSolutionLength();
      if (total !== undefined) {
        event.preventDefault();
        options.onStop();
        void options.onSeek(0, false);
      }
    } else if (event.key === "End") {
      const total = options.getSolutionLength();
      if (total !== undefined) {
        event.preventDefault();
        options.onStop();
        void options.onSeek(total, false);
      }
    }
  };

  const onKeyUp = (event: KeyboardEvent) => {
    const face = event.key.toUpperCase();
    if (FACES.includes(face) && face.length === 1) {
      const btn = document.querySelector(`button[data-move="${face}"]`);
      btn?.classList.remove("active-press");
    } else if (event.key === "Shift") {
      document.getElementById("prime")?.classList.remove("active-press");
    }
  };

  const onVisibilityChange = () => {
    if (document.hidden) {
      clearActivePress();
      options.onSuspend?.();
    }
  };

  const onPageHide = () => {
    clearActivePress();
    options.onSuspend?.();
  };

  document.addEventListener("keydown", onKeyDown);
  document.addEventListener("keyup", onKeyUp);
  window.addEventListener("blur", clearActivePress);
  document.addEventListener("visibilitychange", onVisibilityChange);
  window.addEventListener("pagehide", onPageHide);

  return () => {
    document.removeEventListener("keydown", onKeyDown);
    document.removeEventListener("keyup", onKeyUp);
    window.removeEventListener("blur", clearActivePress);
    document.removeEventListener("visibilitychange", onVisibilityChange);
    window.removeEventListener("pagehide", onPageHide);
  };
}

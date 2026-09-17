import { FACES, FACE_NAMES, FACE_NAMES_EN, NAMES } from "./model";
const paths: Record<string, string> = {
  cube: "m12 3 9 5v9l-9 5-9-5V8Zm0 10v9M3 8l9 5 9-5M7.5 5.5l9 5v9",
  shuffle:
    "m3 5 3 0c5 0 7 14 12 14h3m-4-4 4 4-4 4M3 19h3c2 0 3-2 5-5m2-4c2-3 3-5 5-5h3m-4-4 4 4-4 4",
  arrow: "M4 12h16m-6-6 6 6-6 6",
  play: "m8 4 12 8-12 8Z",
  pause: "M8 4v16M16 4v16",
  back: "m15 5-7 7 7 7",
  next: "m9 5 7 7-7 7",
  first: "M6 5v14M18 5l-7 7 7 7",
  last: "M18 5v14M6 5l7 7-7 7",
  reset: "M3 11a9 9 0 1 1 2 7M3 3v8h8",
  copy: "M8 8h12v13H8ZM4 16H2V2h13v3",
  download: "M12 3v12m-5-5 5 5 5-5M4 16v5h16v-5",
  upload: "M12 16V4m-5 5 5-5 5 5M4 16v5h16v-5",
  close: "m6 6 12 12M6 18 18 6",
  check: "m5 12 4 4L19 6",
  undo: "M9 4 3 10l6 6M3 10h10a7 7 0 0 1 7 7",
  redo: "m15 4 6 6-6 6m6-6H11a7 7 0 0 0-7 7",
  eye: "M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12Zm13 0a3 3 0 1 1-6 0 3 3 0 0 1 6 0",
  help: "M9 8a3 3 0 1 1 4 3c-1 1-1 1-1 3m0 3v1M22 12a10 10 0 1 1-20 0 10 10 0 0 1 20 0",
};
export const icon = (name: string) =>
  `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="${paths[name] || paths.cube}"/></svg>`;
export function mount() {
  document.querySelector("#app")!.innerHTML = `
  <header class="topbar"><a class="brand" href="./" aria-label="Cube Studio ホーム">${icon("cube")}<span>CUBE<span class="brand-light"> STUDIO</span><small>ひと回し先が、見えてくる。</small></span></a><div class="top-actions"><span class="local-badge"><i></i> すべて、このブラウザで。</span><button id="help" class="icon-button" aria-label="使い方">${icon("help")}</button></div></header>
  <main>
    <div class="page-heading"><div><p class="eyebrow">YOUR CUBE. YOUR NEXT MOVE.</p><h1>揃うまで、ひと回しずつ。</h1><p class="lead">混ざったキューブに、見える道筋を。</p></div><span class="edition">01 — SOLVER <span>3 × 3 × 3</span></span></div>
    <div class="workspace">
      <section class="stage" aria-label="キューブの操作">
        <div class="stage-top"><span class="section-tag"><i></i> LIVE CUBE</span><span id="cube-status" class="status-pill">完成状態</span></div>
        <div id="scene"></div>
        <div id="fallback" hidden><p>3D表示を利用できないため、展開図で表示しています。</p><div id="fallback-net" class="cube-net"></div></div>
        <div class="stage-bottom"><span class="gesture">${icon("eye")} ドラッグで視点回転 · スクロールでズーム</span><button id="view-reset" class="text-button">${icon("reset")} 視点を戻す</button></div>
        <div class="manual-controls"><div class="control-label">ひと回しする <span>SHIFT で逆回転</span></div><div id="face-buttons">${[...FACES].map((f) => `<button class="move-button" data-move="${f}" title="${FACE_NAMES_EN[f]}" aria-label="${FACE_NAMES[f]}を回す">${f}</button>`).join("")}<button id="prime" class="modifier" aria-pressed="false" title="逆回転 (反時計回り 90°)" aria-label="逆回転">′</button><button id="double" class="modifier" aria-pressed="false" title="180度回転" aria-label="180度回転">2</button></div><div class="history-buttons"><button id="undo" class="icon-button" aria-label="元に戻す">${icon("undo")}</button><button id="redo" class="icon-button" aria-label="やり直す">${icon("redo")}</button><span></span><button id="reset" class="text-button">リセット</button></div></div>
      </section>
      <aside class="panel">
        <div class="panel-section setup"><div class="section-title"><span class="step-index">01</span><h2>キューブを準備</h2></div>
          <div class="tabs" role="tablist" aria-label="入力方法"><button role="tab" aria-selected="true" aria-controls="scramble-panel" id="tab-scramble" data-tab="scramble">スクランブル</button><button role="tab" aria-selected="false" aria-controls="presets-panel" id="tab-presets" data-tab="presets" tabindex="-1">プリセット</button><button role="tab" aria-selected="false" aria-controls="colors-panel" id="tab-colors" data-tab="colors" tabindex="-1">色を入力</button><button role="tab" aria-selected="false" aria-controls="moves-panel" id="tab-moves" data-tab="moves" tabindex="-1">手順を入力</button></div>
          <div id="scramble-panel" class="tab-panel" role="tabpanel" aria-labelledby="tab-scramble"><p>まずは混ぜて、解き方を見てみましょう。</p><button id="scramble" class="secondary wide">${icon("shuffle")} キューブを混ぜる <span>25手</span></button><div id="scramble-text" class="notation-preview">ランダムな回転で、新しい状態をつくります。</div></div>
          <div id="presets-panel" class="tab-panel" role="tabpanel" aria-labelledby="tab-presets" hidden><p>有名なキューブ状態を読み込むことができます。</p><div id="preset-buttons" class="preset-grid"></div><p id="preset-status" class="notation-preview">プリセットを読み込んでいます…</p></div>
          <div id="colors-panel" class="tab-panel" role="tabpanel" aria-labelledby="tab-colors" hidden><p>実物のキューブを、白が上・緑が前になるように持って入力します。</p><button id="edit-colors" class="secondary wide">${icon("cube")} 6面の色を入力 ${icon("arrow")}</button><button id="camera-colors" class="secondary wide">📷 2方向の画像から入力</button></div>
          <div id="moves-panel" class="tab-panel" role="tabpanel" aria-labelledby="tab-moves" hidden><label for="algorithm">回転記号を空白で区切って入力</label><textarea id="algorithm" placeholder="R U R' U'" maxlength="4096" spellcheck="false"></textarea><button id="apply-algorithm" class="secondary wide">現在の状態に適用 ${icon("arrow")}</button></div>
        </div>
        <div class="panel-section solving"><div class="section-title"><span class="step-index">02</span><h2>解き方を見つける</h2><span id="engine-status" class="engine-status">準備中</span></div><label class="solver-option"><input id="include-orientation" type="checkbox" checked /> 向きも含めて揃える</label><button id="solve" class="primary wide" disabled><span>解法を探す</span>${icon("arrow")}</button><button id="cancel" class="secondary wide" hidden>探索を中止</button><div class="solver-meta"><span id="solver-note">エンジンを読み込んでいます…</span><button id="extended" class="text-button" hidden>30秒で再探索</button></div><p id="message" role="status" aria-live="polite"></p></div>
        <div class="panel-section solution"><div class="section-title"><span class="step-index">03</span><h2>ひと回しずつ、揃える</h2><button id="copy" class="icon-button" aria-label="解法をコピー" disabled>${icon("copy")}</button></div>
          <div id="solution-empty"><div class="empty-orbit">${icon("cube")}</div><p>次の一手が、ここに。</p><small>解法を見つけると、回す面と手順を<br>3Dアニメーションで確認できます。</small></div>
          <div id="solution-content" hidden><div class="solution-stats"><strong id="move-count"></strong><span id="solve-time"></span></div><div id="move-list" aria-label="解法の各ステップ"></div><div class="next-move"><span id="next-symbol"></span><p id="next-instruction"></p></div><div class="playback"><button id="first" class="icon-button" aria-label="最初の手順へ">${icon("first")}</button><button id="prev" class="icon-button" aria-label="前の1手">${icon("back")}</button><button id="play" class="play-button" aria-label="自動再生">${icon("play")}</button><button id="next" class="icon-button" aria-label="次の1手">${icon("next")}</button><button id="last" class="icon-button" aria-label="最後の手順へ">${icon("last")}</button><span id="step-count"></span><label class="speed-label">速度<select id="speed" aria-label="再生速度"><option value="1000">0.5×</option><option value="500" selected>1×</option><option value="250">2×</option></select></label></div><input id="timeline" type="range" min="0" value="0" aria-label="再生位置" /></div>
        </div>
      </aside>
    </div>
    <div class="workspace-footer"><span>${icon("check")} 解法は、実際に揃うことを検証してから表示します。</span><div><button id="save" class="text-button">${icon("download")} 保存</button><button id="load" class="text-button">${icon("upload")} 読込</button><input id="file" type="file" accept=".json,application/json" hidden /></div></div>
    <footer><span>CUBE STUDIO <span class="footer-separator">/</span> A LITTLE ORDER IN THE CHAOS.</span><label><input id="reduced-motion" type="checkbox" /> 動きを減らす</label></footer>
  </main>
  <dialog id="editor"><div class="dialog-header"><div><p class="eyebrow">COLOR YOUR CUBE</p><h2>実物の色を、ここに。</h2></div><button id="editor-close" class="icon-button" aria-label="色入力を閉じる">${icon("close")}</button></div><p>白いセンターを上、緑を前に。各面を正面から見た色を入力してください。</p><div class="editor-layout"><div><div id="palette" aria-label="入力する色"></div><div id="editor-net" class="cube-net"></div><button id="clear-colors" class="text-button">センター以外を未入力にする</button></div><div class="face-guide"><div class="guide-head"><button id="guide-prev" class="icon-button" aria-label="前の面">${icon("back")}</button><strong id="guide-title"></strong><button id="guide-next" class="icon-button" aria-label="次の面">${icon("next")}</button></div><p id="guide-orientation"></p><div id="guide-grid" class="face-grid"></div><small>面を選んで、上のパレットの色で塗ります。</small></div></div><section class="center-input" aria-labelledby="center-heading"><h3 id="center-heading">センターの向き</h3><p>各面のガイドを正面から見て、上向きが0°、時計回りに90°ずつです。</p><div id="center-controls"></div><button id="auto-centers" class="secondary">配色に合う向きを自動設定</button><p>自動設定は解ける向きの一例です。実物の矢印は推定できないため、矢印付きキューブは実物に合わせて指定してください。</p><p id="center-mode" role="status"></p></section><p id="editor-error" role="status"></p><div class="dialog-footer"><span id="color-count"></span><button id="editor-apply" class="primary">この状態を使う ${icon("arrow")}</button></div></dialog>
  <dialog id="camera-editor"><div class="dialog-header"><div><p class="eyebrow">SCAN YOUR CUBE</p><h2>2方向の画像から入力</h2></div><button id="camera-close" class="icon-button" aria-label="画像入力を閉じる">${icon("close")}</button></div><p>対角に近い2方向から撮影し、上面のてっぺんから時計回りにキューブ外周の6角を指定します。</p><div class="camera-files"><label id="camera-drop-a" class="camera-file-card"><span class="file-card-title">画像A（上面・右面・前面）</span><span id="camera-status-a" class="file-status">未選択（クリックまたはドロップ）</span><input id="camera-file-a" type="file" accept="image/*" capture="environment"></label><label id="camera-drop-b" class="camera-file-card"><span class="file-card-title">画像B（下面・左面・背面）</span><span id="camera-status-b" class="file-status">未選択（クリックまたはドロップ）</span><input id="camera-file-b" type="file" accept="image/*" capture="environment"></label></div><div class="camera-view-tabs" role="tablist" aria-label="表示画像"><button id="camera-view-a" class="tab-button" type="button" aria-selected="true">画像A を表示</button><button id="camera-view-b" class="tab-button" type="button" aria-selected="false">画像B を表示</button></div><div class="camera-point-controls"><label>対象の画像<select id="camera-face"><option value="U">画像A（上面・右面・前面）</option><option value="D">画像B（下面・左面・背面）</option></select></label><div class="camera-point-buttons"><button id="camera-detect" class="secondary" type="button">角を自動検出</button><button id="camera-rotate-points" class="secondary" type="button" disabled>🔄 枠を回転</button><button id="camera-clear-points" class="text-button" type="button">角をクリア</button></div></div><canvas id="camera-canvas" width="640" height="480"></canvas><p id="camera-help" role="status">上面のてっぺんから時計回りにキューブ外周の6角をクリックするか、自動検出された角をドラッグして微調整してください。</p><button id="camera-capture" class="secondary" disabled>この3面を読み取る</button><section id="camera-result-section" class="camera-result-section" aria-label="読み取り結果の確認・補正"><div class="camera-result-header"><strong>読み取り結果の確認・補正</strong><span class="camera-result-hint">各セル（センター除く）をクリックして色を修正できます</span></div><div id="camera-palette" class="camera-palette" role="radiogroup" aria-label="補正用の色"></div><div id="camera-result-faces" class="cube-net camera-result-faces"></div></section><p id="camera-error" role="status"></p><div class="dialog-footer"><span id="camera-progress">0 / 6 面</span><button id="camera-apply" class="primary" disabled>色入力へ反映 ${icon("arrow")}</button></div></dialog>
  <dialog id="help-dialog"><div class="dialog-header"><h2>ひと回しずつ、使ってみよう。</h2><button id="help-close" class="icon-button" aria-label="使い方を閉じる">${icon("close")}</button></div><ol class="help-list"><li><strong>キューブを準備</strong><p>スクランブルで試すか、実物の6面の色を入力します。手順を入力して状態をつくることもできます。</p></li><li><strong>解法を探す</strong><p>探索はブラウザ内で完結します。配色は外部に送信されません。探索中も視点を動かせます。</p></li><li><strong>手順をたどる</strong><p>再生ボタンで自動再生、左右ボタンで1手ずつ確認。回転記号を選ぶと、その手の直後に移動します。</p></li></ol><div class="help-notation-table"><div class="help-notation-title">回転記号の早見表</div><table><thead><tr><th>記号</th><th>対象面</th><th>回転（面を正面から見た向き）</th></tr></thead><tbody><tr><td><strong>U</strong></td><td>上面 (Up)</td><td>時計回りに 90°</td></tr><tr><td><strong>R</strong></td><td>右面 (Right)</td><td>時計回りに 90°</td></tr><tr><td><strong>F</strong></td><td>前面 (Front)</td><td>時計回りに 90°</td></tr><tr><td><strong>D</strong></td><td>下面 (Down)</td><td>時計回りに 90°</td></tr><tr><td><strong>L</strong></td><td>左面 (Left)</td><td>時計回りに 90°</td></tr><tr><td><strong>B</strong></td><td>背面 (Back)</td><td>時計回りに 90°</td></tr><tr><td><strong>′ (プライム)</strong></td><td>—</td><td>逆回転（反時計回りに 90°）</td></tr><tr><td><strong>2</strong></td><td>—</td><td>180°回転（半回転）</td></tr></tbody></table></div><p class="help-note">U＝上、R＝右、F＝前、D＝下、L＝左、B＝後ろ。′ は逆回転、2 は180°です。時計回りは、回す面を正面から見た向きです。</p><p class="help-note">キーボード：U R F D L B で回転、Shiftで逆回転。Spaceで再生／停止、← →で前後の手順、Home / End で最初 / 最後にジャンプ。</p><p class="help-note">6色3×3に対応しています。センターの矢印方向は色入力画面で指定できます。高速な2段階探索を使用し、最短解は保証しません。</p></dialog>`;
}
export function net(
  host: HTMLElement,
  state: string,
  editable = false,
  onPaint?: (index: number) => void,
  active = -1,
  centers?: number[],
  errorIndices?: number[],
) {
  host.replaceChildren();
  const errorSet = new Set(errorIndices || []);
  [...FACES].forEach((face, f) => {
    const container = document.createElement("div");
    container.className = `net-face face-${face}`;
    const label = document.createElement("span");
    label.className = "net-label";
    label.textContent = `${face} · ${FACE_NAMES[face]}`;
    container.append(label);
    const grid = document.createElement("div");
    grid.className = "face-grid";
    for (let i = 0; i < 9; i++) {
      const cellIndex = f * 9 + i;
      const cell = document.createElement(editable ? "button" : "span");
      cell.className = "sticker";
      if (errorSet.has(cellIndex)) {
        cell.classList.add("is-error");
      }
      cell.dataset.index = String(cellIndex);
      cell.dataset.color = state[cellIndex];
      cell.textContent =
        i === 4 ? (centers ? ["↑", "→", "↓", "←"][centers[f]] : face) : "";
      if (active === f) cell.classList.add("active-face");
      cell.setAttribute(
        "aria-label",
        `${FACE_NAMES[face]} ${Math.floor(i / 3) + 1}行${(i % 3) + 1}列 ${NAMES[state[cellIndex]]}`,
      );
      if (editable) {
        (cell as HTMLButtonElement).disabled = i === 4;
        cell.onclick = () => onPaint?.(cellIndex);
      }
      grid.append(cell);
    }
    container.append(grid);
    host.append(container);
  });
}

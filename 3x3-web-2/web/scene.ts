import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { RoundedBoxGeometry } from "three/addons/geometries/RoundedBoxGeometry.js";
import { COLORS, FACES, getCellArrowInfo, ARROW_COLORS } from "./model";

const normal = [
  new THREE.Vector3(0, 1, 0),
  new THREE.Vector3(1, 0, 0),
  new THREE.Vector3(0, 0, 1),
  new THREE.Vector3(0, -1, 0),
  new THREE.Vector3(-1, 0, 0),
  new THREE.Vector3(0, 0, -1),
];

const faceUp = [
  new THREE.Vector3(0, 0, -1), // 0: U (奥方向)
  new THREE.Vector3(0, 1, 0), // 1: R (上)
  new THREE.Vector3(0, 1, 0), // 2: F (上)
  new THREE.Vector3(0, 0, 1), // 3: D (手前方向)
  new THREE.Vector3(0, 1, 0), // 4: L (上)
  new THREE.Vector3(0, 1, 0), // 5: B (上)
];
function position(face: number, row: number, col: number) {
  return new THREE.Vector3(
    ...([
      [col - 1, 1.49, row - 1],
      [1.49, 1 - row, 1 - col],
      [col - 1, 1 - row, 1.49],
      [col - 1, -1.49, 1 - row],
      [-1.49, 1 - row, col - 1],
      [1 - col, 1 - row, -1.49],
    ][face] as [number, number, number]),
  );
}
export class CubeScene {
  private renderer: THREE.WebGLRenderer;
  private scene = new THREE.Scene();
  private camera = new THREE.PerspectiveCamera(34, 1, 0.1, 100);
  private controls: OrbitControls;
  private root = new THREE.Group();
  private stickers: THREE.Mesh<
    THREE.BufferGeometry,
    THREE.MeshStandardMaterial
  >[] = [];
  private pieces: THREE.Object3D[] = [];
  private arrowGroup = new THREE.Group(); // 矢印グループ
  centerRotations: number[] = [0, 0, 0, 0, 0, 0];
  private centerLabels: THREE.Mesh[] = [];
  private active?: {
    layer: THREE.Group;
    axis: THREE.Vector3;
    angle: number;
    start: number;
    duration: number;
    finish: () => void;
  };
  private dirty = true;
  private observer: ResizeObserver;
  private next = "";

  constructor(private host: HTMLElement) {
    this.renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
    this.renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
    this.renderer.setClearColor(0, 0);
    this.renderer.outputColorSpace = THREE.SRGBColorSpace;
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = THREE.PCFShadowMap;
    this.renderer.domElement.setAttribute(
      "aria-label",
      "3Dキューブ。ドラッグで視点を回転できます。回転操作は下のボタンを使ってください。",
    );
    this.renderer.domElement.setAttribute("role", "img");
    this.host.append(this.renderer.domElement);
    this.scene.add(this.root, new THREE.HemisphereLight(0xfdf5df, 0x506977, 3));
    this.root.add(this.arrowGroup);
    const key = new THREE.DirectionalLight(0xfff3dc, 4);
    key.position.set(-3, 7, 5);
    key.castShadow = true;
    key.shadow.mapSize.set(1024, 1024);
    key.shadow.normalBias = 0.025;
    this.scene.add(key);
    const fill = new THREE.DirectionalLight(0xb4d5ed, 1.8);
    fill.position.set(5, 2, -3);
    this.scene.add(fill);
    const floor = new THREE.Mesh(
      new THREE.PlaneGeometry(200, 200),
      new THREE.ShadowMaterial({ opacity: 0.18 }),
    );
    floor.rotation.x = -Math.PI / 2;
    floor.position.y = -2.25;
    floor.receiveShadow = true;
    this.scene.add(floor);
    const body = new RoundedBoxGeometry(0.97, 0.97, 0.97, 3, 0.07);
    const bodyMaterial = new THREE.MeshStandardMaterial({
      color: 0x202522,
      roughness: 0.48,
    });
    for (let x = -1; x <= 1; x++)
      for (let y = -1; y <= 1; y++)
        for (let z = -1; z <= 1; z++) {
          if (x === 0 && y === 0 && z === 0) continue;
          const mesh = new THREE.Mesh(body, bodyMaterial);
          mesh.position.set(x, y, z);
          mesh.castShadow = true;
          mesh.receiveShadow = true;
          this.add(mesh);
        }
    const sticker = new RoundedBoxGeometry(0.855, 0.855, 0.026, 3, 0.048);
    for (let f = 0; f < 6; f++)
      for (let i = 0; i < 9; i++) {
        const mesh = new THREE.Mesh(
          sticker,
          new THREE.MeshStandardMaterial({
            color: COLORS[FACES[f]],
            roughness: 0.42,
            metalness: 0.02,
          }),
        );
        mesh.position.copy(position(f, Math.floor(i / 3), i % 3));
        mesh.quaternion.setFromUnitVectors(
          new THREE.Vector3(0, 0, 1),
          normal[f],
        );
        mesh.castShadow = true;
        mesh.receiveShadow = true;
        this.stickers.push(mesh);
        this.add(mesh);
        if (i === 4) {
          const canvas = document.createElement("canvas");
          canvas.width = 128;
          canvas.height = 128;
          const ctx = canvas.getContext("2d")!;
          ctx.fillStyle = "#16231f";
          ctx.font = "700 32px sans-serif";
          ctx.textAlign = "left";
          ctx.textBaseline = "top";
          ctx.fillText(FACES[f], 14, 12);
          const texture = new THREE.CanvasTexture(canvas);
          const label = new THREE.Mesh(
            new THREE.PlaneGeometry(0.855, 0.855),
            new THREE.MeshBasicMaterial({
              map: texture,
              transparent: true,
              depthWrite: false,
            }),
          );
          label.position.copy(mesh.position).addScaledVector(normal[f], 0.017);
          const n = normal[f];
          const baseUp = faceUp[f].clone();
          const zAxis = n.clone().normalize();
          const yAxis = baseUp.clone().normalize();
          const xAxis = new THREE.Vector3()
            .crossVectors(yAxis, zAxis)
            .normalize();
          const rotMatrix = new THREE.Matrix4().makeBasis(xAxis, yAxis, zAxis);
          label.quaternion.setFromRotationMatrix(rotMatrix);
          label.renderOrder = 9;
          this.centerLabels[f] = label;
          this.add(label);
        }
      }
    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    this.controls.addEventListener("change", () => {
      this.dirty = true;
    });
    this.controls.enablePan = false;
    this.controls.enableDamping = true;
    this.controls.minDistance = 6;
    this.controls.maxDistance = 15;
    this.controls.minPolarAngle = 0.12;
    this.controls.maxPolarAngle = Math.PI - 0.12;
    this.resetView();
    this.observer = new ResizeObserver(() => this.resize());
    this.observer.observe(host);
    this.resize();
    this.renderer.setAnimationLoop((time) => this.frame(time));
    this.renderer.domElement.addEventListener("webglcontextlost", (event) => {
      event.preventDefault();
      host.dispatchEvent(new Event("render-failed"));
    });
  }
  private add(mesh: THREE.Object3D) {
    mesh.userData.origin = mesh.position.clone();
    mesh.userData.rotation = mesh.quaternion.clone();
    this.pieces.push(mesh);
    this.root.add(mesh);
  }
  private resize() {
    this.dirty = true;
    const w = this.host.clientWidth,
      h = this.host.clientHeight;
    if (!w || !h) return;
    this.camera.aspect = w / h;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(w, h);
  }
  resetView() {
    this.camera.position.set(7, 5.5, 8.5);
    this.controls.target.set(0, 0, 0);
    this.controls.update();
  }
  dispose() {
    this.finish();
    this.renderer.setAnimationLoop(null);
    this.controls.dispose();
    this.observer.disconnect();
    this.scene.traverse((object) => {
      if (object instanceof THREE.Mesh) {
        object.geometry.dispose();
        const materials = Array.isArray(object.material)
          ? object.material
          : [object.material];
        for (const material of materials) {
          material.map?.dispose();
          material.dispose();
        }
      }
    });
    this.renderer.dispose();
  }
  show(state: string, next = "") {
    this.dirty = true;
    this.next = next;
    this.stickers.forEach((mesh, i) => {
      mesh.material.color.set(COLORS[state[i]] ?? COLORS["?"]);
      mesh.material.emissive.set(
        Math.floor(i / 9) === FACES.indexOf(next[0]) ? 0x1c2010 : 0x000000,
      );
    });
    // 矢印を更新
    this.updateArrows(state);
  }

  getArrowCount(): number {
    return this.getArrows().length;
  }

  getArrows(): THREE.Mesh[] {
    return (this.arrowGroup.children as THREE.Mesh[]).filter(
      (m) => m.userData?.stickerIdx !== undefined && !m.userData?.isOutline,
    );
  }

  applyMoveToCenters(move: string) {
    const f = FACES.indexOf(move[0]);
    if (f >= 0) {
      const dAngle = move.endsWith("2")
        ? Math.PI
        : move.endsWith("'")
          ? -Math.PI / 2
          : Math.PI / 2;
      this.centerRotations[f] =
        (this.centerRotations[f] + dAngle) % (2 * Math.PI);
      this.updateCenterLabels();
    }
  }

  resetCenterRotations() {
    this.centerRotations.fill(0);
    this.updateCenterLabels();
  }

  private updateCenterLabels() {
    for (let f = 0; f < 6; f++) {
      const label = this.centerLabels[f];
      if (!label) continue;
      const angle = this.centerRotations[f] ?? 0;
      const n = normal[f];
      const baseUp = faceUp[f].clone().applyAxisAngle(n, angle);
      const zAxis = n.clone().normalize();
      const yAxis = baseUp.clone().normalize();
      const xAxis = new THREE.Vector3().crossVectors(yAxis, zAxis).normalize();
      const rotMatrix = new THREE.Matrix4().makeBasis(xAxis, yAxis, zAxis);
      label.quaternion.setFromRotationMatrix(rotMatrix);
    }
  }

  private updateArrows(state?: string) {
    // 既存の矢印を削除
    this.arrowGroup.clear();
    if (!state || state.length < 54) return;

    this.updateCenterLabels();

    const arrowInfo = getCellArrowInfo(state, this.centerRotations);

    // セル表面に貼り付ける矢印の共通ジオメトリ
    const stemW = 0.05;
    const stemH = 0.16;
    const headW = 0.15;
    const headH = 0.22;
    const notch = 0.03;

    // 内側のカラー矢印
    const arrowShape = new THREE.Shape();
    arrowShape.moveTo(-stemW, -stemH);
    arrowShape.lineTo(stemW, -stemH);
    arrowShape.lineTo(stemW, notch);
    arrowShape.lineTo(headW, notch);
    arrowShape.lineTo(0, headH);
    arrowShape.lineTo(-headW, notch);
    arrowShape.lineTo(-stemW, notch);
    arrowShape.closePath();
    const arrowGeometry = new THREE.ShapeGeometry(arrowShape);

    // 外側の暗色アウトライン（あらゆるステッカー地色から矢印をくっきり際立たせる）
    const o = 0.022;
    const outlineShape = new THREE.Shape();
    outlineShape.moveTo(-(stemW + o), -(stemH + o));
    outlineShape.lineTo(stemW + o, -(stemH + o));
    outlineShape.lineTo(stemW + o, notch - o * 0.4);
    outlineShape.lineTo(headW + o * 1.3, notch - o * 0.4);
    outlineShape.lineTo(0, headH + o * 1.3);
    outlineShape.lineTo(-(headW + o * 1.3), notch - o * 0.4);
    outlineShape.lineTo(-(stemW + o), notch - o * 0.4);
    outlineShape.closePath();
    const outlineGeometry = new THREE.ShapeGeometry(outlineShape);

    const outlineMaterial = new THREE.MeshBasicMaterial({
      color: ARROW_COLORS.OUTLINE,
      transparent: true,
      opacity: 0.96,
      depthWrite: false,
      side: THREE.DoubleSide,
    });

    // 全54セルの矢印を配置（各セルの表面に印刷された向き・角度）
    for (let i = 0; i < 54; i++) {
      const stickerMesh = this.stickers[i];
      const faceIdx = Math.floor(i / 9);
      const n = normal[faceIdx];
      const angle = arrowInfo.angles[i];
      const color = arrowInfo.colors[i];
      const kind = arrowInfo.kinds[i];
      const pieceIdx = arrowInfo.pieceIndices[i];

      // 共通の回転姿勢（面の基準上方向から angle 回転）
      const baseUp = faceUp[faceIdx].clone();
      const targetDir = baseUp.applyAxisAngle(n, angle);
      const zAxis = n.clone().normalize();
      const yAxis = targetDir.clone().normalize();
      const xAxis = new THREE.Vector3().crossVectors(yAxis, zAxis).normalize();
      const rotMatrix = new THREE.Matrix4().makeBasis(xAxis, yAxis, zAxis);

      // 1. 暗色アウトラインメッシュ（背景ステッカーとの境界を明確化）
      const outlineMesh = new THREE.Mesh(outlineGeometry, outlineMaterial);
      outlineMesh.userData = { isOutline: true, stickerIdx: i };
      outlineMesh.position.copy(stickerMesh.position).addScaledVector(n, 0.015);
      outlineMesh.quaternion.setFromRotationMatrix(rotMatrix);
      outlineMesh.renderOrder = 10;
      this.arrowGroup.add(outlineMesh);

      // 2. 内側のカラー矢印メッシュ
      const mat = new THREE.MeshBasicMaterial({
        color,
        transparent: true,
        opacity: 1.0,
        depthWrite: false,
        side: THREE.DoubleSide,
      });

      const arrowMesh = new THREE.Mesh(arrowGeometry, mat);
      arrowMesh.userData = {
        stickerIdx: i,
        color,
        rotAngle: angle,
        kind,
        pieceIdx,
      };

      // セル表面から 0.017 浮かせ、アウトラインの前面に密着して表示
      arrowMesh.position.copy(stickerMesh.position).addScaledVector(n, 0.017);
      arrowMesh.quaternion.setFromRotationMatrix(rotMatrix);
      arrowMesh.renderOrder = 11;

      this.arrowGroup.add(arrowMesh);
    }
  }

  private getCenter(indices: number[]): THREE.Vector3 {
    const center = new THREE.Vector3();
    indices.forEach((idx) => {
      center.add(this.stickers[idx].position);
    });
    center.divideScalar(indices.length);
    return center;
  }
  async turn(move: string, state: string, duration: number) {
    this.finish();
    this.applyMoveToCenters(move);
    if (duration <= 0) {
      this.show(state, this.next);
      return;
    }
    this.arrowGroup.visible = false;
    const axis = normal[FACES.indexOf(move[0])];
    const layer = new THREE.Group();
    this.root.add(layer);
    this.pieces
      .filter((mesh) => mesh.position.dot(axis) > 0.5)
      .forEach((mesh) => layer.attach(mesh));
    const angle =
      ((move.endsWith("2") ? 2 : move.endsWith("'") ? -1 : 1) * -Math.PI) / 2;
    await new Promise<void>((resolve) => {
      this.active = {
        layer,
        axis,
        angle,
        start: performance.now(),
        duration,
        finish: () => {
          this.show(state, this.next);
          resolve();
        },
      };
    });
  }
  finish() {
    const active = this.active;
    if (!active) return;
    this.active = undefined;
    for (const mesh of this.pieces) {
      this.root.add(mesh);
      mesh.position.copy(mesh.userData.origin);
      mesh.quaternion.copy(mesh.userData.rotation);
    }
    this.root.remove(active.layer);
    this.arrowGroup.visible = true;
    active.finish();
  }
  private frame(time: number) {
    if (document.hidden) return;
    if (this.active) {
      this.dirty = true;
      const p = Math.min(1, (time - this.active.start) / this.active.duration);
      this.active.layer.quaternion.setFromAxisAngle(
        this.active.axis,
        this.active.angle * (p * p * (3 - 2 * p)),
      );
      if (p >= 1) this.finish();
    }
    this.controls.update();
    if (this.dirty) {
      this.renderer.render(this.scene, this.camera);
      this.dirty = false;
    }
  }
}

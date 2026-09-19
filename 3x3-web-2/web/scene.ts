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

function rotationMatrixForFaceAngle(
  faceIdx: number,
  clockwiseAngle: number,
): THREE.Matrix4 {
  const n = normal[faceIdx];
  // 外向き法線に対する右手系回転では時計回り回転角は -clockwiseAngle
  const targetDir = faceUp[faceIdx].clone().applyAxisAngle(n, -clockwiseAngle);
  const zAxis = n.clone().normalize();
  const yAxis = targetDir.clone().normalize();
  const xAxis = new THREE.Vector3().crossVectors(yAxis, zAxis).normalize();
  return new THREE.Matrix4().makeBasis(xAxis, yAxis, zAxis);
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
  private arrowGeometry: THREE.ShapeGeometry;
  private outlineGeometry: THREE.ShapeGeometry;
  private outlineMaterial: THREE.MeshBasicMaterial;
  private colorMaterials = new Map<number, THREE.MeshBasicMaterial>();
  private outlineMeshes: THREE.Mesh<
    THREE.ShapeGeometry,
    THREE.MeshBasicMaterial
  >[] = [];
  private arrowMeshes: THREE.Mesh<
    THREE.ShapeGeometry,
    THREE.MeshBasicMaterial
  >[] = [];
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
          const rotMatrix = rotationMatrixForFaceAngle(f, 0);
          label.quaternion.setFromRotationMatrix(rotMatrix);
          label.renderOrder = 9;
          this.centerLabels[f] = label;
          this.add(label);
        }
      }

    // 矢印用ジオメトリとアウトラインマテリアルを事前生成（再利用でGPUメモリリーク防止）
    const stemW = 0.05;
    const stemH = 0.16;
    const headW = 0.15;
    const headH = 0.22;
    const notch = 0.03;

    const arrowShape = new THREE.Shape();
    arrowShape.moveTo(-stemW, -stemH);
    arrowShape.lineTo(stemW, -stemH);
    arrowShape.lineTo(stemW, notch);
    arrowShape.lineTo(headW, notch);
    arrowShape.lineTo(0, headH);
    arrowShape.lineTo(-headW, notch);
    arrowShape.lineTo(-stemW, notch);
    arrowShape.closePath();
    this.arrowGeometry = new THREE.ShapeGeometry(arrowShape);

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
    this.outlineGeometry = new THREE.ShapeGeometry(outlineShape);

    this.outlineMaterial = new THREE.MeshBasicMaterial({
      color: ARROW_COLORS.OUTLINE,
      transparent: true,
      opacity: 0.96,
      depthWrite: false,
      side: THREE.DoubleSide,
    });

    const defaultColorMat = this.getColorMaterial(ARROW_COLORS.NORMAL);
    for (let i = 0; i < 54; i++) {
      const outlineMesh = new THREE.Mesh(
        this.outlineGeometry,
        this.outlineMaterial,
      );
      outlineMesh.userData = { isOutline: true, stickerIdx: i };
      outlineMesh.renderOrder = 10;
      this.outlineMeshes.push(outlineMesh);
      this.arrowGroup.add(outlineMesh);

      const arrowMesh = new THREE.Mesh(this.arrowGeometry, defaultColorMat);
      arrowMesh.userData = { stickerIdx: i };
      arrowMesh.renderOrder = 11;
      this.arrowMeshes.push(arrowMesh);
      this.arrowGroup.add(arrowMesh);
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
    this.setViewPreset("iso");
  }
  setViewPreset(preset: "iso" | "front" | "top" | "right") {
    switch (preset) {
      case "iso":
        this.camera.position.set(7, 5.5, 8.5);
        break;
      case "front":
        this.camera.position.set(0, 0, 11.5);
        break;
      case "top":
        this.camera.position.set(0, 11.5, 0.001);
        break;
      case "right":
        this.camera.position.set(11.5, 0, 0);
        break;
    }
    this.controls.target.set(0, 0, 0);
    this.controls.update();
  }
  private getColorMaterial(color: number): THREE.MeshBasicMaterial {
    let mat = this.colorMaterials.get(color);
    if (!mat) {
      mat = new THREE.MeshBasicMaterial({
        color,
        transparent: true,
        opacity: 1.0,
        depthWrite: false,
        side: THREE.DoubleSide,
      });
      this.colorMaterials.set(color, mat);
    }
    return mat;
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
    this.arrowGeometry?.dispose();
    this.outlineGeometry?.dispose();
    this.outlineMaterial?.dispose();
    this.colorMaterials.forEach((mat) => mat.dispose());
    this.colorMaterials.clear();
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
    return this.arrowMeshes.length;
  }

  getArrows(): THREE.Mesh[] {
    return this.arrowMeshes;
  }

  private updateCenterLabels() {
    for (let f = 0; f < 6; f++) {
      const label = this.centerLabels[f];
      if (!label) continue;
      const angle = this.centerRotations[f] ?? 0;
      const rotMatrix = rotationMatrixForFaceAngle(f, angle);
      label.quaternion.setFromRotationMatrix(rotMatrix);
    }
  }

  private updateArrows(state?: string) {
    if (!state || state.length < 54) {
      this.arrowGroup.visible = false;
      return;
    }
    this.arrowGroup.visible = true;

    this.updateCenterLabels();

    const arrowInfo = getCellArrowInfo(state, this.centerRotations);

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
      const rotMatrix = rotationMatrixForFaceAngle(faceIdx, angle);

      // 1. 暗色アウトラインメッシュ
      const outlineMesh = this.outlineMeshes[i];
      outlineMesh.position.copy(stickerMesh.position).addScaledVector(n, 0.015);
      outlineMesh.quaternion.setFromRotationMatrix(rotMatrix);
      outlineMesh.userData.origin = outlineMesh.position.clone();
      outlineMesh.userData.rotation = outlineMesh.quaternion.clone();

      // 2. 内側のカラー矢印メッシュ（マテリアルはキャッシュを再利用）
      const arrowMesh = this.arrowMeshes[i];
      arrowMesh.material = this.getColorMaterial(color);
      arrowMesh.position.copy(stickerMesh.position).addScaledVector(n, 0.017);
      arrowMesh.quaternion.setFromRotationMatrix(rotMatrix);
      arrowMesh.userData = {
        stickerIdx: i,
        color,
        rotAngle: angle,
        kind,
        pieceIdx,
        origin: arrowMesh.position.clone(),
        rotation: arrowMesh.quaternion.clone(),
      };
    }
  }

  async turn(move: string, state: string, duration: number) {
    this.finish();
    if (duration <= 0) {
      this.show(state, this.next);
      return;
    }
    const axis = normal[FACES.indexOf(move[0])];
    const layer = new THREE.Group();
    this.root.add(layer);
    this.pieces
      .filter((mesh) => mesh.position.dot(axis) > 0.5)
      .forEach((mesh) => layer.attach(mesh));

    // 回転する層に属する矢印（カラー矢印およびアウトライン）を layer に attach
    const movingArrows = (this.arrowGroup.children as THREE.Object3D[]).filter(
      (mesh) => {
        const pos = new THREE.Vector3();
        mesh.getWorldPosition(pos);
        return pos.dot(axis) > 0.5;
      },
    );
    movingArrows.forEach((mesh) => layer.attach(mesh));

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
    for (const mesh of this.outlineMeshes) {
      this.arrowGroup.add(mesh);
      if (mesh.userData.origin) {
        mesh.position.copy(mesh.userData.origin);
        mesh.quaternion.copy(mesh.userData.rotation);
      }
    }
    for (const mesh of this.arrowMeshes) {
      this.arrowGroup.add(mesh);
      if (mesh.userData.origin) {
        mesh.position.copy(mesh.userData.origin);
        mesh.quaternion.copy(mesh.userData.rotation);
      }
    }
    this.root.remove(active.layer);
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

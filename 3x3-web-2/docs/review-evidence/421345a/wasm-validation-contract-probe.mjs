import { readFileSync } from "node:fs";
import {
  initSync,
  validate,
  is_valid,
  is_solved,
  apply_moves,
} from "../../../pkg/cube_studio.js";

initSync({
  module: readFileSync(
    new URL("../../../pkg/cube_studio_bg.wasm", import.meta.url),
  ),
});
const solved = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
const moved = JSON.parse(apply_moves(solved, "R")).state;
for (const [name, state] of [
  ["SOLVED", solved],
  ["R", moved],
  ["INVALID", "INVALID"],
]) {
  for (const [api, f] of [
    ["validate", validate],
    ["is_valid", is_valid],
    ["is_solved", is_solved],
  ]) {
    try {
      const result = f(state);
      console.log(`${name} ${api}=${result} typeof=${typeof result}`);
    } catch (error) {
      console.log(`${name} ${api}=THROW ${String(error)}`);
    }
  }
}

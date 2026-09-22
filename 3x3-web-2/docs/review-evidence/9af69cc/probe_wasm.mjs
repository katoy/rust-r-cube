// Independent geometric sticker model checks WASM moves and UI arrow angles.
// No use of Rust piece tables or web/model CORNERS/EDGES in the oracle.
import { readFileSync } from 'node:fs';
import assert from 'node:assert/strict';
import { initSync, initialize, apply_moves, scramble, solve_with_algorithm } from '../../../pkg/cube_studio.js';
import { getCellArrowInfo } from '../../../web/model.ts';

initSync({ module: readFileSync(new URL('../../../pkg/cube_studio_bg.wasm', import.meta.url)) });
initialize();
const faces = 'URFDLB';
const solved = [...faces].map(f => f.repeat(9)).join('');
const normals = [[0,1,0],[1,0,0],[0,0,1],[0,-1,0],[-1,0,0],[0,0,-1]];
const ups = [[0,0,-1],[0,1,0],[0,1,0],[0,0,1],[0,1,0],[0,1,0]];
const dot = (a,b) => a.reduce((s,x,i) => s+x*b[i],0);
const cross = (a,b) => [a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]];
const rights = ups.map((u,i) => cross(u,normals[i]));
const quarter = (v,n) => cross(v,n).map((x,i) => x+n[i]*dot(v,n));
const geometry = Array.from({length:54}, (_,i) => {
  const f=Math.floor(i/9), r=Math.floor(i%9/3), c=i%3;
  return { p:normals[f].map((n,k)=>n+rights[f][k]*(c-1)+ups[f][k]*(1-r)), n:[...normals[f]], up:[...ups[f]], color:faces[f] };
});
const slotKey = sticker => `${sticker.p.join(',')}:${sticker.n.join(',')}`;
const slots = geometry.map(slotKey);
function turnPhysical(stickers, move) {
  const face=faces.indexOf(move[0]), n=normals[face], turns=move.endsWith('2')?2:move.endsWith("'")?3:1;
  for(const sticker of stickers) {
    if(dot(sticker.p,n)!==1) continue;
    for(let t=0;t<turns;t++) {
      sticker.p=quarter(sticker.p,n);
      sticker.n=quarter(sticker.n,n);
      sticker.up=quarter(sticker.up,n);
    }
  }
}
function ordered(stickers) {
  const bySlot=new Map(stickers.map(s=>[slotKey(s),s]));
  return slots.map(k=>bySlot.get(k));
}
const report=(name,data)=>console.log(JSON.stringify({name,...data}));
let state=solved;
const stickers=structuredClone(geometry);
const centers=[0,0,0,0,0,0];
let rng=20260923;
for(let step=0;step<3000;step++) {
  rng^=rng<<13; rng^=rng>>>17; rng^=rng<<5;
  const m=(rng>>>0)%18, f=Math.floor(m/3), turn=m%3+1;
  const move=faces[f]+['','2',"'"][m%3];
  turnPhysical(stickers,move);
  centers[f]=(centers[f]+turn)%4;
  state=JSON.parse(apply_moves(state,move)).state;
  const view=ordered(stickers);
  assert.equal(state,view.map(s=>s.color).join(''),`physical colors, step ${step}`);
  const info=getCellArrowInfo(state,centers.map(c=>c*Math.PI/2));
  for(let i=0;i<54;i++) {
    const face=Math.floor(i/9), angle=Math.atan2(dot(view[i].up,rights[face]),dot(view[i].up,ups[face]));
    assert(Math.abs(Math.sin(info.angles[i]-angle))<1e-9 && Math.cos(info.angles[i]-angle)>0,`arrow ${i}, step ${step}`);
  }
}
report('physical_sticker_oracle',{seed:20260923,moves:3000,colorChecks:3000,arrowChecks:3000*54,mismatches:0});

for(const algorithm of ['kociemba','cfop','thistlethwaite','korf']) {
  for(const orientation of [false,true]) {
    let success=0, rejected=0, maxMs=0, zeroNodes=0;
    for(let seed=1;seed<=25;seed++) {
      const scrambleMoves=scramble(seed).split(' ');
      const input=JSON.parse(apply_moves(solved,scrambleMoves.join(' '))).state;
      const turns=[0,0,0,0,0,0];
      for(const move of scrambleMoves) turns[faces.indexOf(move[0])]=(turns[faces.indexOf(move[0])]+(move.endsWith('2')?2:move.endsWith("'")?3:1))%4;
      // Vary center orientations while retaining legal parity.
      turns[seed%6]=(turns[seed%6]+2)%4;
      const before=performance.now();
      let result;
      try { result=JSON.parse(solve_with_algorithm(input,500,orientation,turns.join(','),algorithm)); }
      catch(error) { rejected++; report('solver_rejected',{algorithm,orientation,seed,error:String(error)}); continue; }
      maxMs=Math.max(maxMs,performance.now()-before);
      const replay=JSON.parse(apply_moves(input,result.moves.join(' ')));
      assert.equal(replay.state,solved);
      assert.deepEqual(result.states,replay.states);
      assert.equal(result.states.length,result.moves.length+1);
      let end=0;
      for(const phase of result.phases) { assert.equal(phase.start,end); assert(phase.end>=phase.start); end=phase.end; }
      assert.equal(end,result.moves.length);
      if(orientation) {
        for(const move of result.moves) turns[faces.indexOf(move[0])]=(turns[faces.indexOf(move[0])]+(move.endsWith('2')?2:move.endsWith("'")?3:1))%4;
        assert.deepEqual(turns,[0,0,0,0,0,0]);
      }
      if(result.nodes===0) zeroNodes++;
      success++;
    }
    report('wasm_solver_sweep',{algorithm,orientation,seeds:'1..25',budgetMs:500,success,rejected,zeroNodes,maxMs:Number(maxMs.toFixed(3))});
  }
}

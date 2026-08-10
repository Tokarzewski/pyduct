#!/usr/bin/env node
// Embeds the venti WASM core in Node:
//  1. pure scalar math via C-ABI exports,
//  2. a full network built through the handle-based API, solved, and its
//     per-component results read back (issue #12 surface).
//
// Usage:
//   ./scripts/build-wasm.sh --release
//   node host/wasm_node_example.js target/wasm32-wasip1/release/venti.wasm
const { readFileSync } = require('fs');
const { WASI } = require('node:wasi');

async function main(path) {
  const wasm = readFileSync(path);
  const wasi = new WASI({ version: 'preview1' });
  const { instance } = await WebAssembly.instantiate(wasm, {
    wasi_snapshot_preview1: wasi.wasiImport,
  });
  // Initialize the WASI environment so host-gettable services (e.g. the
  // `random_get` that std's HashMap RandomState needs) are available.
  try { wasi.initialize(instance); } catch (e) { wasi.start(instance); }
  const m = instance.exports;

  // ---- 1. scalar math (f64 returns + out-params) ----
  console.log('friction_factor(5e4, 9e-4)      =', m.venti_friction_factor(50000, 0.0009).toFixed(6));
  console.log('standard_air_density            =', m.venti_standard_air_density());

  const addr = m.memory.buffer.byteLength - 16;
  const out = new Float64Array(m.memory.buffer, addr, 2);
  let st = m.venti_velocity_method_round(0.1, 4.0, addr, addr + 8);
  console.log('velocity_method_round(0.1, 4.0) -> status', st,
    '| D =', out[0].toFixed(4), 'm | v =', out[1].toFixed(4), 'm/s');

  // ---- 2. network via the handle API ----
  // Tiny allocator on top of the WASM heap + memory view.
  function bytesView(ptr, len) { return new Uint8Array(m.memory.buffer, ptr, len); }
  function writeString(s) {
    const n = s.length;
    const ptr = m.venti_alloc(n);
    const b = new TextEncoder().encode(s);
    bytesView(ptr, n).set(b);
    return { ptr, len: n };
  }
  function paramsArray(vals) {
    const ptr = m.venti_alloc(6 * 8);
    new Float64Array(m.memory.buffer, ptr, 6).set(vals);
    return ptr;
  }

  const name = writeString('Supply');
  const net = m.venti_network_create(name.ptr, name.len);
  m.venti_free(name.ptr, name.len);

  const records = [
    ['ahu', 'AHU', 0, [0, 0, 0, 0, 0, 0]],
    ['duct', 'Main Duct', 2, [Math.PI * 0.01, 0.2, 20.0, 0.0001, 0, 0]],
    ['fit', 'Elbow', 4, [Math.PI * 0.01, 0.5, 0, 0, 0, 0]],
    ['term', 'Terminal', 1, [Math.PI * 0.01, 1.0, 0.1, 0, 0, 0]],
  ];
  for (const [id, nm, ctype, prms] of records) {
    const a = writeString(id); const b = writeString(nm); const p = paramsArray(prms);
    st = m.venti_network_add(net, a.ptr, a.len, b.ptr, b.len, ctype, p);
    if (st !== 0) throw new Error(`add ${id} failed: ${st}`);
    m.venti_free(a.ptr, a.len); m.venti_free(b.ptr, b.len); m.venti_free(p, 48);
  }
  for (const [s, t] of [['ahu','duct'], ['duct','fit'], ['fit','term']]) {
    const a = writeString(s); const b = writeString(t);
    st = m.venti_network_connect(net, a.ptr, a.len, b.ptr, b.len);
    if (st !== 0) throw new Error(`connect ${s}->${t} failed: ${st}`);
    m.venti_free(a.ptr, a.len); m.venti_free(b.ptr, b.len);
  }

  console.log('\ncomponents:', m.venti_network_component_count(net));
  console.log('critical-path ΔP =', m.venti_network_solve(net, 1.204, 1.825e-5).toFixed(4), 'Pa');

  const n = m.venti_results_count(net);
  console.log('\nresults rows:', n);
  for (let i = 0; i < n; i++) {
    const row = new Float64Array(m.memory.buffer, m.memory.buffer.byteLength - 64, 1);
    const fin = new Int32Array(m.memory.buffer, m.memory.buffer.byteLength - 24, 1);
    const vin = new Int32Array(m.memory.buffer, m.memory.buffer.byteLength - 20, 1);
    const q = m.memory.buffer.byteLength - 64;
    const v = m.memory.buffer.byteLength - 56;
    const ps = m.memory.buffer.byteLength - 48;
    const fset = m.memory.buffer.byteLength - 24;
    const vset = m.memory.buffer.byteLength - 20;
    m.venti_results_row(net, i, q, fset, v, vset, ps);
    const buf = m.venti_alloc(64);
    const olenBuf = new Int32Array(m.memory.buffer, m.memory.buffer.byteLength - 4, 1);
    m.venti_results_field_string(net, i, 0, buf, 64, m.memory.buffer.byteLength - 4);
    const id = new TextDecoder().decode(bytesView(buf, olenBuf[0]));
    m.venti_free(buf, 64);
    console.log(`  ${id.padEnd(6)} Q_in=${fin[0] ? row[0].toFixed(3) : '—'} m3/s | ΔP=${new Float64Array(m.memory.buffer, ps, 1)[0].toFixed(3)} Pa`);
    void vin;
  }

  m.venti_network_free(net);
}

main(process.argv[2] || 'target/wasm32-wasip1/release/venti.wasm')
  .catch((e) => { console.error('FAIL', e); process.exit(1); });

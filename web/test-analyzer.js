/**
 * Node.js Test Suite for web/js/analyzer.js and visualizer.js
 * Verifies that the JS analyzer correctly parses existing fixtures and generates CFGs.
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { LlvmAnalyzer } from './js/analyzer.js';
import { LlvmVisualizer } from './js/visualizer.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

function runTests() {
  console.log('--- Testing Web LLVM Analyzer & Visualizer ---');

  const analyzer = new LlvmAnalyzer({ targetVersion: 18 });
  const visualizer = new LlvmVisualizer({ layout: 'TD' });

  // Test 1: Branching CFG fixture
  const branchingPath = path.resolve(__dirname, '../tests/fixtures/branching_cfg.ll');
  const branchingCode = fs.readFileSync(branchingPath, 'utf8');

  const res1 = analyzer.analyze(branchingCode);
  console.log(`✓ Fixture branching_cfg.ll analyzed.`);
  console.log(`  Functions discovered: ${res1.functions.map(f => f.name).join(', ')}`);
  
  if (!res1.cfgs['collatz']) {
    throw new Error('Expected function @collatz to be parsed into CFG');
  }

  const collatzCfg = res1.cfgs['collatz'];
  console.log(`  Basic blocks in @collatz: ${collatzCfg.blockOrder.join(' -> ')}`);
  
  const expectedBlocks = ['entry', 'loop', 'even', 'odd', 'loop_next', 'done'];
  for (const b of expectedBlocks) {
    if (!collatzCfg.blocks[b]) {
      throw new Error(`Missing expected basic block '${b}'`);
    }
  }

  // Check Mermaid output
  const mermaid = visualizer.toMermaid(collatzCfg);
  if (!mermaid.includes('flowchart TD') || !mermaid.includes('-->|true|') || !mermaid.includes('-->|false|')) {
    throw new Error('Mermaid output does not contain expected conditional branches');
  }
  console.log(`✓ Mermaid flowchart generated correctly (${mermaid.split('\n').length} lines).`);

  // Check DOT output
  const dot = visualizer.toDot(collatzCfg);
  if (!dot.includes('digraph') || !dot.includes('"entry"') || !dot.includes('"done"')) {
    throw new Error('DOT output does not contain expected digraph structures');
  }
  console.log(`✓ Graphviz DOT generated correctly.`);

  // Test 2: Opaque pointer fixture with LLVM 18
  const opaquePath = path.resolve(__dirname, '../tests/fixtures/valid_opaque.ll');
  const opaqueCode = fs.readFileSync(opaquePath, 'utf8');
  const res2 = analyzer.analyze(opaqueCode);
  console.log(`✓ Fixture valid_opaque.ll analyzed: ${res2.functions.map(f => f.name).join(', ')}`);
  if (!res2.cfgs['calculate']) {
    throw new Error('Expected function @calculate in valid_opaque.ll');
  }

  // Test 3: Version diagnostic check
  // With LLVM 14 target, valid_opaque.ll should raise an LLVM-VER-002 warning for 'ptr'
  analyzer.setVersion(14);
  const res3 = analyzer.analyze(opaqueCode);
  const ptrDiag = res3.diagnostics.find(d => d.code === 'LLVM-VER-002');
  if (!ptrDiag) {
    throw new Error('Expected LLVM-VER-002 warning when parsing opaque ptr with target LLVM 14');
  }
  console.log(`✓ Version compatibility check passed: correctly detected '${ptrDiag.message}' for LLVM 14.`);

  console.log('\nAll web analyzer & visualizer tests passed successfully! ✨');
}

runTests();

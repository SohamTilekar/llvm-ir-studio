/**
 * LLVM IR Visualizer Generator - Advanced CFG Card Formatter
 * Generates colorful, grouped, and simplified CFG graphs with SSA variable backtracking.
 */

export class LlvmVisualizer {
  constructor(options = {}) {
    this.layout = options.layout || 'TD'; // 'TD' or 'LR'
    this.density = options.density || 'normal'; // 'compact' | 'normal' | 'full'
    this.viewMode = options.viewMode || 'simplified'; // 'simplified' | 'raw'
  }

  setLayout(layout) {
    this.layout = layout === 'LR' ? 'LR' : 'TD';
  }

  setDensity(density) {
    if (['compact', 'normal', 'full'].includes(density)) {
      this.density = density;
    }
  }

  setViewMode(mode) {
    if (['simplified', 'raw'].includes(mode)) {
      this.viewMode = mode;
    }
  }

  toMermaid(cfg) {
    if (!cfg || !cfg.blockOrder || cfg.blockOrder.length === 0) {
      return 'flowchart TD\n  empty["<div class=\'bb-card empty\'>No Basic Blocks Detected</div>"]';
    }

    let out = `flowchart ${this.layout}\n`;
    const edgeStyles = [];
    let edgeIndex = 0;

    // Detect loop headers
    const orderIndex = new Map();
    cfg.blockOrder.forEach((name, idx) => orderIndex.set(name, idx));

    const loopHeaders = new Set();
    for (const name of cfg.blockOrder) {
      const bb = cfg.blocks[name];
      if (!bb) continue;
      const myIdx = orderIndex.get(name) || 0;
      for (const pred of bb.predecessors) {
        const predIdx = orderIndex.get(pred);
        if (predIdx !== undefined && predIdx >= myIdx) {
          loopHeaders.add(name);
          break;
        }
      }
    }

    // 1. Generate Nodes
    for (const name of cfg.blockOrder) {
      const bb = cfg.blocks[name];
      if (!bb) continue;

      const nodeId = this.sanitizeId(name);
      const blockType = this.classifyBlock(name, bb, cfg, loopHeaders);
      const cardHtml = this.buildCardHtml(bb, name, blockType);

      const safeLabel = this.escapeForMermaid(cardHtml);
      out += `  ${nodeId}["${safeLabel}"]\n`;
    }

    // 2. Generate Edges & record colored linkStyles
    for (const name of cfg.blockOrder) {
      const bb = cfg.blocks[name];
      if (!bb) continue;

      const fromId = this.sanitizeId(name);
      const fromIdx = orderIndex.get(name) || 0;
      const term = bb.terminator;

      if (!term) {
        for (const succ of bb.successors) {
          const succId = this.sanitizeId(succ);
          out += `  ${fromId} --> ${succId}\n`;
          edgeStyles.push('stroke:#38bdf8,stroke-width:2.2px,fill:none');
          edgeIndex++;
        }
        continue;
      }

      switch (term.kind) {
        case 'CondBranch': {
          const trueId = this.sanitizeId(term.trueLabel);
          const falseId = this.sanitizeId(term.falseLabel);

          // True branch (Green)
          out += `  ${fromId} -->|true| ${trueId}\n`;
          edgeStyles.push('stroke:#22c55e,stroke-width:3px,fill:none');
          edgeIndex++;

          // False branch (Red)
          out += `  ${fromId} -->|false| ${falseId}\n`;
          edgeStyles.push('stroke:#ef4444,stroke-width:3px,fill:none');
          edgeIndex++;
          break;
        }

        case 'Branch': {
          for (const succ of bb.successors) {
            const succId = this.sanitizeId(succ);
            const succIdx = orderIndex.get(succ) || 0;
            const isBackEdge = succIdx <= fromIdx;

            if (isBackEdge) {
              out += `  ${fromId} -.->|loop| ${succId}\n`;
              edgeStyles.push('stroke:#c084fc,stroke-width:2.5px,stroke-dasharray:5 5,fill:none');
            } else {
              out += `  ${fromId} --> ${succId}\n`;
              edgeStyles.push('stroke:#38bdf8,stroke-width:2.2px,fill:none');
            }
            edgeIndex++;
          }
          break;
        }

        case 'Switch': {
          if (term.defaultLabel) {
            out += `  ${fromId} -->|default| ${this.sanitizeId(term.defaultLabel)}\n`;
            edgeStyles.push('stroke:#f59e0b,stroke-width:2.5px,fill:none');
            edgeIndex++;
          }
          if (term.cases) {
            for (const c of term.cases) {
              const caseLabel = c.val ? `case ${this.escapeForMermaid(c.val)}` : 'case';
              out += `  ${fromId} -->|${caseLabel}| ${this.sanitizeId(c.dest)}\n`;
              edgeStyles.push('stroke:#f59e0b,stroke-width:2px,fill:none');
              edgeIndex++;
            }
          }
          break;
        }

        case 'Invoke': {
          if (term.normalLabel) {
            out += `  ${fromId} -->|normal| ${this.sanitizeId(term.normalLabel)}\n`;
            edgeStyles.push('stroke:#22c55e,stroke-width:2px,fill:none');
            edgeIndex++;
          }
          if (term.unwindLabel) {
            out += `  ${fromId} -.->|unwind| ${this.sanitizeId(term.unwindLabel)}\n`;
            edgeStyles.push('stroke:#ef4444,stroke-width:2px,stroke-dasharray:4 4,fill:none');
            edgeIndex++;
          }
          break;
        }

        default: {
          for (const succ of bb.successors) {
            out += `  ${fromId} --> ${this.sanitizeId(succ)}\n`;
            edgeStyles.push('stroke:#38bdf8,stroke-width:2px,fill:none');
            edgeIndex++;
          }
          break;
        }
      }
    }

    // 3. Append linkStyle statements for vibrant arrow colors
    for (let i = 0; i < edgeStyles.length; i++) {
      out += `  linkStyle ${i} ${edgeStyles[i]};\n`;
    }

    return out;
  }

  classifyBlock(name, bb, cfg, loopHeaders) {
    if (name === cfg.entryBlock) return 'entry';
    if (bb.terminator && bb.terminator.kind === 'Return') return 'return';
    if (bb.terminator && bb.terminator.kind === 'Unreachable') return 'unreach';
    if (loopHeaders.has(name) || name.toLowerCase().includes('loop')) return 'loop';
    if (bb.terminator && (bb.terminator.kind === 'CondBranch' || bb.terminator.kind === 'Switch')) return 'decision';
    return 'normal';
  }

  buildCardHtml(bb, name, blockType) {
    const config = {
      entry: {
        border: '#00e5ff',
        bg: '#0c2238',
        headerBg: 'rgba(0, 229, 255, 0.22)',
        titleColor: '#38bdf8',
        badgeColor: '#00e5ff',
        tag: '⚡ ENTRY',
        shadow: '0 0 16px rgba(0, 229, 255, 0.25)',
      },
      loop: {
        border: '#a855f7',
        bg: '#1c1033',
        headerBg: 'rgba(168, 85, 247, 0.25)',
        titleColor: '#c084fc',
        badgeColor: '#d8b4fe',
        tag: '🔁 LOOP',
        shadow: '0 0 16px rgba(168, 85, 247, 0.25)',
      },
      decision: {
        border: '#f59e0b',
        bg: '#261606',
        headerBg: 'rgba(245, 158, 11, 0.22)',
        titleColor: '#fbbf24',
        badgeColor: '#fde68a',
        tag: '🔀 DECISION',
        shadow: '0 0 16px rgba(245, 158, 11, 0.25)',
      },
      return: {
        border: '#10b981',
        bg: '#062015',
        headerBg: 'rgba(16, 185, 129, 0.22)',
        titleColor: '#34d399',
        badgeColor: '#a7f3d0',
        tag: '⮑ EXIT',
        shadow: '0 0 16px rgba(16, 185, 129, 0.25)',
      },
      unreach: {
        border: '#ef4444',
        bg: '#280909',
        headerBg: 'rgba(239, 68, 68, 0.25)',
        titleColor: '#f87171',
        badgeColor: '#fca5a5',
        tag: '⛔ TRAP',
        shadow: '0 0 16px rgba(239, 68, 68, 0.25)',
      },
      normal: {
        border: '#3b82f6',
        bg: '#0f172a',
        headerBg: 'rgba(59, 130, 246, 0.15)',
        titleColor: '#93c5fd',
        badgeColor: '#bfdbfe',
        tag: '📦 BLOCK',
        shadow: '0 4px 16px rgba(0, 0, 0, 0.4)',
      },
    }[blockType];

    const totalOps = bb.instructions.length;
    let cardStyle = `min-width:240px;max-width:420px;background:${config.bg};border:2px solid ${config.border};border-radius:8px;box-shadow:${config.shadow};font-family:ui-monospace,SFMono-Regular,Menlo,monospace;text-align:left;overflow:hidden;`;

    let html = `<div class='bb-card' style='${cardStyle}'>`;

    // 1. Header Bar
    html += `<div style='display:flex;justify-content:space-between;align-items:center;padding:7px 12px;background:${config.headerBg};border-bottom:1px solid rgba(255,255,255,0.12);'>`;
    html += `<span style='font-weight:700;font-size:12px;color:${config.titleColor};'>${config.tag} %${this.escapeHtml(name)}</span>`;
    html += `<span style='font-size:10px;padding:2px 7px;border-radius:10px;background:rgba(0,0,0,0.3);color:${config.badgeColor};border:1px solid rgba(255,255,255,0.15);font-weight:600;'>${totalOps} ops</span>`;
    html += `</div>`;

    // 2. Body Content (Grouped or Compact)
    if (this.density === 'compact') {
      html += `<div style='padding:8px 12px;color:#94a3b8;font-size:11px;font-style:italic;'>${totalOps} instructions</div>`;
    } else {
      html += `<div style='padding:6px 10px;display:flex;flex-direction:column;gap:5px;background:rgba(0,0,0,0.25);'>`;

      let instList = bb.instructions.map((i) => i.clean || i.raw || '');
      if (instList.length > 0 && bb.terminator) {
        instList = instList.slice(0, instList.length - 1);
      }

      if (instList.length === 0 && !bb.terminator) {
        html += `<div style='color:#64748b;font-size:11px;font-style:italic;'>(empty block)</div>`;
      } else {
        const groups = this.groupInstructions(instList);

        for (const group of groups) {
          html += `<div style='margin-top:3px;'>`;
          html += `<div style='font-size:9.5px;font-weight:700;color:${group.color};letter-spacing:0.04em;margin-bottom:2px;text-transform:uppercase;'>${group.icon} ${group.title}</div>`;

          for (const item of group.items) {
            const formatted = this.viewMode === 'simplified' ? this.simplifyInstruction(item) : item;
            html += `<div style='font-size:11px;line-height:1.4;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;padding:1px 0;'>${this.highlightTokens(formatted)}</div>`;
          }
          html += `</div>`;
        }
      }

      html += `</div>`;
    }

    // 3. Terminator Footer
    if (bb.terminator) {
      const termInfo = this.formatTerminator(bb.terminator);
      html += `<div style='padding:7px 12px;background:rgba(0,0,0,0.5);border-top:1px solid rgba(255,255,255,0.1);display:flex;align-items:center;gap:6px;font-size:11.5px;font-weight:600;color:${termInfo.color};'>`;
      html += `<span>${termInfo.icon}</span>`;
      html += `<span>${termInfo.text}</span>`;
      html += `</div>`;
    }

    html += `</div>`;
    return html;
  }

  groupInstructions(instructions) {
    const groups = {
      phi: { title: 'SSA Merge', icon: '𝚽', color: '#c084fc', items: [] },
      compute: { title: 'Computation', icon: '⚡', color: '#38bdf8', items: [] },
      mem: { title: 'Memory', icon: '💾', color: '#4ade80', items: [] },
      call: { title: 'Call', icon: '📞', color: '#f472b6', items: [] },
      other: { title: 'Other', icon: '⚙', color: '#94a3b8', items: [] },
    };

    for (const inst of instructions) {
      const trimmed = inst.trim();
      if (trimmed.includes(' = phi ')) {
        groups.phi.items.push(trimmed);
      } else if (/\b(load|store|alloca|getelementptr|fence|atomic)\b/.test(trimmed)) {
        groups.mem.items.push(trimmed);
      } else if (/\b(call|invoke)\b/.test(trimmed)) {
        groups.call.items.push(trimmed);
      } else if (/\b(add|sub|mul|sdiv|udiv|fadd|fsub|fmul|fdiv|srem|urem|frem|and|or|xor|shl|lshr|ashr|icmp|fcmp|select)\b/.test(trimmed)) {
        groups.compute.items.push(trimmed);
      } else {
        groups.other.items.push(trimmed);
      }
    }

    const result = [];
    if (groups.phi.items.length > 0) result.push(groups.phi);
    if (groups.compute.items.length > 0) result.push(groups.compute);
    if (groups.mem.items.length > 0) result.push(groups.mem);
    if (groups.call.items.length > 0) result.push(groups.call);
    if (groups.other.items.length > 0) result.push(groups.other);

    if (this.density === 'normal') {
      let count = 0;
      for (const g of result) {
        if (count + g.items.length > 7) {
          const keep = Math.max(1, 7 - count);
          const omitted = g.items.length - keep;
          g.items = g.items.slice(0, keep);
          if (omitted > 0) {
            g.items.push(`··· ${omitted} more ops ···`);
          }
          break;
        }
        count += g.items.length;
      }
    }

    return result;
  }

  simplifyInstruction(inst) {
    let s = inst.replace(/;.*$/, '').trim();

    // 1. Division: %half = sdiv i32 %curr, 2 -> half = i32 curr / 2 (signed divide)
    const divMatch = s.match(
      /^(%[a-zA-Z$._0-9]+)\s*=\s*(sdiv|udiv|fdiv)\s+(?:(?:exact)\s+)?([a-zA-Z0-9]+)\s+([^,]+),\s*(.+)$/
    );
    if (divMatch) {
      const [_, res, op, ty, op1, op2] = divMatch;
      const desc = op === 'sdiv' ? 'signed divide' : op === 'udiv' ? 'unsigned divide' : 'float divide';
      const cleanRes = res.replace(/^%/, '');
      const cleanOp1 = op1.trim().replace(/^%/, '');
      const cleanOp2 = op2.trim().replace(/^%/, '');
      return `${cleanRes} = ${ty} ${cleanOp1} / ${cleanOp2} (${desc})`;
    }

    // 2. Remainder: %rem = srem i32 %curr, 2 -> rem = i32 curr % 2 (signed remainder)
    const remMatch = s.match(
      /^(%[a-zA-Z$._0-9]+)\s*=\s*(srem|urem|frem)\s+([a-zA-Z0-9]+)\s+([^,]+),\s*(.+)$/
    );
    if (remMatch) {
      const [_, res, op, ty, op1, op2] = remMatch;
      const desc = op === 'srem' ? 'signed remainder' : op === 'urem' ? 'unsigned remainder' : 'float remainder';
      const cleanRes = res.replace(/^%/, '');
      const cleanOp1 = op1.trim().replace(/^%/, '');
      const cleanOp2 = op2.trim().replace(/^%/, '');
      return `${cleanRes} = ${ty} ${cleanOp1} % ${cleanOp2} (${desc})`;
    }

    // 3. Comparisons: %is_even = icmp eq i32 %rem, 0 -> is_even = i32 rem == 0 (is equal)
    const icmpMatch = s.match(
      /^(%[a-zA-Z$._0-9]+)\s*=\s*icmp\s+([a-z]+)\s+([a-zA-Z0-9]+)\s+([^,]+),\s*(.+)$/
    );
    if (icmpMatch) {
      const [_, res, pred, ty, op1, op2] = icmpMatch;
      const predMap = {
        eq: { sym: '==', desc: 'is equal' },
        ne: { sym: '!=', desc: 'not equal' },
        slt: { sym: '<', desc: 'signed less than' },
        sle: { sym: '<=', desc: 'signed less or equal' },
        sgt: { sym: '>', desc: 'signed greater than' },
        sge: { sym: '>=', desc: 'signed greater or equal' },
        ult: { sym: '<', desc: 'unsigned less than' },
        ule: { sym: '<=', desc: 'unsigned less or equal' },
        ugt: { sym: '>', desc: 'unsigned greater than' },
        uge: { sym: '>=', desc: 'unsigned greater or equal' },
      };
      const p = predMap[pred] || { sym: pred, desc: pred };
      const cleanRes = res.replace(/^%/, '');
      const cleanOp1 = op1.trim().replace(/^%/, '');
      const cleanOp2 = op2.trim().replace(/^%/, '');
      return `${cleanRes} = ${ty} ${cleanOp1} ${p.sym} ${cleanOp2} (${p.desc})`;
    }

    // 4. Arithmetic: %plus1 = add i32 %mul, 1 -> plus1 = i32 mul + 1 (add)
    const arithMatch = s.match(
      /^(%[a-zA-Z$._0-9]+)\s*=\s*(add|fadd|sub|fsub|mul|fmul)\s+(?:(?:nuw|nsw)\s+)*([a-zA-Z0-9]+)\s+([^,]+),\s*(.+)$/
    );
    if (arithMatch) {
      const [_, res, op, ty, op1, op2] = arithMatch;
      const opInfo = {
        add: { sym: '+', desc: 'add' },
        fadd: { sym: '+', desc: 'float add' },
        sub: { sym: '-', desc: 'subtract' },
        fsub: { sym: '-', desc: 'float subtract' },
        mul: { sym: '*', desc: 'multiply' },
        fmul: { sym: '*', desc: 'float multiply' },
      }[op] || { sym: op, desc: op };
      const cleanRes = res.replace(/^%/, '');
      const cleanOp1 = op1.trim().replace(/^%/, '');
      const cleanOp2 = op2.trim().replace(/^%/, '');
      return `${cleanRes} = ${ty} ${cleanOp1} ${opInfo.sym} ${cleanOp2} (${opInfo.desc})`;
    }

    // 5. Phi: %curr = phi i32 [ %n, %entry ], [ %next, %loop_next ] -> curr = i32 phi(n from %entry, next from %loop_next) (SSA merge)
    const phiMatch = s.match(/^(%[a-zA-Z$._0-9]+)\s*=\s*phi\s+([a-zA-Z0-9]+)\s+(.+)$/);
    if (phiMatch) {
      const [_, res, ty, pairsStr] = phiMatch;
      const cleanRes = res.replace(/^%/, '');
      const pairs = [];
      const pairMatches = pairsStr.matchAll(/\[\s*([^,]+),\s*%([a-zA-Z$._0-9]+)\s*\]/g);
      for (const m of pairMatches) {
        const val = m[1].trim().replace(/^%/, '');
        pairs.push(`${val} from %${m[2]}`);
      }
      return `${cleanRes} = ${ty} phi(${pairs.join(', ')}) (SSA merge)`;
    }

    // 6. Memory Load: %val = load i32, ptr %ptr -> val = i32 *ptr (load from memory)
    const loadMatch = s.match(/^(%[a-zA-Z$._0-9]+)\s*=\s*load\s+(?:(?:volatile)\s+)?([a-zA-Z0-9]+),\s*ptr\s+([^,]+)/);
    if (loadMatch) {
      const cleanRes = loadMatch[1].replace(/^%/, '');
      const ty = loadMatch[2];
      const cleanPtr = loadMatch[3].trim().replace(/^%/, '');
      return `${cleanRes} = ${ty} *${cleanPtr} (load from memory)`;
    }

    // 7. Memory Store: store i32 %val, ptr %dest -> *dest = i32 val (store to memory)
    const storeMatch = s.match(/^store\s+(?:(?:volatile)\s+)?([a-zA-Z0-9]+)\s+([^,]+),\s*ptr\s+([^,]+)/);
    if (storeMatch) {
      const ty = storeMatch[1];
      const cleanVal = storeMatch[2].trim().replace(/^%/, '');
      const cleanDest = storeMatch[3].trim().replace(/^%/, '');
      return `*${cleanDest} = ${ty} ${cleanVal} (store to memory)`;
    }

    // 8. Call: %call1 = call i32 @fib(i32 %sub1) -> call1 = i32 @fib(sub1) (call function)
    const callMatch = s.match(/^(?:(%[a-zA-Z$._0-9]+)\s*=\s*)?call\s+([a-zA-Z0-9]+)\s+(@[a-zA-Z$._0-9]+)\(([^)]*)\)/);
    if (callMatch) {
      const cleanRes = callMatch[1] ? `${callMatch[1].replace(/^%/, '')} = ` : '';
      const ty = callMatch[2];
      const fn = callMatch[3];
      const args = callMatch[4]
        .split(',')
        .map((a) => a.trim().split(/\s+/).pop().replace(/^%/, ''))
        .filter((a) => a.length > 0)
        .join(', ');
      return `${cleanRes}${ty} ${fn}(${args}) (call function)`;
    }

    // 9. GEP: %elem = getelementptr i32, ptr %buf, i32 %i -> elem = gep(buf, offset: i) (pointer offset)
    const gepMatch = s.match(
      /^(%[a-zA-Z$._0-9]+)\s*=\s*getelementptr\s+(?:inbounds\s+)?([a-zA-Z0-9]+),\s*ptr\s+([^,]+),\s*(.+)$/
    );
    if (gepMatch) {
      const cleanRes = gepMatch[1].replace(/^%/, '');
      const cleanBase = gepMatch[3].trim().replace(/^%/, '');
      const offsets = gepMatch[4].split(',').map((o) => o.trim().split(/\s+/).pop().replace(/^%/, '')).join(', ');
      return `${cleanRes} = gep(${cleanBase}, offset: ${offsets}) (pointer offset)`;
    }

    // 10. Bitwise: %and = and i32 %a, %b -> and = i32 a & b (bitwise and)
    const bitMatch = s.match(/^(%[a-zA-Z$._0-9]+)\s*=\s*(and|or|xor|shl|ashr|lshr)\s+([a-zA-Z0-9]+)\s+([^,]+),\s*(.+)$/);
    if (bitMatch) {
      const [_, res, op, ty, op1, op2] = bitMatch;
      const bitInfo = {
        and: { sym: '&', desc: 'bitwise and' },
        or: { sym: '|', desc: 'bitwise or' },
        xor: { sym: '^', desc: 'bitwise xor' },
        shl: { sym: '<<', desc: 'shift left' },
        ashr: { sym: '>>', desc: 'arithmetic shift right' },
        lshr: { sym: '>>', desc: 'logical shift right' },
      }[op] || { sym: op, desc: op };
      const cleanRes = res.replace(/^%/, '');
      const cleanOp1 = op1.trim().replace(/^%/, '');
      const cleanOp2 = op2.trim().replace(/^%/, '');
      return `${cleanRes} = ${ty} ${cleanOp1} ${bitInfo.sym} ${cleanOp2} (${bitInfo.desc})`;
    }

    return s;
  }

  highlightTokens(str) {
    if (!str) return '';
    const escaped = this.escapeHtml(str);

    // Single-pass tokenizer to inject interactive variable pills and syntax spans
    const tokenRegex =
      /(%[a-zA-Z$._0-9]+)|(@[a-zA-Z$._0-9]+)|(\([a-zA-Z\s]+\))|(\b(?:i\d+|float|double|half|ptr|void)\b)|(\b(?:phi|gep|alloca|return|call)\b)|(\b-?\d+\b)|([\+\-\*\/\%&\|\^<>=!]+)/g;

    return escaped.replace(tokenRegex, (match, reg, glob, desc, ty, kw, num, op) => {
      if (reg) {
        const clean = reg.replace(/^%/, '');
        return `<span class='var-pill' data-var='${reg}' title='Click to backtrack ${reg} (SSA Def-Use)'>${clean}</span>`;
      }
      if (glob) return `<span style='color:#fb923c;font-weight:600;'>${glob}</span>`;
      if (desc) return `<span class='op-desc'>${desc}</span>`;
      if (ty) return `<span class='type-pill'>${ty}</span>`;
      if (kw) return `<span style='color:#a78bfa;font-weight:700;'>${kw}</span>`;
      if (num) return `<span style='color:#f472b6;'>${num}</span>`;
      if (op) return `<span style='color:#f43f5e;font-weight:700;'>${op}</span>`;
      return match;
    });
  }

  formatTerminator(term) {
    switch (term.kind) {
      case 'CondBranch': {
        const condEsc = this.escapeHtml(term.cond || '%cond');
        const cleanCond = condEsc.replace(/^%/, '');
        return {
          icon: '🔀',
          text: `if (<span class='var-pill' data-var='${condEsc}' title='Click to backtrack condition'>${cleanCond}</span>) → %${term.trueLabel} else → %${term.falseLabel}`,
          color: '#fbbf24',
        };
      }
      case 'Branch': {
        const dest = term.successors[0] || '';
        return {
          icon: '↳',
          text: `jump → <span style='color:#facc15;font-weight:700;'>%${this.escapeHtml(dest)}</span>`,
          color: '#facc15',
        };
      }
      case 'Switch': {
        return {
          icon: '⚡',
          text: `switch (${term.cases ? term.cases.length : 0} cases)`,
          color: '#f59e0b',
        };
      }
      case 'Return': {
        const val = term.label.replace(/^ret\s+/, '').replace(/^[a-z0-9]+\s+/, '');
        const cleanVal = val.startsWith('%')
          ? `<span class='var-pill' data-var='${val}' title='Click to backtrack return value'>${val.replace(/^%/, '')}</span>`
          : val
          ? `<span style='color:#34d399;'>${this.escapeHtml(val)}</span>`
          : '';
        return {
          icon: '⮑',
          text: `return ${cleanVal}`,
          color: '#34d399',
        };
      }
      case 'Unreachable': {
        return {
          icon: '⛔',
          text: 'trap (unreachable)',
          color: '#ef4444',
        };
      }
      case 'Invoke': {
        return {
          icon: '↗',
          text: 'invoke (with unwind)',
          color: '#818cf8',
        };
      }
      default: {
        return {
          icon: '↳',
          text: this.escapeHtml(term.label || 'terminator'),
          color: '#94a3b8',
        };
      }
    }
  }

  toDot(cfg) {
    if (!cfg || !cfg.blockOrder) return 'digraph CFG {}';

    let dot = `digraph "CFG for '${cfg.functionName}'" {\n`;
    dot += '  node [shape=record, fontname="JetBrains Mono, Menlo, monospace", fontsize=10, style=filled, fillcolor="#0f172a", fontcolor="#f8fafc", color="#334155"];\n';
    dot += '  edge [fontname="JetBrains Mono, Menlo, monospace", fontsize=9, color="#64748b", fontcolor="#94a3b8"];\n';
    dot += '  bgcolor="#0b0f19";\n\n';

    for (const name of cfg.blockOrder) {
      const bb = cfg.blocks[name];
      const nodeId = `"${name}"`;
      const instLines = bb.instructions.map((i) => this.escapeDot(this.simplifyInstruction(i.clean || i.raw))).join('\\l');
      const label = `{ ${this.escapeDot(name)}: | ${instLines}\\l }`;
      dot += `  ${nodeId} [label="${label}"];\n`;
    }

    dot += '\n';

    for (const name of cfg.blockOrder) {
      const bb = cfg.blocks[name];
      const fromId = `"${name}"`;
      const term = bb.terminator;

      if (term && term.kind === 'CondBranch') {
        dot += `  ${fromId} -> "${term.trueLabel}" [label="true", color="#22c55e", fontcolor="#22c55e", penwidth=2.5];\n`;
        dot += `  ${fromId} -> "${term.falseLabel}" [label="false", color="#ef4444", fontcolor="#ef4444", penwidth=2.5];\n`;
      } else if (term && term.kind === 'Switch') {
        if (term.defaultLabel) {
          dot += `  ${fromId} -> "${term.defaultLabel}" [label="default", color="#f59e0b"];\n`;
        }
        if (term.cases) {
          for (const c of term.cases) {
            dot += `  ${fromId} -> "${c.dest}" [label="case ${this.escapeDot(c.val)}", color="#f59e0b"];\n`;
          }
        }
      } else {
        for (const succ of bb.successors) {
          dot += `  ${fromId} -> "${succ}" [color="#38bdf8", penwidth=1.8];\n`;
        }
      }
    }

    dot += '}\n';
    return dot;
  }

  sanitizeId(name) {
    const clean = name.replace(/[^a-zA-Z0-9_]/g, '_');
    if (!clean || /^[0-9]/.test(clean)) {
      return `bb_${clean}`;
    }
    return clean;
  }

  escapeHtml(str) {
    return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  }

  escapeForMermaid(str) {
    return str.replace(/"/g, "'").replace(/\[/g, '&#91;').replace(/\]/g, '&#93;');
  }

  escapeDot(str) {
    return str
      .replace(/\\/g, '\\\\')
      .replace(/"/g, '\\"')
      .replace(/\{/g, '\\{')
      .replace(/\}/g, '\\}')
      .replace(/\|/g, '\\|')
      .replace(/</g, '\\<')
      .replace(/>/g, '\\>');
  }
}

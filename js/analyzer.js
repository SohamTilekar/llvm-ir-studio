/**
 * LLVM IR In-Browser Analyzer & CFG Builder
 * Replicates and extends ir-analysis logic with full SSA Def-Use Graph extraction.
 */

export class LlvmAnalyzer {
  constructor(options = {}) {
    this.targetVersion = options.targetVersion || 18;
  }

  setVersion(version) {
    this.targetVersion = Number(version) || 18;
  }

  analyze(source) {
    const lines = source.split(/\r?\n/);
    const diagnostics = [];
    const functions = [];
    const cfgs = {};

    // 1. Locate all function definitions: define [linkage] [visibility] [cconv] [ret_type] @func_name(args) [attrs] {
    const fnDefRegex = /^\s*define\b[^{]*@([a-zA-Z$._0-9]+|"[^"]+")\s*\(([^)]*)\)[^{]*\{/;

    let inFunction = false;
    let currentFn = null;
    let fnStartLine = 0;
    let fnLines = [];
    let fnArgsRaw = '';

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      const trimmed = line.trim();

      if (!inFunction) {
        const match = line.match(fnDefRegex);
        if (match) {
          inFunction = true;
          const fnName = match[1].replace(/^"|"$/g, '');
          fnArgsRaw = match[2] || '';
          currentFn = {
            name: fnName,
            rawSignature: line.trim().replace(/\{$/, '').trim(),
            startLine: i,
            endLine: i,
            argsRaw: fnArgsRaw,
          };
          fnStartLine = i;
          fnLines = [{ lineNum: i, text: line }];
        }
      } else {
        fnLines.push({ lineNum: i, text: line });
        // Check for function closing brace
        if (trimmed === '}' || (trimmed.startsWith('}') && !trimmed.includes('{'))) {
          inFunction = false;
          currentFn.endLine = i;
          functions.push(currentFn);

          // Build CFG for this function with SSA Def-Use Map
          const cfg = this.buildFunctionCfg(currentFn.name, fnLines, diagnostics, currentFn.argsRaw);
          cfgs[currentFn.name] = cfg;

          currentFn = null;
          fnLines = [];
        }
      }

      // Check version-specific rules across the file
      this.checkLineVersionRules(trimmed, i, diagnostics);
    }

    if (inFunction && currentFn) {
      diagnostics.push({
        code: 'LLVM-SYNTAX-003',
        message: `Unclosed function definition '@${currentFn.name}' (missing closing '}')`,
        line: currentFn.startLine + 1,
        column: 1,
        severity: 'Error',
      });
      currentFn.endLine = lines.length - 1;
      functions.push(currentFn);
      cfgs[currentFn.name] = this.buildFunctionCfg(currentFn.name, fnLines, diagnostics, currentFn.argsRaw);
    }

    return {
      functions,
      cfgs,
      diagnostics,
      version: this.targetVersion,
    };
  }

  buildFunctionCfg(fnName, linesWithNums, diagnostics, fnArgsRaw = '') {
    const cfg = {
      functionName: fnName,
      entryBlock: 'entry',
      blocks: {},
      blockOrder: [],
      ssaMap: {}, // Complete SSA Def-Use and Use-Def table
    };

    // 1. Initialize SSA Map with function arguments
    if (fnArgsRaw) {
      const argMatches = fnArgsRaw.matchAll(/([a-zA-Z0-9$_\*\[\]]+)\s+(%[a-zA-Z$._0-9]+)/g);
      for (const m of argMatches) {
        const ty = m[1].trim();
        const reg = m[2].trim();
        cfg.ssaMap[reg] = {
          name: reg,
          cleanName: reg.replace(/^%/, ''),
          type: ty,
          line: linesWithNums[0].lineNum + 1,
          block: 'entry',
          isArg: true,
          raw: `parameter ${reg}: ${ty}`,
          inputs: [],
          uses: [],
        };
      }
    }

    let currentBlockName = 'entry';
    let blockDefined = false;

    // 2. Scan lines inside the function body
    for (let idx = 1; idx < linesWithNums.length - 1; idx++) {
      const { lineNum, text } = linesWithNums[idx];
      const trimmed = text.trim();

      if (!trimmed || trimmed.startsWith(';')) {
        continue;
      }

      // Label pattern e.g. "entry:", "loop.body:", "7:"
      const labelMatch = trimmed.match(/^([a-zA-Z$._0-9]+):\s*(;.*)?$/);
      if (labelMatch) {
        const labelName = labelMatch[1];
        currentBlockName = labelName;
        blockDefined = true;

        if (!cfg.blocks[labelName]) {
          cfg.blocks[labelName] = {
            name: labelName,
            line: lineNum,
            instructions: [],
            terminator: null,
            successors: [],
            predecessors: [],
          };
          cfg.blockOrder.push(labelName);
        }
        continue;
      }

      // If instruction encountered before any explicit label, use entry
      if (!cfg.blocks[currentBlockName]) {
        cfg.blocks[currentBlockName] = {
          name: currentBlockName,
          line: lineNum,
          instructions: [],
          terminator: null,
          successors: [],
          predecessors: [],
        };
        cfg.blockOrder.push(currentBlockName);
      }

      const bb = cfg.blocks[currentBlockName];
      const instWithoutComment = trimmed.replace(/\s*;.*$/, '').trim();

      if (instWithoutComment) {
        bb.instructions.push({
          raw: trimmed,
          clean: instWithoutComment,
          line: lineNum,
        });

        // 3. Extract SSA Definition & Uses
        const assignMatch = instWithoutComment.match(/^(%[a-zA-Z$._0-9]+)\s*=\s*(.+)$/);
        let defVar = null;
        let rhs = instWithoutComment;

        if (assignMatch) {
          defVar = assignMatch[1];
          rhs = assignMatch[2];

          // Determine type from instruction
          const typeMatch = rhs.match(/\b(i\d+|float|double|half|ptr|void|\[[^\]]+\])\b/);
          const varType = typeMatch ? typeMatch[1] : '';

          // Find upstream input registers used by this instruction (ignoring block labels in phi)
          let inputs = (rhs.match(/%[a-zA-Z$._0-9]+/g) || []).filter((v) => v !== defVar);
          if (rhs.startsWith('phi ')) {
            // For phi nodes, incoming pair is [ %val, %label ]
            const phiVals = [];
            for (const m of rhs.matchAll(/\[\s*([^,]+),\s*%[a-zA-Z$._0-9]+\s*\]/g)) {
              const val = m[1].trim();
              if (val.startsWith('%') && val !== defVar) {
                phiVals.push(val);
              }
            }
            inputs = phiVals;
          }

          cfg.ssaMap[defVar] = {
            name: defVar,
            cleanName: defVar.replace(/^%/, ''),
            type: varType,
            line: lineNum + 1,
            block: currentBlockName,
            isArg: false,
            raw: instWithoutComment,
            inputs: Array.from(new Set(inputs)),
            uses: cfg.ssaMap[defVar]?.uses || [],
          };
        }

        // Record usage of each input register
        const usedVars = (rhs.match(/%[a-zA-Z$._0-9]+/g) || []).filter((v) => v !== defVar);
        for (const uv of new Set(usedVars)) {
          if (!cfg.ssaMap[uv]) {
            cfg.ssaMap[uv] = {
              name: uv,
              cleanName: uv.replace(/^%/, ''),
              type: '',
              line: lineNum + 1,
              block: currentBlockName,
              isArg: false,
              raw: 'external / argument',
              inputs: [],
              uses: [],
            };
          }
          cfg.ssaMap[uv].uses.push({
            line: lineNum + 1,
            block: currentBlockName,
            inst: instWithoutComment,
            targetVar: defVar,
          });
        }

        // Check if instruction is a terminator
        const term = this.parseTerminator(instWithoutComment, lineNum);
        if (term) {
          bb.terminator = term;
          bb.successors = term.successors;

          // If terminator uses variables (e.g. cond in br i1 %cond or ret %val)
          const termVars = instWithoutComment.match(/%[a-zA-Z$._0-9]+/g) || [];
          for (const tv of new Set(termVars)) {
            if (cfg.ssaMap[tv]) {
              cfg.ssaMap[tv].uses.push({
                line: lineNum + 1,
                block: currentBlockName,
                inst: instWithoutComment,
                isTerminator: true,
              });
            }
          }
        }
      }
    }

    if (cfg.blockOrder.length > 0) {
      cfg.entryBlock = cfg.blockOrder[0];
    }

    // Validate blocks and link predecessors
    const knownBlockNames = new Set(cfg.blockOrder);

    for (const bName of cfg.blockOrder) {
      const bb = cfg.blocks[bName];

      if (!bb.terminator) {
        diagnostics.push({
          code: 'LLVM-CFG-002',
          message: `Basic block '${bName}' does not end with a valid terminator instruction (br, ret, switch, unreachable, invoke)`,
          line: bb.line + 1,
          column: 1,
          severity: 'Warning',
        });
      }

      for (const succ of bb.successors) {
        if (!knownBlockNames.has(succ)) {
          diagnostics.push({
            code: 'LLVM-CFG-001',
            message: `Basic block '${bName}' branches to non-existent label '%${succ}'`,
            line: bb.line + 1,
            column: 1,
            severity: 'Error',
          });
        } else {
          cfg.blocks[succ].predecessors.push(bName);
        }
      }
    }

    return cfg;
  }

  parseTerminator(inst, lineNum) {
    if (inst === 'ret' || inst.startsWith('ret ') || inst.startsWith('ret\t')) {
      return { kind: 'Return', label: inst, successors: [] };
    }

    if (inst === 'unreachable') {
      return { kind: 'Unreachable', label: 'unreachable', successors: [] };
    }

    const unidirBr = inst.match(/^br\s+label\s+%([a-zA-Z$._0-9]+)/);
    if (unidirBr) {
      return {
        kind: 'Branch',
        label: `br label %${unidirBr[1]}`,
        successors: [unidirBr[1]],
      };
    }

    const condBr = inst.match(
      /^br\s+i1\s+([^,]+),\s*label\s+%([a-zA-Z$._0-9]+),\s*label\s+%([a-zA-Z$._0-9]+)/
    );
    if (condBr) {
      return {
        kind: 'CondBranch',
        cond: condBr[1].trim(),
        trueLabel: condBr[2],
        falseLabel: condBr[3],
        label: `br i1 ${condBr[1].trim()}`,
        successors: [condBr[2], condBr[3]],
      };
    }

    if (inst.startsWith('switch ')) {
      const defaultMatch = inst.match(/label\s+%([a-zA-Z$._0-9]+)/);
      const defaultLabel = defaultMatch ? defaultMatch[1] : '';
      const successors = defaultLabel ? [defaultLabel] : [];
      const cases = [];

      const caseMatches = inst.matchAll(/([a-zA-Z0-9]+)\s+([^,]+),\s*label\s+%([a-zA-Z$._0-9]+)/g);
      for (const m of caseMatches) {
        cases.push({ val: m[2].trim(), dest: m[3] });
        if (!successors.includes(m[3])) {
          successors.push(m[3]);
        }
      }

      return {
        kind: 'Switch',
        defaultLabel,
        cases,
        label: 'switch',
        successors,
      };
    }

    const invokeMatch = inst.match(
      /to\s+label\s+%([a-zA-Z$._0-9]+)\s+unwind\s+label\s+%([a-zA-Z$._0-9]+)/
    );
    if (invokeMatch) {
      return {
        kind: 'Invoke',
        normalLabel: invokeMatch[1],
        unwindLabel: invokeMatch[2],
        label: 'invoke',
        successors: [invokeMatch[1], invokeMatch[2]],
      };
    }

    if (inst.startsWith('resume ')) {
      return { kind: 'Resume', label: inst, successors: [] };
    }

    return null;
  }

  checkLineVersionRules(line, lineNum, diagnostics) {
    if (!line || line.startsWith(';')) return;

    if (this.targetVersion >= 17) {
      const typedPtrMatch = line.match(/\b(i\d+|float|double|\[[^\]]+\]|\%[a-zA-Z0-9$._]+)\*\s/);
      if (typedPtrMatch && !line.includes('ptr')) {
        diagnostics.push({
          code: 'LLVM-VER-001',
          message: `Typed pointer '${typedPtrMatch[0].trim()}' is obsolete in LLVM ${this.targetVersion}; use opaque 'ptr'`,
          line: lineNum + 1,
          column: line.indexOf(typedPtrMatch[0]) + 1,
          severity: 'Warning',
        });
      }
    } else if (this.targetVersion <= 14) {
      if (/\bptr\b/.test(line) && !line.includes('ptrtoint') && !line.includes('inttoptr')) {
        diagnostics.push({
          code: 'LLVM-VER-002',
          message: `Opaque pointer 'ptr' is unsupported in LLVM ${this.targetVersion} (requires LLVM 15+)`,
          line: lineNum + 1,
          column: line.indexOf('ptr') + 1,
          severity: 'Warning',
        });
      }
    }

    if (this.targetVersion >= 16) {
      if (/\b(readnone|readonly|writeonly|argmemonly)\b/.test(line)) {
        diagnostics.push({
          code: 'LLVM-VER-003',
          message: `Memory attribute is deprecated in LLVM ${this.targetVersion}; use 'memory(...)' instead`,
          line: lineNum + 1,
          column: 1,
          severity: 'Information',
        });
      }
    }
  }
}

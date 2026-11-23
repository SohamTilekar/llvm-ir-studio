/**
 * LLVM IR Studio - Web Application Controller
 * Enhanced Pan-Zoom, SSA Def-Use Backtracking & Editor-Graph Correlation
 */

import { LlvmAnalyzer } from './analyzer.js';
import { LlvmVisualizer } from './visualizer.js';
import { PRESETS } from './presets.js';
import { registerLlvmLanguage } from './monaco-llvm.js';

class LlvmStudioApp {
  constructor() {
    this.analyzer = new LlvmAnalyzer({ targetVersion: 18 });
    this.visualizer = new LlvmVisualizer({ layout: 'TD', density: 'normal', viewMode: 'simplified' });

    this.editor = null;
    this.currentFunction = null;
    this.analysisResult = null;
    this.activeTab = 'graph'; // 'graph' | 'mermaid' | 'dot' | 'diags'

    // Pan & Zoom state
    this.zoomScale = 1.0;
    this.panX = 40;
    this.panY = 60;
    this.isPanning = false;
    this.dragStartX = 0;
    this.dragStartY = 0;
    this.hasDragged = false;

    // SSA Variable Inspection & Correlation State
    this.activeVariable = null;
    this.monacoDecorations = [];

    this.debounceTimer = null;
  }

  async init() {
    this.setupPresetsDropdown();
    this.setupEventListeners();
    await this.initMonaco();
    this.initPanZoom();
    this.loadInitialContent();
  }

  setupPresetsDropdown() {
    const select = document.getElementById('preset-select');
    select.innerHTML = '<option value="" disabled selected>Load Preset Snippet...</option>';

    for (const [key, preset] of Object.entries(PRESETS)) {
      const opt = document.createElement('option');
      opt.value = key;
      opt.textContent = preset.name;
      select.appendChild(opt);
    }

    select.addEventListener('change', (e) => {
      const key = e.target.value;
      if (PRESETS[key]) {
        this.editor.setValue(PRESETS[key].code.trim());
        this.showToast(`Loaded preset: ${PRESETS[key].name}`);
      }
    });
  }

  async initMonaco() {
    return new Promise((resolve) => {
      window.require.config({
        paths: { vs: 'https://cdn.jsdelivr.net/npm/monaco-editor@0.52.0/min/vs' }
      });

      window.require(['vs/editor/editor.main'], () => {
        registerLlvmLanguage(window.monaco);

        const container = document.getElementById('editor-container');
        this.editor = window.monaco.editor.create(container, {
          value: '',
          language: 'llvm',
          theme: 'llvm-studio-dark',
          automaticLayout: true,
          fontFamily: "'JetBrains Mono', 'Fira Code', ui-monospace, Menlo, Consolas, monospace",
          fontSize: 13,
          lineHeight: 20,
          minimap: { enabled: false },
          scrollBeyondLastLine: false,
          tabSize: 2,
          renderLineHighlight: 'all',
          cursorBlinking: 'smooth',
          bracketPairColorization: { enabled: true },
        });

        this.editor.onDidChangeModelContent(() => {
          this.triggerAnalysisDebounced();
        });

        // Bi-directional correlation: Cursor position in editor -> inspect SSA variable in CFG
        this.editor.onDidChangeCursorPosition((e) => {
          const model = this.editor.getModel();
          if (!model) return;
          const word = model.getWordAtPosition(e.position);
          if (!word) return;

          const lineContent = model.getLineContent(e.position.lineNumber);
          const startCol = word.startColumn;
          const charBefore = startCol > 1 ? lineContent[startCol - 2] : '';
          const candidate = charBefore === '%' ? `%${word.word}` : `%${word.word}`;

          const cfg = this.currentFunction && this.analysisResult?.cfgs
            ? this.analysisResult.cfgs[this.currentFunction]
            : null;

          if (cfg && cfg.ssaMap && cfg.ssaMap[candidate]) {
            this.inspectVariable(candidate, false);
          }
        });

        resolve();
      });
    });
  }

  loadInitialContent() {
    let initialCode = '';
    const hash = window.location.hash;
    if (hash && hash.startsWith('#code=')) {
      try {
        const encoded = hash.slice('#code='.length);
        initialCode = decodeURIComponent(atob(encoded));
      } catch (e) {
        console.warn('Failed to decode code from URL hash', e);
      }
    }

    if (!initialCode) {
      initialCode = PRESETS.collatz.code.trim();
    }

    this.editor.setValue(initialCode);
  }

  triggerAnalysisDebounced() {
    if (this.debounceTimer) clearTimeout(this.debounceTimer);
    this.debounceTimer = setTimeout(() => {
      this.runAnalysis();
    }, 250);
  }

  runAnalysis() {
    if (!this.editor) return;

    const source = this.editor.getValue();
    this.analysisResult = this.analyzer.analyze(source);

    // 1. Update Monaco Editor Error Markers
    this.updateEditorMarkers(this.analysisResult.diagnostics);

    // 2. Update Function Dropdown
    this.updateFunctionDropdown(this.analysisResult.functions);

    // 3. Update Diagnostics Drawer / Badge
    this.updateDiagnosticsUI(this.analysisResult.diagnostics);

    // 4. Render Active View
    this.renderCurrentView();

    // 5. Update Shareable URL Hash
    this.updateUrlHash(source);
  }

  updateEditorMarkers(diagnostics) {
    if (!window.monaco || !this.editor) return;

    const model = this.editor.getModel();
    if (!model) return;

    const markers = diagnostics.map((d) => ({
      severity:
        d.severity === 'Error'
          ? window.monaco.MarkerSeverity.Error
          : d.severity === 'Warning'
          ? window.monaco.MarkerSeverity.Warning
          : window.monaco.MarkerSeverity.Info,
      startLineNumber: d.line,
      startColumn: d.column || 1,
      endLineNumber: d.line,
      endColumn: model.getLineMaxColumn(d.line) || 80,
      message: `[${d.code}] ${d.message}`,
    }));

    window.monaco.editor.setModelMarkers(model, 'llvm-ir-studio', markers);
  }

  updateFunctionDropdown(functions) {
    const select = document.getElementById('function-select');
    const prevSelected = this.currentFunction;
    select.innerHTML = '';

    if (functions.length === 0) {
      const opt = document.createElement('option');
      opt.textContent = 'No functions found';
      opt.disabled = true;
      select.appendChild(opt);
      this.currentFunction = null;
      return;
    }

    let foundPrev = false;
    for (const f of functions) {
      const opt = document.createElement('option');
      opt.value = f.name;
      opt.textContent = `@${f.name}`;
      if (f.name === prevSelected) {
        opt.selected = true;
        foundPrev = true;
      }
      select.appendChild(opt);
    }

    if (!foundPrev && functions.length > 0) {
      this.currentFunction = functions[0].name;
      select.value = this.currentFunction;
    } else {
      this.currentFunction = prevSelected || functions[0].name;
    }
  }

  updateDiagnosticsUI(diagnostics) {
    const errorCount = diagnostics.filter((d) => d.severity === 'Error').length;
    const warnCount = diagnostics.filter((d) => d.severity === 'Warning').length;

    const badge = document.getElementById('diag-badge');
    badge.textContent = `${errorCount} errors, ${warnCount} warnings`;
    badge.className = errorCount > 0 ? 'badge error' : warnCount > 0 ? 'badge warn' : 'badge ok';

    const diagList = document.getElementById('diagnostics-list');
    if (diagnostics.length === 0) {
      diagList.innerHTML = '<div class="empty-state">No syntax or CFG diagnostics detected. All basic blocks well-formed.</div>';
    } else {
      diagList.innerHTML = diagnostics
        .map(
          (d) => `
        <div class="diag-item ${d.severity.toLowerCase()}" onclick="window.app.jumpToLine(${d.line})">
          <span class="diag-sev">[${d.severity.toUpperCase()}]</span>
          <span class="diag-loc">Line ${d.line}:</span>
          <span class="diag-msg">${d.message}</span>
          <span class="diag-code">(${d.code})</span>
        </div>
      `
        )
        .join('');
    }
  }

  async renderCurrentView() {
    const cfg = this.currentFunction && this.analysisResult?.cfgs
      ? this.analysisResult.cfgs[this.currentFunction]
      : null;

    if (this.activeTab === 'graph') {
      await this.renderGraph(cfg);
    } else if (this.activeTab === 'mermaid') {
      const mermaidCode = this.visualizer.toMermaid(cfg);
      document.getElementById('mermaid-source').textContent = mermaidCode;
    } else if (this.activeTab === 'dot') {
      const dotCode = this.visualizer.toDot(cfg);
      document.getElementById('dot-source').textContent = dotCode;
    }
  }

  async renderGraph(cfg) {
    const container = document.getElementById('graph-target');
    if (!cfg) {
      container.innerHTML = '<div class="empty-state">Select or write a function definition (define ...) to view its CFG.</div>';
      return;
    }

    const mermaidSource = this.visualizer.toMermaid(cfg);

    try {
      if (window.mermaid) {
        window.mermaid.initialize({
          startOnLoad: false,
          theme: 'dark',
          securityLevel: 'loose',
          flowchart: {
            curve: 'basis',
            nodeSpacing: 40,
            rankSpacing: 40,
            htmlLabels: true,
          },
        });

        const id = `mermaid-svg-${Date.now()}`;
        const { svg } = await window.mermaid.render(id, mermaidSource);
        container.innerHTML = svg;

        // Attach interactive click listeners to nodes and variable pills
        this.attachInteractiveListeners(container, cfg);
      }
    } catch (err) {
      console.error('Mermaid render error:', err);
      container.innerHTML = `<div class="error-state"><b>Graph Render Error:</b><br/>${err.message || String(err)}</div>`;
    }
  }

  attachInteractiveListeners(container, cfg) {
    // 1. Variable Pill click listeners (SSA Backtracking)
    const pills = container.querySelectorAll('.var-pill');
    pills.forEach((pill) => {
      pill.addEventListener('click', (e) => {
        e.stopPropagation();
        const varName = pill.dataset.var;
        if (varName) {
          this.inspectVariable(varName);
        }
      });
    });

    // 2. Node click listeners (Jump to Block)
    const nodes = container.querySelectorAll('.node');
    nodes.forEach((node) => {
      node.style.cursor = 'pointer';
      node.addEventListener('click', (e) => {
        if (this.hasDragged) return;
        if (e.target.closest('.var-pill')) return;

        const textContent = node.textContent || '';
        for (const bbName of cfg.blockOrder) {
          if (textContent.includes(bbName)) {
            const bb = cfg.blocks[bbName];
            if (bb && typeof bb.line === 'number') {
              this.jumpToLine(bb.line + 1);
              this.showToast(`Navigated to basic block '%${bbName}'`);
            }
            break;
          }
        }
      });
    });
  }

  // ============================================================
  // SSA Variable Backtrack & Editor-Graph Correlation
  // ============================================================
  inspectVariable(varName, shouldScrollEditor = true) {
    if (!varName) return;
    const normalized = varName.startsWith('%') ? varName : `%${varName}`;

    const cfg = this.currentFunction && this.analysisResult?.cfgs
      ? this.analysisResult.cfgs[this.currentFunction]
      : null;

    if (!cfg || !cfg.ssaMap) return;

    const varInfo = cfg.ssaMap[normalized];
    if (!varInfo) {
      this.showToast(`Variable '${normalized}' not found in SSA map`);
      return;
    }

    this.activeVariable = normalized;

    // 1. Correlate with Monaco Editor (Line decorations)
    this.correlateInEditor(varInfo, shouldScrollEditor);

    // 2. Correlate in Graph Canvas (Pill badges & Node glow)
    this.correlateInGraph(normalized, varInfo);

    // 3. Open and populate SSA Backtrack Drawer
    this.populateSsaDrawer(normalized, varInfo);

    this.showToast(`Backtracking SSA variable '${normalized}'`);
  }

  correlateInEditor(varInfo, shouldScroll) {
    if (!window.monaco || !this.editor) return;

    const newDecorations = [];

    // Definition line decoration (Gold/Amber)
    if (varInfo.line) {
      newDecorations.push({
        range: new window.monaco.Range(varInfo.line, 1, varInfo.line, 1),
        options: {
          isWholeLine: true,
          className: 'monaco-def-line',
          overviewRuler: {
            color: '#eab308',
            position: window.monaco.editor.OverviewRulerLane.Full,
          },
        },
      });

      if (shouldScroll) {
        this.editor.revealLineInCenter(varInfo.line);
        this.editor.setPosition({ lineNumber: varInfo.line, column: 1 });
      }
    }

    // Usage lines decorations (Cyan/Blue)
    if (varInfo.uses && varInfo.uses.length > 0) {
      for (const use of varInfo.uses) {
        if (use.line && use.line !== varInfo.line) {
          newDecorations.push({
            range: new window.monaco.Range(use.line, 1, use.line, 1),
            options: {
              isWholeLine: true,
              className: 'monaco-use-line',
              overviewRuler: {
                color: '#38bdf8',
                position: window.monaco.editor.OverviewRulerLane.Right,
              },
            },
          });
        }
      }
    }

    this.monacoDecorations = this.editor.deltaDecorations(this.monacoDecorations, newDecorations);
  }

  correlateInGraph(targetVar, varInfo) {
    const container = document.getElementById('graph-target');
    if (!container) return;

    // Reset previous pill highlights
    container.querySelectorAll('.var-pill').forEach((p) => {
      p.classList.remove('active-target', 'active-producer', 'active-consumer');
    });

    container.querySelectorAll('.bb-card').forEach((c) => {
      c.classList.remove('highlight-pulse');
    });

    // 1. Highlight target variable pills (Yellow/Amber)
    container.querySelectorAll(`.var-pill[data-var="${targetVar}"]`).forEach((p) => {
      p.classList.add('active-target');
    });

    // 2. Highlight upstream producers / inputs (Cyan)
    if (varInfo.inputs) {
      for (const inp of varInfo.inputs) {
        container.querySelectorAll(`.var-pill[data-var="${inp}"]`).forEach((p) => {
          p.classList.add('active-producer');
        });
      }
    }

    // 3. Highlight downstream consumers / uses (Purple)
    if (varInfo.uses) {
      for (const u of varInfo.uses) {
        if (u.targetVar) {
          container.querySelectorAll(`.var-pill[data-var="${u.targetVar}"]`).forEach((p) => {
            p.classList.add('active-consumer');
          });
        }
      }
    }

    // 4. Pulse the basic block containing the definition
    const defBlockName = varInfo.block;
    if (defBlockName) {
      const nodes = container.querySelectorAll('.node');
      nodes.forEach((n) => {
        if ((n.textContent || '').includes(defBlockName)) {
          const card = n.querySelector('.bb-card');
          if (card) card.classList.add('highlight-pulse');
        }
      });
    }
  }

  populateSsaDrawer(varName, varInfo) {
    const drawer = document.getElementById('ssa-drawer');
    if (!drawer) return;

    document.getElementById('ssa-var-name').textContent = varName;
    document.getElementById('ssa-var-type').textContent = varInfo.type || 'type';

    // Definition details
    const defEl = document.getElementById('ssa-def-text');
    defEl.innerHTML = `Defined at <b>Line ${varInfo.line}</b> in block <span style='color:#38bdf8;'>%${varInfo.block}</span>:<br/><span style='color:#facc15;font-weight:600;'>${varInfo.raw}</span>`;

    // Upstream inputs (Backtrack buttons)
    const inputsContainer = document.getElementById('ssa-inputs-chain');
    inputsContainer.innerHTML = '';
    if (!varInfo.inputs || varInfo.inputs.length === 0) {
      inputsContainer.innerHTML = `<span style='color:#64748b;font-size:0.7rem;font-style:italic;'>None (function parameter or constant)</span>`;
    } else {
      for (const inp of varInfo.inputs) {
        const btn = document.createElement('button');
        btn.className = 'ssa-chain-btn';
        btn.innerHTML = `← <span style='font-weight:700;'>${inp}</span>`;
        btn.title = `Backtrack to producer of ${inp}`;
        btn.addEventListener('click', () => this.inspectVariable(inp));
        inputsContainer.appendChild(btn);
      }
    }

    // Downstream consumers (Usage buttons)
    const usesContainer = document.getElementById('ssa-uses-chain');
    usesContainer.innerHTML = '';
    if (!varInfo.uses || varInfo.uses.length === 0) {
      usesContainer.innerHTML = `<span style='color:#64748b;font-size:0.7rem;font-style:italic;'>None (unused register)</span>`;
    } else {
      for (const use of varInfo.uses) {
        const btn = document.createElement('button');
        btn.className = 'ssa-chain-btn consumer';
        const label = use.targetVar ? `→ ${use.targetVar} (Line ${use.line})` : `→ ${use.block} branch (Line ${use.line})`;
        btn.innerHTML = `<span style='font-weight:700;'>${label}</span>`;
        btn.title = use.inst;
        if (use.targetVar) {
          btn.addEventListener('click', () => this.inspectVariable(use.targetVar));
        } else {
          btn.addEventListener('click', () => this.jumpToLine(use.line));
        }
        usesContainer.appendChild(btn);
      }
    }

    drawer.classList.add('open');
  }

  closeSsaDrawer() {
    const drawer = document.getElementById('ssa-drawer');
    if (drawer) drawer.classList.remove('open');

    // Clear Monaco decorations
    if (this.editor && this.monacoDecorations.length > 0) {
      this.monacoDecorations = this.editor.deltaDecorations(this.monacoDecorations, []);
    }

    // Reset pill highlights in graph
    const container = document.getElementById('graph-target');
    if (container) {
      container.querySelectorAll('.var-pill').forEach((p) => {
        p.classList.remove('active-target', 'active-producer', 'active-consumer');
      });
      container.querySelectorAll('.bb-card').forEach((c) => {
        c.classList.remove('highlight-pulse');
      });
    }

    this.activeVariable = null;
  }

  jumpToLine(line) {
    if (!this.editor) return;
    this.editor.revealLineInCenter(line);
    this.editor.setPosition({ lineNumber: line, column: 1 });
    this.editor.focus();
  }

  // ============================================================
  // Enhanced Pan & Zoom Controller with Cursor-Centric Zoom
  // ============================================================
  initPanZoom() {
    const viewport = document.getElementById('graph-viewport');

    viewport.addEventListener('mousedown', (e) => {
      if (e.target.closest('#canvas-hud') || e.target.closest('#ssa-drawer')) return;

      this.isPanning = true;
      this.hasDragged = false;
      this.dragStartX = e.clientX - this.panX;
      this.dragStartY = e.clientY - this.panY;
      viewport.classList.add('panning');
    });

    window.addEventListener('mousemove', (e) => {
      if (!this.isPanning) return;

      const newPanX = e.clientX - this.dragStartX;
      const newPanY = e.clientY - this.dragStartY;

      if (Math.abs(newPanX - this.panX) > 4 || Math.abs(newPanY - this.panY) > 4) {
        this.hasDragged = true;
      }

      this.panX = newPanX;
      this.panY = newPanY;
      this.applyTransform();
    });

    window.addEventListener('mouseup', () => {
      if (this.isPanning) {
        this.isPanning = false;
        viewport.classList.remove('panning');
      }
    });

    // Cursor-centric wheel zoom
    viewport.addEventListener('wheel', (e) => {
      e.preventDefault();

      const rect = viewport.getBoundingClientRect();
      const mouseX = e.clientX - rect.left;
      const mouseY = e.clientY - rect.top;

      const zoomFactor = e.deltaY < 0 ? 1.12 : 0.89;
      const nextScale = Math.max(0.15, Math.min(4.0, this.zoomScale * zoomFactor));

      this.panX = mouseX - (mouseX - this.panX) * (nextScale / this.zoomScale);
      this.panY = mouseY - (mouseY - this.panY) * (nextScale / this.zoomScale);
      this.zoomScale = nextScale;

      this.applyTransform();
    }, { passive: false });

    // Double click to fit
    viewport.addEventListener('dblclick', (e) => {
      if (e.target.closest('#canvas-hud') || e.target.closest('#ssa-drawer')) return;
      this.fitToScreen();
    });
  }

  zoomAtCenter(factor) {
    const viewport = document.getElementById('graph-viewport');
    const centerX = viewport.clientWidth / 2;
    const centerY = viewport.clientHeight / 2;

    const nextScale = Math.max(0.15, Math.min(4.0, this.zoomScale * factor));
    this.panX = centerX - (centerX - this.panX) * (nextScale / this.zoomScale);
    this.panY = centerY - (centerY - this.panY) * (nextScale / this.zoomScale);
    this.zoomScale = nextScale;

    this.applyTransform();
  }

  resetPanZoom() {
    this.zoomScale = 1.0;
    this.panX = 40;
    this.panY = 60;
    this.applyTransform();
  }

  fitToScreen() {
    const viewport = document.getElementById('graph-viewport');
    const svgEl = document.querySelector('#graph-target svg');
    if (!svgEl) return;

    let svgW = 0;
    let svgH = 0;

    try {
      const bbox = svgEl.getBBox();
      svgW = bbox.width || svgEl.clientWidth;
      svgH = bbox.height || svgEl.clientHeight;
    } catch (e) {
      svgW = svgEl.clientWidth || 600;
      svgH = svgEl.clientHeight || 400;
    }

    if (svgW <= 0 || svgH <= 0) return;

    const pad = 60;
    const availW = Math.max(100, viewport.clientWidth - pad * 2);
    const availH = Math.max(100, viewport.clientHeight - pad * 2);

    const scaleX = availW / svgW;
    const scaleY = availH / svgH;
    this.zoomScale = Math.max(0.2, Math.min(1.5, Math.min(scaleX, scaleY)));

    this.panX = (viewport.clientWidth - svgW * this.zoomScale) / 2;
    this.panY = Math.max(40, (viewport.clientHeight - svgH * this.zoomScale) / 2);

    this.applyTransform();
    this.showToast('Graph fitted to view');
  }

  applyTransform() {
    const target = document.getElementById('graph-target');
    const viewport = document.getElementById('graph-viewport');
    if (!target) return;

    target.style.transform = `translate(${this.panX}px, ${this.panY}px) scale(${this.zoomScale})`;

    if (viewport) {
      viewport.style.backgroundPosition = `${this.panX}px ${this.panY}px`;
    }

    const pct = Math.round(this.zoomScale * 100);
    const label = document.getElementById('hud-zoom-percent');
    if (label) {
      label.textContent = `${pct}%`;
    }
  }

  setupEventListeners() {
    // Function switcher
    document.getElementById('function-select').addEventListener('change', (e) => {
      this.currentFunction = e.target.value;
      this.closeSsaDrawer();
      this.renderCurrentView();
    });

    // LLVM Version switcher
    document.getElementById('version-select').addEventListener('change', (e) => {
      const ver = parseInt(e.target.value, 10);
      this.analyzer.setVersion(ver);
      this.runAnalysis();
      this.showToast(`Target LLVM version set to ${ver}`);
    });

    // Density Selector (Compact / Standard / Full)
    document.getElementById('density-select').addEventListener('change', (e) => {
      this.visualizer.setDensity(e.target.value);
      this.renderCurrentView();
      this.showToast(`Card density set to '${e.target.value}'`);
    });

    // View Mode Selector (Simplified vs Raw)
    const viewModeSelect = document.getElementById('view-mode-select');
    if (viewModeSelect) {
      viewModeSelect.addEventListener('change', (e) => {
        this.visualizer.setViewMode(e.target.value);
        this.renderCurrentView();
        this.showToast(e.target.value === 'simplified' ? 'View switched to Simplified Pseudo-Math' : 'View switched to Raw LLVM IR');
      });
    }

    // HUD Orientation toggle (TD vs LR)
    document.getElementById('hud-layout-btn').addEventListener('click', () => {
      const next = this.visualizer.layout === 'TD' ? 'LR' : 'TD';
      this.visualizer.setLayout(next);
      document.getElementById('hud-layout-btn').textContent = `Orientation: ${next}`;
      this.renderCurrentView();
    });

    // HUD Zoom controls
    document.getElementById('hud-zoom-in').addEventListener('click', () => this.zoomAtCenter(1.2));
    document.getElementById('hud-zoom-out').addEventListener('click', () => this.zoomAtCenter(1 / 1.2));
    document.getElementById('hud-fit').addEventListener('click', () => this.fitToScreen());
    document.getElementById('hud-reset').addEventListener('click', () => this.resetPanZoom());

    // SSA Drawer Close Button
    const closeBtn = document.getElementById('ssa-close-btn');
    if (closeBtn) {
      closeBtn.addEventListener('click', () => this.closeSsaDrawer());
    }

    // Basic Block Search / Jump input
    const searchInput = document.getElementById('block-search');
    const handleBlockSearch = () => {
      const query = searchInput.value.trim().toLowerCase();
      if (!query) return;

      const nodes = document.querySelectorAll('#graph-target .node');
      let found = null;

      nodes.forEach((n) => {
        const text = (n.textContent || '').toLowerCase();
        const card = n.querySelector('.bb-card');
        if (card) card.classList.remove('highlight-pulse');

        if (!found && text.includes(query)) {
          found = n;
        }
      });

      if (found) {
        const card = found.querySelector('.bb-card');
        if (card) card.classList.add('highlight-pulse');

        const viewport = document.getElementById('graph-viewport');
        const nodeRect = found.getBoundingClientRect();
        const viewRect = viewport.getBoundingClientRect();

        const nodeCenterX = nodeRect.left + nodeRect.width / 2 - viewRect.left;
        const nodeCenterY = nodeRect.top + nodeRect.height / 2 - viewRect.top;

        this.panX += viewRect.width / 2 - nodeCenterX;
        this.panY += viewRect.height / 2 - nodeCenterY;
        this.applyTransform();

        const cfg = this.currentFunction ? this.analysisResult?.cfgs[this.currentFunction] : null;
        if (cfg) {
          for (const bbName of cfg.blockOrder) {
            if ((found.textContent || '').includes(bbName)) {
              const bb = cfg.blocks[bbName];
              if (bb && typeof bb.line === 'number') {
                this.jumpToLine(bb.line + 1);
              }
              break;
            }
          }
        }
        this.showToast(`Found basic block matching '${query}'`);
      } else {
        this.showToast(`No basic block matching '${query}' found`);
      }
    };

    searchInput.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') handleBlockSearch();
    });

    // Tab buttons
    document.querySelectorAll('.tab-btn').forEach((btn) => {
      btn.addEventListener('click', () => {
        document.querySelectorAll('.tab-btn').forEach((b) => b.classList.remove('active'));
        document.querySelectorAll('.view-panel').forEach((p) => p.classList.remove('active'));

        btn.classList.add('active');
        const tab = btn.dataset.tab;
        this.activeTab = tab;
        document.getElementById(`panel-${tab}`).classList.add('active');
        this.renderCurrentView();
      });
    });

    // Export options
    document.getElementById('btn-copy-mermaid').addEventListener('click', () => {
      const cfg = this.currentFunction ? this.analysisResult?.cfgs[this.currentFunction] : null;
      const code = this.visualizer.toMermaid(cfg);
      navigator.clipboard.writeText(code);
      this.showToast('Copied Mermaid code to clipboard!');
    });

    document.getElementById('btn-copy-dot').addEventListener('click', () => {
      const cfg = this.currentFunction ? this.analysisResult?.cfgs[this.currentFunction] : null;
      const code = this.visualizer.toDot(cfg);
      navigator.clipboard.writeText(code);
      this.showToast('Copied DOT graph to clipboard!');
    });

    document.getElementById('btn-export-svg').addEventListener('click', () => {
      const svgEl = document.querySelector('#graph-target svg');
      if (!svgEl) {
        this.showToast('No SVG graph available to export');
        return;
      }
      const svgData = new XMLSerializer().serializeToString(svgEl);
      const blob = new Blob([svgData], { type: 'image/svg+xml;charset=utf-8' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${this.currentFunction || 'llvm-ir'}-cfg.svg`;
      a.click();
      URL.revokeObjectURL(url);
      this.showToast('SVG exported successfully!');
    });

    document.getElementById('btn-share').addEventListener('click', () => {
      const source = this.editor.getValue();
      const encoded = btoa(encodeURIComponent(source));
      const fullUrl = `${window.location.origin}${window.location.pathname}#code=${encoded}`;
      navigator.clipboard.writeText(fullUrl);
      this.showToast('Shareable playground URL copied to clipboard!');
    });

    // Resizable split pane divider
    this.setupSplitter();
  }

  setupSplitter() {
    const divider = document.getElementById('pane-divider');
    const leftPane = document.getElementById('editor-pane');
    let isDragging = false;

    divider.addEventListener('mousedown', () => {
      isDragging = true;
      document.body.style.cursor = 'col-resize';
      document.body.style.userSelect = 'none';
    });

    window.addEventListener('mousemove', (e) => {
      if (!isDragging) return;
      const containerWidth = document.getElementById('workspace').offsetWidth;
      const newWidth = Math.max(300, Math.min(containerWidth - 300, e.clientX));
      leftPane.style.width = `${newWidth}px`;
      if (this.editor) this.editor.layout();
    });

    window.addEventListener('mouseup', () => {
      if (isDragging) {
        isDragging = false;
        document.body.style.cursor = '';
        document.body.style.userSelect = '';
        if (this.editor) this.editor.layout();
      }
    });
  }

  updateUrlHash(source) {
    try {
      const encoded = btoa(encodeURIComponent(source));
      window.history.replaceState(null, '', `#code=${encoded}`);
    } catch (e) {
      // Ignore hash update if too long
    }
  }

  showToast(message) {
    const toast = document.getElementById('toast');
    toast.textContent = message;
    toast.classList.add('show');
    setTimeout(() => {
      toast.classList.remove('show');
    }, 2800);
  }
}

window.addEventListener('DOMContentLoaded', () => {
  window.app = new LlvmStudioApp();
  window.app.init();
});

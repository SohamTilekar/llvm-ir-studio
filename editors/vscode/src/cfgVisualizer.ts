import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';
import { execFile } from 'child_process';
import { promisify } from 'util';
import { ServerManager } from './serverManager';

const execFileAsync = promisify(execFile);

export class CfgVisualizer {
  private currentPanels: Map<string, vscode.WebviewPanel> = new Map();

  constructor(
    private readonly context: vscode.ExtensionContext,
    private readonly serverManager: ServerManager
  ) {
    // Automatically update open CFG panels when user saves the LLVM IR file
    context.subscriptions.push(
      vscode.workspace.onDidSaveTextDocument(async (doc) => {
        if (doc.languageId === 'llvm' || doc.fileName.endsWith('.ll')) {
          for (const [key, panel] of this.currentPanels.entries()) {
            if (key.startsWith(doc.fileName + ':')) {
              const funcName = key.slice(doc.fileName.length + 1);
              await this.renderCfg(panel, doc.fileName, funcName);
            }
          }
        }
      })
    );
  }

  public async showCfg(): Promise<void> {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      vscode.window.showErrorMessage('No active LLVM IR editor.');
      return;
    }

    const doc = editor.document;
    if (doc.languageId !== 'llvm' && !doc.fileName.endsWith('.ll')) {
      vscode.window.showWarningMessage('Active file is not recognized as an LLVM IR file (.ll).');
      return;
    }

    // Save document if dirty to ensure server reads latest state
    if (doc.isDirty) {
      await doc.save();
    }

    const text = doc.getText();
    const functions = this.extractFunctions(text);

    if (functions.length === 0) {
      vscode.window.showWarningMessage('No function definitions (define ...) found in this file.');
      return;
    }

    // Determine current function under cursor
    const cursorLine = editor.selection.active.line;
    let selectedFunc = this.findFunctionAtLine(functions, cursorLine);

    if (!selectedFunc) {
      if (functions.length === 1) {
        selectedFunc = functions[0].name;
      } else {
        const picked = await vscode.window.showQuickPick(
          functions.map((f) => ({
            label: `@${f.name}`,
            description: `Lines ${f.startLine + 1} - ${f.endLine + 1}`,
            functionName: f.name,
          })),
          { placeHolder: 'Select a function to visualize CFG' }
        );
        if (!picked) return;
        selectedFunc = picked.functionName;
      }
    }

    const panelKey = `${doc.fileName}:${selectedFunc}`;
    let panel = this.currentPanels.get(panelKey);

    if (!panel) {
      panel = vscode.window.createWebviewPanel(
        'llvmIrCfg',
        `CFG: @${selectedFunc}`,
        vscode.ViewColumn.Beside,
        {
          enableScripts: true,
          retainContextWhenHidden: true,
          localResourceRoots: [this.context.extensionUri],
        }
      );

      this.currentPanels.set(panelKey, panel);
      panel.onDidDispose(() => {
        this.currentPanels.delete(panelKey);
      });

      // Handle messages from the webview (e.g. copy or refresh)
      panel.webview.onDidReceiveMessage(async (msg) => {
        if (msg.type === 'refresh') {
          await this.renderCfg(panel!, doc.fileName, selectedFunc!);
        } else if (msg.type === 'copyMermaid') {
          await vscode.env.clipboard.writeText(msg.text);
          vscode.window.showInformationMessage('Mermaid CFG copied to clipboard!');
        }
      });
    }

    panel.reveal(vscode.ViewColumn.Beside);
    await this.renderCfg(panel, doc.fileName, selectedFunc);
  }

  private async renderCfg(panel: vscode.WebviewPanel, filePath: string, funcName: string): Promise<void> {
    try {
      const serverBin = await this.serverManager.getBinaryPath();

      // Retrieve pure Mermaid graph from llvm-ir-ls
      const { stdout: mermaidCode } = await execFileAsync(serverBin, [
        'cfg',
        filePath,
        '--fn',
        `@${funcName}`,
        '--format',
        'mermaid',
      ]);

      panel.webview.html = this.buildHtml(funcName, path.basename(filePath), mermaidCode.trim());
    } catch (err: any) {
      panel.webview.html = this.buildErrorHtml(funcName, err.message || String(err));
    }
  }

  private extractFunctions(text: string): Array<{ name: string; startLine: number; endLine: number }> {
    const lines = text.split(/\r?\n/);
    const functions: Array<{ name: string; startLine: number; endLine: number }> = [];

    const defRegex = /^\s*define\b[^{]*@([a-zA-Z$._0-9]+|"[^"]+")\s*\(/;
    let currentFunc: { name: string; startLine: number } | null = null;

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      const match = line.match(defRegex);
      if (match) {
        currentFunc = { name: match[1].replace(/^"|"$/g, ''), startLine: i };
      }
      if (currentFunc && line.trim() === '}') {
        functions.push({
          name: currentFunc.name,
          startLine: currentFunc.startLine,
          endLine: i,
        });
        currentFunc = null;
      }
    }

    return functions;
  }

  private findFunctionAtLine(
    functions: Array<{ name: string; startLine: number; endLine: number }>,
    line: number
  ): string | null {
    for (const f of functions) {
      if (line >= f.startLine && line <= f.endLine) {
        return f.name;
      }
    }
    return null;
  }

  private buildHtml(funcName: string, fileName: string, mermaidSource: string): string {
    const escapedMermaid = mermaidSource
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');

    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>CFG: @${funcName}</title>
  <style>
    :root {
      --bg: var(--vscode-editor-background, #1e1e1e);
      --fg: var(--vscode-editor-foreground, #d4d4d4);
      --card-bg: var(--vscode-editorWidget-background, #252526);
      --border: var(--vscode-editorWidget-border, #454545);
      --accent: var(--vscode-textLink-foreground, #3794ff);
      --btn-bg: var(--vscode-button-background, #0e639c);
      --btn-fg: var(--vscode-button-foreground, #ffffff);
      --btn-hover: var(--vscode-button-hoverBackground, #1177bb);
    }
    body {
      margin: 0;
      padding: 16px;
      font-family: var(--vscode-font-family, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif);
      background-color: var(--bg);
      color: var(--fg);
      display: flex;
      flex-direction: column;
      height: 100vh;
      box-sizing: border-box;
      overflow: hidden;
    }
    .toolbar {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 8px 12px;
      background: var(--card-bg);
      border: 1px solid var(--border);
      border-radius: 6px;
      margin-bottom: 12px;
      flex-shrink: 0;
    }
    .title-group {
      display: flex;
      align-items: center;
      gap: 10px;
    }
    h2 {
      margin: 0;
      font-size: 1.1rem;
      font-family: var(--vscode-editor-font-family, monospace);
      color: var(--accent);
    }
    .badge {
      font-size: 0.75rem;
      padding: 2px 8px;
      background: rgba(255, 255, 255, 0.08);
      border-radius: 4px;
      opacity: 0.8;
    }
    .actions {
      display: flex;
      gap: 8px;
    }
    button {
      background: var(--btn-bg);
      color: var(--btn-fg);
      border: none;
      padding: 4px 10px;
      border-radius: 4px;
      cursor: pointer;
      font-size: 0.8rem;
      display: flex;
      align-items: center;
      gap: 4px;
    }
    button:hover {
      background: var(--btn-hover);
    }
    #viewport {
      flex: 1;
      border: 1px solid var(--border);
      border-radius: 6px;
      background: var(--card-bg);
      overflow: auto;
      display: flex;
      align-items: center;
      justify-content: center;
      position: relative;
    }
    #mermaid-container {
      padding: 24px;
      transform-origin: center center;
      transition: transform 0.1s ease-out;
    }
    .mermaid {
      font-family: var(--vscode-editor-font-family, monospace);
    }
    .mermaid svg {
      max-width: none !important;
      height: auto !important;
    }
  </style>
  <script type="module">
    import mermaid from 'https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.esm.min.mjs';
    const isDark = document.body.classList.contains('vscode-dark') || !document.body.classList.contains('vscode-light');
    mermaid.initialize({
      startOnLoad: true,
      theme: isDark ? 'dark' : 'default',
      securityLevel: 'loose',
      flowchart: {
        curve: 'basis',
        nodeSpacing: 45,
        rankSpacing: 45,
        htmlLabels: true
      }
    });

    const vscode = acquireVsCodeApi();
    window.refreshCfg = () => vscode.postMessage({ type: 'refresh' });
    window.copyMermaid = () => vscode.postMessage({ type: 'copyMermaid', text: ${JSON.stringify(mermaidSource)} });

    let scale = 1.0;
    const container = document.getElementById('mermaid-container');
    window.zoomIn = () => { scale *= 1.2; container.style.transform = \`scale(\${scale})\`; };
    window.zoomOut = () => { scale /= 1.2; container.style.transform = \`scale(\${scale})\`; };
    window.resetZoom = () => { scale = 1.0; container.style.transform = 'scale(1)'; };
  </script>
</head>
<body class="vscode-dark">
  <div class="toolbar">
    <div class="title-group">
      <h2>@${funcName}</h2>
      <span class="badge">${fileName}</span>
    </div>
    <div class="actions">
      <button onclick="window.zoomIn()">Zoom +</button>
      <button onclick="window.zoomOut()">Zoom -</button>
      <button onclick="window.resetZoom()">Reset</button>
      <button onclick="window.copyMermaid()">Copy Source</button>
      <button onclick="window.refreshCfg()">Refresh</button>
    </div>
  </div>
  <div id="viewport">
    <div id="mermaid-container">
      <pre class="mermaid">
${escapedMermaid}
      </pre>
    </div>
  </div>
</body>
</html>`;
  }

  private buildErrorHtml(funcName: string, error: string): string {
    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>CFG Error</title>
  <style>
    body {
      font-family: var(--vscode-font-family, sans-serif);
      padding: 24px;
      color: var(--vscode-errorForeground, #f48771);
      background: var(--vscode-editor-background, #1e1e1e);
    }
    pre {
      background: rgba(255, 0, 0, 0.1);
      border: 1px solid rgba(255, 0, 0, 0.3);
      padding: 12px;
      border-radius: 4px;
      font-family: monospace;
      color: #ff9999;
    }
  </style>
</head>
<body>
  <h3>Failed to generate CFG for function <code>@${funcName}</code></h3>
  <pre>${error.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')}</pre>
</body>
</html>`;
  }
}

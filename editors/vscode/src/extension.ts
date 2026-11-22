import * as vscode from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  Trace,
} from 'vscode-languageclient/node';
import { ServerManager } from './serverManager';
import { CfgVisualizer } from './cfgVisualizer';
import { execFile } from 'child_process';
import { promisify } from 'util';

const execFileAsync = promisify(execFile);

let client: LanguageClient | undefined;
let outputChannel: vscode.OutputChannel;

export async function activate(context: vscode.ExtensionContext) {
  outputChannel = vscode.window.createOutputChannel('LLVM IR Advance');
  context.subscriptions.push(outputChannel);
  outputChannel.appendLine('LLVM IR Advance extension activating...');

  const serverManager = new ServerManager(context);
  const cfgVisualizer = new CfgVisualizer(context, serverManager);

  // Register commands
  context.subscriptions.push(
    vscode.commands.registerCommand('llvm-ir.showCfg', async () => {
      await cfgVisualizer.showCfg();
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('llvm-ir.restartServer', async () => {
      outputChannel.appendLine('Restarting LLVM IR language server...');
      await stopServer();
      await startServer(context, serverManager);
      vscode.window.showInformationMessage('LLVM IR Language Server restarted.');
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('llvm-ir.checkFile', async () => {
      await runCheckCommand(serverManager);
    })
  );

  // Listen for configuration changes
  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration(async (e) => {
      if (e.affectsConfiguration('llvm-ir')) {
        outputChannel.appendLine('Configuration changed; restarting language server...');
        await stopServer();
        await startServer(context, serverManager);
      }
    })
  );

  // Start LSP server
  await startServer(context, serverManager);
}

async function startServer(context: vscode.ExtensionContext, serverManager: ServerManager) {
  try {
    const binaryPath = await serverManager.getBinaryPath();
    outputChannel.appendLine(`Using llvm-ir-ls at: ${binaryPath}`);

    const config = vscode.workspace.getConfiguration('llvm-ir');
    const initOptions = {
      version: config.get<number | null>('version'),
      verify_on_save: config.get<boolean>('verifyOnSave', true),
      llvm_as_path: config.get<string | null>('llvmAsPath'),
    };

    const serverOptions: ServerOptions = {
      run: { command: binaryPath, args: ['lsp'] },
      debug: { command: binaryPath, args: ['lsp'] },
    };

    const clientOptions: LanguageClientOptions = {
      documentSelector: [{ scheme: 'file', language: 'llvm' }],
      synchronize: {
        configurationSection: 'llvm-ir',
      },
      initializationOptions: initOptions,
      outputChannel,
    };

    client = new LanguageClient(
      'llvm-ir-ls',
      'LLVM IR Language Server',
      serverOptions,
      clientOptions
    );

    const traceLevel = config.get<string>('trace.server', 'off');
    if (traceLevel === 'messages') {
      client.setTrace(Trace.Messages);
    } else if (traceLevel === 'verbose') {
      client.setTrace(Trace.Verbose);
    }

    await client.start();
    outputChannel.appendLine('LLVM IR Language Server started successfully.');
  } catch (err: any) {
    outputChannel.appendLine(`Failed to start language server: ${err.message || err}`);
    vscode.window.showErrorMessage(
      `Failed to start LLVM IR Language Server: ${err.message || err}`
    );
  }
}

async function stopServer(): Promise<void> {
  if (client) {
    await client.stop();
    client = undefined;
  }
}

async function runCheckCommand(serverManager: ServerManager): Promise<void> {
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

  if (doc.isDirty) {
    await doc.save();
  }

  try {
    const binaryPath = await serverManager.getBinaryPath();
    const config = vscode.workspace.getConfiguration('llvm-ir');
    const ver = config.get<number | null>('version');

    const args = ['check', doc.fileName, '--format', 'text'];
    if (ver) {
      args.push('--version', ver.toString());
    }

    const { stdout, stderr } = await execFileAsync(binaryPath, args);
    const result = (stdout + '\n' + stderr).trim();

    outputChannel.clear();
    outputChannel.appendLine(`=== Diagnostics Check for ${doc.fileName} ===`);
    outputChannel.appendLine(result || 'No diagnostics or errors detected.');
    outputChannel.show(true);

    vscode.window.showInformationMessage('LLVM IR check completed. See Output channel for details.');
  } catch (err: any) {
    const output = (err.stdout || '') + '\n' + (err.stderr || '') + '\n' + (err.message || '');
    outputChannel.clear();
    outputChannel.appendLine(`=== Diagnostics Check for ${doc.fileName} ===`);
    outputChannel.appendLine(output.trim());
    outputChannel.show(true);
  }
}

export async function deactivate(): Promise<void> {
  await stopServer();
}

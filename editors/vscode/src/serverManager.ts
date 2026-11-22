import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';
import * as os from 'os';
import * as https from 'https';
import { execFile } from 'child_process';
import { promisify } from 'util';

const execFileAsync = promisify(execFile);
const GITHUB_REPO = 'SohamTilekar/llvm-ir-studio';

export class ServerManager {
  private cachedPath: string | null = null;

  constructor(private readonly context: vscode.ExtensionContext) {}

  public async getBinaryPath(): Promise<string> {
    if (this.cachedPath && fs.existsSync(this.cachedPath)) {
      return this.cachedPath;
    }

    const resolved = await this.resolveBinary();
    if (resolved) {
      this.cachedPath = resolved;
      return resolved;
    }

    // If not found, prompt user to download or specify
    const action = await vscode.window.showWarningMessage(
      'LLVM IR Language Server (llvm-ir-ls) not found on PATH or in standard directories.',
      'Download Latest Release',
      'Locate Binary',
      'Cancel'
    );

    if (action === 'Download Latest Release') {
      const downloadedPath = await this.downloadLatestRelease();
      if (downloadedPath) {
        this.cachedPath = downloadedPath;
        return downloadedPath;
      }
    } else if (action === 'Locate Binary') {
      const selected = await vscode.window.showOpenDialog({
        canSelectFiles: true,
        canSelectFolders: false,
        canSelectMany: false,
        openLabel: 'Select llvm-ir-ls binary',
      });
      if (selected && selected.length > 0) {
        const customPath = selected[0].fsPath;
        await vscode.workspace
          .getConfiguration('llvm-ir')
          .update('serverPath', customPath, vscode.ConfigurationTarget.Global);
        this.cachedPath = customPath;
        return customPath;
      }
    }

    throw new Error('llvm-ir-ls binary not found. Please install llvm-ir-ls or configure "llvm-ir.serverPath".');
  }

  private async resolveBinary(): Promise<string | null> {
    const binaryName = process.platform === 'win32' ? 'llvm-ir-ls.exe' : 'llvm-ir-ls';

    // 1. User configuration override
    const configuredPath = vscode.workspace
      .getConfiguration('llvm-ir')
      .get<string | null>('serverPath');
    if (configuredPath && fs.existsSync(configuredPath)) {
      return configuredPath;
    }

    // 2. Check PATH
    const pathBinary = await this.findInPath(binaryName);
    if (pathBinary) {
      return pathBinary;
    }

    // 3. Check common user home locations (~/.local/bin, ~/.cargo/bin)
    const home = os.homedir();
    const candidateDirs = [
      path.join(home, '.local', 'bin'),
      path.join(home, '.cargo', 'bin'),
    ];

    for (const dir of candidateDirs) {
      const candidate = path.join(dir, binaryName);
      if (fs.existsSync(candidate)) {
        return candidate;
      }
    }

    // 4. Check workspace targets (for local development/debugging)
    if (vscode.workspace.workspaceFolders) {
      for (const wf of vscode.workspace.workspaceFolders) {
        const releaseTarget = path.join(wf.uri.fsPath, 'target', 'release', binaryName);
        if (fs.existsSync(releaseTarget)) {
          return releaseTarget;
        }
        const debugTarget = path.join(wf.uri.fsPath, 'target', 'debug', binaryName);
        if (fs.existsSync(debugTarget)) {
          return debugTarget;
        }
      }
    }

    // 5. Check global storage download cache
    const cachedGlobal = path.join(this.context.globalStorageUri.fsPath, 'bin', binaryName);
    if (fs.existsSync(cachedGlobal)) {
      return cachedGlobal;
    }

    return null;
  }

  private async findInPath(binaryName: string): Promise<string | null> {
    const cmd = process.platform === 'win32' ? 'where' : 'which';
    try {
      const { stdout } = await execFileAsync(cmd, [binaryName]);
      const lines = stdout.trim().split(/\r?\n/);
      if (lines.length > 0 && fs.existsSync(lines[0])) {
        return lines[0];
      }
    } catch {
      // not found in PATH
    }
    return null;
  }

  private async downloadLatestRelease(): Promise<string | null> {
    const binaryName = process.platform === 'win32' ? 'llvm-ir-ls.exe' : 'llvm-ir-ls';
    const targetTriple = this.getTargetTriple();
    if (!targetTriple) {
      vscode.window.showErrorMessage(
        `Prebuilt binaries are not available for platform ${process.platform} (${process.arch}). Please build with 'cargo install --path .' from the repository.`
      );
      return null;
    }

    return vscode.window.withProgress(
      {
        location: vscode.ProgressLocation.Notification,
        title: 'Downloading llvm-ir-ls language server...',
        cancellable: false,
      },
      async (progress) => {
        try {
          progress.report({ message: 'Fetching release metadata...' });
          const release = await this.fetchJson(
            `https://api.github.com/repos/${GITHUB_REPO}/releases/latest`
          );

          const assetSuffix = process.platform === 'win32' ? '.zip' : '.tar.gz';
          const asset = release.assets?.find(
            (a: any) =>
              typeof a.name === 'string' &&
              a.name.includes(targetTriple) &&
              a.name.endsWith(assetSuffix)
          );

          if (!asset || !asset.browser_download_url) {
            throw new Error(`Asset matching target ${targetTriple} not found in latest release.`);
          }

          progress.report({ message: `Downloading ${asset.name}...` });
          const storageDir = path.join(this.context.globalStorageUri.fsPath, 'bin');
          if (!fs.existsSync(storageDir)) {
            fs.mkdirSync(storageDir, { recursive: true });
          }

          const archivePath = path.join(storageDir, asset.name);
          await this.downloadFile(asset.browser_download_url, archivePath);

          progress.report({ message: 'Extracting binary...' });
          if (assetSuffix === '.tar.gz') {
            await execFileAsync('tar', ['-xzf', archivePath, '-C', storageDir]);
          } else {
            // Windows zip extraction via powershell
            await execFileAsync('powershell', [
              '-Command',
              `Expand-Archive -Path '${archivePath}' -DestinationPath '${storageDir}' -Force`,
            ]);
          }

          // Clean up archive
          if (fs.existsSync(archivePath)) {
            fs.unlinkSync(archivePath);
          }

          const destination = path.join(storageDir, binaryName);
          if (fs.existsSync(destination)) {
            if (process.platform !== 'win32') {
              fs.chmodSync(destination, 0o755);
            }
            vscode.window.showInformationMessage('llvm-ir-ls installed successfully!');
            return destination;
          }

          throw new Error('Binary not found after extraction.');
        } catch (err: any) {
          vscode.window.showErrorMessage(`Failed to download llvm-ir-ls: ${err.message}`);
          return null;
        }
      }
    );
  }

  private getTargetTriple(): string | null {
    const { platform, arch } = process;
    if (platform === 'linux') {
      if (arch === 'x64') return 'x86_64-unknown-linux-gnu';
      if (arch === 'arm64') return 'aarch64-unknown-linux-gnu';
    } else if (platform === 'darwin') {
      if (arch === 'arm64') return 'aarch64-apple-darwin';
      if (arch === 'x64') return 'x86_64-apple-darwin';
    } else if (platform === 'win32') {
      if (arch === 'x64') return 'x86_64-pc-windows-msvc';
    }
    return null;
  }

  private fetchJson(url: string): Promise<any> {
    return new Promise((resolve, reject) => {
      const options = {
        headers: {
          'User-Agent': 'vscode-llvm-ir-advance',
        },
      };
      https
        .get(url, options, (res) => {
          if (res.statusCode && res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
            return resolve(this.fetchJson(res.headers.location));
          }
          if (res.statusCode !== 200) {
            return reject(new Error(`HTTP ${res.statusCode}: ${res.statusMessage}`));
          }
          let data = '';
          res.on('data', (chunk) => (data += chunk));
          res.on('end', () => {
            try {
              resolve(JSON.parse(data));
            } catch (e) {
              reject(e);
            }
          });
        })
        .on('error', reject);
    });
  }

  private downloadFile(url: string, dest: string): Promise<void> {
    return new Promise((resolve, reject) => {
      const file = fs.createWriteStream(dest);
      const options = {
        headers: {
          'User-Agent': 'vscode-llvm-ir-advance',
        },
      };

      const request = (targetUrl: string) => {
        https
          .get(targetUrl, options, (res) => {
            if (res.statusCode && res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
              return request(res.headers.location);
            }
            if (res.statusCode !== 200) {
              file.close();
              fs.unlinkSync(dest);
              return reject(new Error(`HTTP ${res.statusCode}: ${res.statusMessage}`));
            }
            res.pipe(file);
            file.on('finish', () => {
              file.close();
              resolve();
            });
          })
          .on('error', (err) => {
            file.close();
            if (fs.existsSync(dest)) fs.unlinkSync(dest);
            reject(err);
          });
      };

      request(url);
    });
  }
}

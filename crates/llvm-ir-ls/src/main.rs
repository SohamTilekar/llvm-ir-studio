mod playground;

use clap::{Parser, Subcommand};
use ir_analysis::analyze;
use ir_core::{Document, LlvmParser};
use ir_lsp::Backend;
use ir_version::detect_version;
use ir_viz::{export_cfg, VizFormat};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use tower_lsp::{LspService, Server};

#[derive(Parser, Debug)]
#[command(name = "llvm-ir-ls")]
#[command(author = "llvm-ir-ls contributors")]
#[command(version)]
#[command(about = "Universal LLVM IR Language Server & Visualizer", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the LSP language server over stdio (default)
    Lsp,

    /// Check an LLVM IR file for syntax, version, and SSA diagnostics
    Check {
        /// Path to .ll file
        file: PathBuf,

        /// Output format: 'text' or 'json' (machine/AI-friendly)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Explicit target LLVM version (e.g. 14, 17, 18)
        #[arg(long)]
        version: Option<u32>,
    },

    /// Extract and visualize the Control Flow Graph (CFG) of a function
    Cfg {
        /// Path to .ll file
        file: PathBuf,

        /// Function name to visualize (e.g. '@main' or 'main')
        #[arg(long = "fn")]
        func: Option<String>,

        /// Output format: 'mermaid', 'dot', or 'html'
        #[arg(short, long, default_value = "mermaid")]
        format: String,

        /// Optional output file path (defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Launch the interactive in-browser LLVM IR editor & visualizer playground
    Playground {
        /// Optional path to an LLVM IR file to pre-load
        file: Option<PathBuf>,

        /// Port to serve playground on (default 3000)
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Lsp) {
        Commands::Lsp => {
            // NEVER log to stdout in LSP mode; stdio is reserved for JSON-RPC
            tracing_subscriber::fmt()
                .with_writer(std::io::stderr)
                .init();

            let stdin = tokio::io::stdin();
            let stdout = tokio::io::stdout();

            let (service, socket) = LspService::new(|client| Backend::new(client));
            Server::new(stdin, stdout, socket).serve(service).await;
            ExitCode::SUCCESS
        }

        Commands::Check { file, format, version } => {
            let content = match fs::read_to_string(&file) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error reading file '{:?}': {}", file, e);
                    return ExitCode::FAILURE;
                }
            };

            let llvm_ver = detect_version(Some(&file), &content, version);
            let mut parser = LlvmParser::new();
            let doc = Document::new(&content);
            let tree = match parser.parse(&content, None) {
                Some(t) => t,
                None => {
                    eprintln!("Failed to parse file '{:?}'", file);
                    return ExitCode::FAILURE;
                }
            };

            let res = analyze(&doc, &tree, llvm_ver);

            let has_errors = res
                .diagnostics
                .iter()
                .any(|d| d.severity == tower_lsp::lsp_types::DiagnosticSeverity::ERROR);

            if format.to_lowercase() == "json" {
                let json_out = json!({
                    "file": file.to_string_lossy(),
                    "target_version": llvm_ver.major(),
                    "has_errors": has_errors,
                    "diagnostics": res.diagnostics,
                    "functions": res.symbols.functions.keys().collect::<Vec<_>>(),
                    "globals": res.symbols.globals.keys().collect::<Vec<_>>(),
                    "types": res.symbols.types.keys().collect::<Vec<_>>(),
                });
                println!("{}", serde_json::to_string_pretty(&json_out).unwrap());
            } else {
                println!(
                    "Checked '{}' targeting LLVM {} ({} diagnostics found)",
                    file.display(),
                    llvm_ver.major(),
                    res.diagnostics.len()
                );
                for d in &res.diagnostics {
                    let level = match d.severity {
                        tower_lsp::lsp_types::DiagnosticSeverity::ERROR => "error",
                        tower_lsp::lsp_types::DiagnosticSeverity::WARNING => "warning",
                        tower_lsp::lsp_types::DiagnosticSeverity::INFORMATION => "info",
                        tower_lsp::lsp_types::DiagnosticSeverity::HINT => "hint",
                        _ => "diag",
                    };
                    println!(
                        "  [{}] {}:{}:{}: {} [{}]",
                        level,
                        file.display(),
                        d.range.start.line + 1,
                        d.range.start.character + 1,
                        d.message,
                        d.code
                    );
                    for fix in &d.quick_fixes {
                        println!("    suggested fix: {} -> '{}'", fix.title, fix.new_text);
                    }
                }
            }

            if has_errors {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }

        Commands::Cfg {
            file,
            func,
            format,
            output,
        } => {
            let content = match fs::read_to_string(&file) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error reading file '{:?}': {}", file, e);
                    return ExitCode::FAILURE;
                }
            };

            let viz_format: VizFormat = match format.parse() {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("{}", e);
                    return ExitCode::FAILURE;
                }
            };

            let llvm_ver = detect_version(Some(&file), &content, None);
            let mut parser = LlvmParser::new();
            let doc = Document::new(&content);
            let tree = match parser.parse(&content, None) {
                Some(t) => t,
                None => {
                    eprintln!("Failed to parse file '{:?}'", file);
                    return ExitCode::FAILURE;
                }
            };

            let res = analyze(&doc, &tree, llvm_ver);

            // Find target function CFG
            let cfg = if let Some(ref fn_name) = func {
                let name = if fn_name.starts_with('@') {
                    fn_name.clone()
                } else {
                    format!("@{}", fn_name)
                };
                match res.cfgs.get(&name) {
                    Some(c) => c,
                    None => {
                        eprintln!(
                            "Function '{}' not found in '{:?}'. Available functions: {:?}",
                            fn_name,
                            file,
                            res.cfgs.keys().collect::<Vec<_>>()
                        );
                        return ExitCode::FAILURE;
                    }
                }
            } else {
                match res.cfgs.values().next() {
                    Some(c) => c,
                    None => {
                        eprintln!("No function definitions found in '{:?}'", file);
                        return ExitCode::FAILURE;
                    }
                }
            };

            let exported = export_cfg(cfg, viz_format);

            if let Some(out_path) = output {
                if let Err(e) = fs::write(&out_path, &exported) {
                    eprintln!("Error writing CFG output to '{:?}': {}", out_path, e);
                    return ExitCode::FAILURE;
                }
                println!("Wrote CFG to '{}'", out_path.display());
            } else {
                println!("{}", exported);
            }

            ExitCode::SUCCESS
        }

        Commands::Playground { file, port } => {
            playground::run_playground(file, port).await
        }
    }
}

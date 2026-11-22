use crate::diagnostic::AnalysisDiagnostic;
use std::path::Path;
use std::process::Command;
use tower_lsp::lsp_types::{DiagnosticSeverity, Position, Range};

/// Runs `llvm-as` (or `opt -passes=verify -disable-output`) on the given file path
/// and returns parsed LSP diagnostics.
pub fn verify_with_external_tool(
    file_path: &Path,
    llvm_as_bin: Option<&str>,
) -> Vec<AnalysisDiagnostic> {
    let bin = llvm_as_bin.unwrap_or("llvm-as");
    let output = match Command::new(bin)
        .arg("-o")
        .arg("/dev/null")
        .arg(file_path)
        .output()
    {
        Ok(out) => out,
        Err(_) => return Vec::new(),
    };

    if output.status.success() {
        return Vec::new();
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    parse_llvm_tool_stderr(&stderr)
}

pub fn parse_llvm_tool_stderr(stderr: &str) -> Vec<AnalysisDiagnostic> {
    let mut diags = Vec::new();

    // Example output:
    // llvm-as: test.ll:12:5: error: use of undefined value '%bar'
    for line in stderr.lines() {
        if let Some(error_idx) = line.find(": error: ") {
            let prefix = &line[..error_idx];
            let message = line[error_idx + ": error: ".len()..].trim();

            let parts: Vec<&str> = prefix.split(':').collect();
            let (line_num, col_num) = if parts.len() >= 3 {
                let l = parts[parts.len() - 2].parse::<u32>().unwrap_or(1);
                let c = parts[parts.len() - 1].parse::<u32>().unwrap_or(1);
                (l.saturating_sub(1), c.saturating_sub(1))
            } else {
                (0, 0)
            };

            let range = Range {
                start: Position {
                    line: line_num,
                    character: col_num,
                },
                end: Position {
                    line: line_num,
                    character: col_num + 1,
                },
            };

            diags.push(AnalysisDiagnostic::new(
                "LLVM-VERIFY-001",
                format!("llvm-as verifier: {}", message),
                range,
                DiagnosticSeverity::ERROR,
            ));
        }
    }

    diags
}

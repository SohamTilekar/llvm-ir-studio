pub mod analyzer;
pub mod cfg;
pub mod diagnostic;
pub mod symbol;
pub mod verifier;

pub use analyzer::{analyze, AnalysisResult};
pub use cfg::{BasicBlock, FunctionCfg, TerminatorKind};
pub use diagnostic::{AnalysisDiagnostic, QuickFix};
pub use symbol::{contains_position, FunctionSymbols, Symbol, SymbolKind, SymbolTable};
pub use verifier::verify_with_external_tool;

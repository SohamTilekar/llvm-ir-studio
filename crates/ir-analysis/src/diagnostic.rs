use serde::{Deserialize, Serialize};
use tower_lsp::lsp_types::{
    Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, Location, NumberOrString, Range,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickFix {
    pub title: String,
    pub range: Range,
    pub new_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisDiagnostic {
    pub code: String,
    pub message: String,
    pub range: Range,
    pub severity: DiagnosticSeverity,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub related_information: Vec<DiagnosticRelatedInformation>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub quick_fixes: Vec<QuickFix>,
}

impl AnalysisDiagnostic {
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        range: Range,
        severity: DiagnosticSeverity,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            range,
            severity,
            related_information: Vec::new(),
            quick_fixes: Vec::new(),
        }
    }

    pub fn with_related(mut self, message: impl Into<String>, location: Location) -> Self {
        self.related_information.push(DiagnosticRelatedInformation {
            location,
            message: message.into(),
        });
        self
    }

    pub fn with_quick_fix(
        mut self,
        title: impl Into<String>,
        range: Range,
        new_text: impl Into<String>,
    ) -> Self {
        self.quick_fixes.push(QuickFix {
            title: title.into(),
            range,
            new_text: new_text.into(),
        });
        self
    }

    pub fn to_lsp(&self) -> Diagnostic {
        Diagnostic {
            range: self.range,
            severity: Some(self.severity),
            code: Some(NumberOrString::String(self.code.clone())),
            code_description: None,
            source: Some("llvm-ir-ls".to_string()),
            message: self.message.clone(),
            related_information: if self.related_information.is_empty() {
                None
            } else {
                Some(self.related_information.clone())
            },
            tags: None,
            data: if self.quick_fixes.is_empty() {
                None
            } else {
                serde_json::to_value(&self.quick_fixes).ok()
            },
        }
    }
}

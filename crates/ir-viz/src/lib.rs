pub mod dot;
pub mod html;
pub mod mermaid;

pub use dot::to_dot;
pub use html::to_html;
pub use mermaid::to_mermaid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VizFormat {
    Mermaid,
    Dot,
    Html,
}

impl std::str::FromStr for VizFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mermaid" | "mmd" => Ok(VizFormat::Mermaid),
            "dot" | "graphviz" => Ok(VizFormat::Dot),
            "html" | "svg" => Ok(VizFormat::Html),
            other => Err(format!("Unknown viz format: '{}'. Choose mermaid, dot, or html.", other)),
        }
    }
}

pub fn export_cfg(cfg: &ir_analysis::FunctionCfg, format: VizFormat) -> String {
    match format {
        VizFormat::Mermaid => to_mermaid(cfg),
        VizFormat::Dot => to_dot(cfg),
        VizFormat::Html => to_html(cfg),
    }
}

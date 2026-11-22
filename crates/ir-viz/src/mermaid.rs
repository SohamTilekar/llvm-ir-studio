use ir_analysis::{FunctionCfg, TerminatorKind};

pub fn to_mermaid(cfg: &FunctionCfg) -> String {
    let mut out = String::new();
    out.push_str("flowchart TD\n");

    // Add nodes
    for name in &cfg.block_order {
        if let Some(bb) = cfg.blocks.get(name) {
            let node_id = sanitize_id(name);
            let mut label = format!("<b>{}:</b>", html_escape(name));
            for inst in &bb.instructions {
                label.push_str("<br/>");
                label.push_str(&html_escape(inst));
            }
            out.push_str(&format!("  {}[\"{}\"]\n", node_id, label));
        }
    }

    // Add edges
    for name in &cfg.block_order {
        if let Some(bb) = cfg.blocks.get(name) {
            let from_id = sanitize_id(name);
            match &bb.terminator {
                Some(TerminatorKind::CondBranch { true_label, false_label, .. }) => {
                    let true_id = sanitize_id(true_label);
                    let false_id = sanitize_id(false_label);
                    out.push_str(&format!("  {} -->|true| {}\n", from_id, true_id));
                    out.push_str(&format!("  {} -->|false| {}\n", from_id, false_id));
                }
                Some(TerminatorKind::Branch) => {
                    for succ in &bb.successors {
                        out.push_str(&format!("  {} --> {}\n", from_id, sanitize_id(succ)));
                    }
                }
                Some(TerminatorKind::Switch { cases, default_label, .. }) => {
                    out.push_str(&format!("  {} -->|default| {}\n", from_id, sanitize_id(default_label)));
                    for (val, dest) in cases {
                        let label = if val.is_empty() { "case".to_string() } else { format!("case {}", val) };
                        out.push_str(&format!("  {} -->|{}| {}\n", from_id, label, sanitize_id(dest)));
                    }
                }
                Some(TerminatorKind::Return) => {
                    let ret_id = format!("{}_ret", from_id);
                    out.push_str(&format!("  {}([Return])\n", ret_id));
                    out.push_str(&format!("  {} --> {}\n", from_id, ret_id));
                }
                Some(TerminatorKind::Unreachable) => {
                    let unreach_id = format!("{}_unreach", from_id);
                    out.push_str(&format!("  {}([Unreachable])\n", unreach_id));
                    out.push_str(&format!("  {} --> {}\n", from_id, unreach_id));
                }
                _ => {
                    for succ in &bb.successors {
                        out.push_str(&format!("  {} --> {}\n", from_id, sanitize_id(succ)));
                    }
                }
            }
        }
    }

    out
}

fn sanitize_id(name: &str) -> String {
    let clean: String = name.chars().map(|c| if c.is_alphanumeric() { c } else { '_' }).collect();
    if clean.is_empty() || clean.starts_with(|c: char| c.is_ascii_digit()) {
        format!("bb_{}", clean)
    } else {
        clean
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

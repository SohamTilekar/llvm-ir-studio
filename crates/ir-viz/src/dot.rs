use ir_analysis::{FunctionCfg, TerminatorKind};

pub fn to_dot(cfg: &FunctionCfg) -> String {
    let mut out = String::new();
    out.push_str(&format!("digraph \"{}\" {{\n", cfg.function_name));
    out.push_str("  node [shape=box, fontname=\"Courier\", style=\"rounded,filled\", fillcolor=\"#f0f4f8\", color=\"#334155\"];\n");
    out.push_str("  edge [fontname=\"Helvetica\", fontsize=10, color=\"#475569\"];\n\n");

    for name in &cfg.block_order {
        if let Some(bb) = cfg.blocks.get(name) {
            let mut label = format!("{}:\\l", name);
            for inst in &bb.instructions {
                label.push_str(&format!("  {}\\l", escape_dot(inst)));
            }
            out.push_str(&format!("  \"{}\" [label=\"{}\"];\n", name, label));
        }
    }

    out.push('\n');

    for name in &cfg.block_order {
        if let Some(bb) = cfg.blocks.get(name) {
            match &bb.terminator {
                Some(TerminatorKind::CondBranch { true_label, false_label, .. }) => {
                    out.push_str(&format!("  \"{}\" -> \"{}\" [label=\"true\", color=\"#16a34a\"];\n", name, true_label));
                    out.push_str(&format!("  \"{}\" -> \"{}\" [label=\"false\", color=\"#dc2626\"];\n", name, false_label));
                }
                Some(TerminatorKind::Branch) => {
                    for succ in &bb.successors {
                        out.push_str(&format!("  \"{}\" -> \"{}\";\n", name, succ));
                    }
                }
                Some(TerminatorKind::Switch { default_label, cases, .. }) => {
                    out.push_str(&format!("  \"{}\" -> \"{}\" [label=\"default\"];\n", name, default_label));
                    for (val, dest) in cases {
                        out.push_str(&format!("  \"{}\" -> \"{}\" [label=\"{}\"];\n", name, dest, val));
                    }
                }
                _ => {
                    for succ in &bb.successors {
                        out.push_str(&format!("  \"{}\" -> \"{}\";\n", name, succ));
                    }
                }
            }
        }
    }

    out.push_str("}\n");
    out
}

fn escape_dot(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\l")
}

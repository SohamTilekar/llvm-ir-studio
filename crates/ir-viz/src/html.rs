use crate::mermaid::to_mermaid;
use ir_analysis::FunctionCfg;

pub fn to_html(cfg: &FunctionCfg) -> String {
    let mermaid_graph = to_mermaid(cfg);

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>CFG - {}</title>
  <style>
    body {{
      margin: 0;
      padding: 20px;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
      background: #0f172a;
      color: #f8fafc;
    }}
    .header {{
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 20px;
      padding-bottom: 12px;
      border-bottom: 1px solid #334155;
    }}
    h1 {{
      margin: 0;
      font-size: 1.5rem;
      font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
      color: #38bdf8;
    }}
    .badge {{
      background: #1e293b;
      padding: 4px 10px;
      border-radius: 6px;
      font-size: 0.85rem;
      border: 1px solid #475569;
    }}
    #graph-container {{
      background: #1e293b;
      border: 1px solid #334155;
      border-radius: 8px;
      padding: 24px;
      overflow: auto;
      min-height: 500px;
      display: flex;
      justify-content: center;
      align-items: center;
    }}
    .mermaid {{
      font-family: ui-monospace, monospace;
    }}
  </style>
  <script type="module">
    import mermaid from 'https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.esm.min.mjs';
    mermaid.initialize({{
      startOnLoad: true,
      theme: 'dark',
      securityLevel: 'loose',
      flowchart: {{
        curve: 'basis',
        nodeSpacing: 50,
        rankSpacing: 50
      }}
    }});
  </script>
</head>
<body>
  <div class="header">
    <h1>Function CFG: {}</h1>
    <span class="badge">LLVM IR Visualizer</span>
  </div>
  <div id="graph-container">
    <pre class="mermaid">
{}
    </pre>
  </div>
</body>
</html>
"#,
        cfg.function_name, cfg.function_name, mermaid_graph
    )
}

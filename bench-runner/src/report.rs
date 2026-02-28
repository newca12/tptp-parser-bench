use crate::runner::BenchResult;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;

/// Generate an interactive HTML report with Chart.js from benchmark results.
pub fn generate_html_report(results: &[BenchResult], output_path: &Path) -> Result<()> {
    let results_json = serde_json::to_string(results)?;
    let html = build_html(&results_json, results);
    std::fs::write(output_path, html)?;
    Ok(())
}

/// Generate JSON compatible with `benchmark-action/github-action-benchmark`
/// using the `customSmallerIsBetter` tool format.
pub fn generate_github_action_json(results: &[BenchResult]) -> String {
    #[derive(serde::Serialize)]
    struct Entry {
        name: String,
        unit: String,
        value: f64,
        range: String,
        extra: String,
    }

    let entries: Vec<Entry> = results
        .iter()
        .filter(|r| r.success)
        .map(|r| Entry {
            name: format!("{} / {}", r.display_name, r.file_name),
            unit: "ns".to_string(),
            value: r.mean_ns,
            range: format!("± {:.0}", r.stddev_ns),
            extra: format!(
                "{} bytes, {:.2} MB/s, {} ({})",
                r.file_size_bytes, r.throughput_mb_s, r.language, r.technique
            ),
        })
        .collect();

    serde_json::to_string_pretty(&entries).unwrap_or_else(|_| "[]".to_string())
}

// ── HTML builder ────────────────────────────────────────────────────────────

fn build_html(results_json: &str, results: &[BenchResult]) -> String {
    // Collect unique parsers and files for the overview.
    let mut parsers: BTreeMap<String, (&str, &str, &str, &str)> = BTreeMap::new();
    for r in results {
        parsers
            .entry(r.parser_name.clone())
            .or_insert((&r.display_name, &r.language, &r.technique, &r.parser_name));
    }

    let parser_rows: String = parsers
        .values()
        .map(|(display, lang, tech, _name)| {
            format!(
                "          <tr><td>{}</td><td><span class=\"badge badge-{}\">{}</span></td><td>{}</td></tr>",
                display,
                lang.to_lowercase(),
                lang,
                tech,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Collect unique files.
    let files: Vec<String> = {
        let mut f: Vec<String> = results.iter().map(|r| r.file_name.clone()).collect();
        f.sort();
        f.dedup();
        f
    };

    let file_options: String = files
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let sel = if i == 0 { " selected" } else { "" };
            format!("            <option value=\"{}\"{}>{}</option>", f, sel, f)
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r##"<!DOCTYPE html>
<html lang="en" data-theme="dark">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>🏟️ TPTP Parser Arena</title>
  <script src="https://cdn.jsdelivr.net/npm/chart.js@4"></script>
  <style>
    :root {{
      --bg: #0d1117;
      --surface: #161b22;
      --surface2: #21262d;
      --border: #30363d;
      --text: #c9d1d9;
      --text-muted: #8b949e;
      --accent: #58a6ff;
      --green: #3fb950;
      --red: #f85149;
      --orange: #d29922;
    }}
    * {{ margin: 0; padding: 0; box-sizing: border-box; }}
    body {{
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif;
      background: var(--bg);
      color: var(--text);
      line-height: 1.6;
      padding: 2rem;
      max-width: 1200px;
      margin: 0 auto;
    }}
    h1 {{ font-size: 2rem; margin-bottom: 0.25rem; }}
    h2 {{ font-size: 1.4rem; margin: 2rem 0 1rem; color: var(--accent); }}
    h3 {{ font-size: 1.1rem; margin: 1.5rem 0 0.75rem; color: var(--text-muted); }}
    p.subtitle {{ color: var(--text-muted); margin-bottom: 2rem; }}
    a {{ color: var(--accent); text-decoration: none; }}
    a:hover {{ text-decoration: underline; }}

    .card {{
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 1.5rem;
      margin-bottom: 1.5rem;
    }}
    table {{ width: 100%; border-collapse: collapse; }}
    th, td {{
      text-align: left;
      padding: 0.6rem 1rem;
      border-bottom: 1px solid var(--border);
    }}
    th {{ color: var(--text-muted); font-weight: 600; font-size: 0.85rem; text-transform: uppercase; }}
    tr:last-child td {{ border-bottom: none; }}

    .badge {{
      display: inline-block;
      padding: 2px 8px;
      border-radius: 12px;
      font-size: 0.8rem;
      font-weight: 600;
    }}
    .badge-rust {{ background: #b7410e22; color: #f97316; border: 1px solid #f9731644; }}
    .badge-scala {{ background: #dc322f22; color: #dc322f; border: 1px solid #dc322f44; }}
    .badge-python {{ background: #30699822; color: #ffd43b; border: 1px solid #ffd43b44; }}
    .badge-java {{ background: #f8981d22; color: #f8981d; border: 1px solid #f8981d44; }}
    .badge-c {{ background: #55555522; color: #a8b9cc; border: 1px solid #a8b9cc44; }}

    .chart-controls {{
      display: flex;
      gap: 1rem;
      align-items: center;
      margin-bottom: 1rem;
      flex-wrap: wrap;
    }}
    .chart-controls select {{
      background: var(--surface2);
      color: var(--text);
      border: 1px solid var(--border);
      border-radius: 6px;
      padding: 0.4rem 0.8rem;
      font-size: 0.9rem;
    }}
    .chart-container {{
      position: relative;
      height: 320px;
      width: 100%;
    }}
    .stats-grid {{
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
      gap: 1rem;
      margin-top: 1rem;
    }}
    .stat-card {{
      background: var(--surface2);
      border-radius: 8px;
      padding: 1rem;
      text-align: center;
    }}
    .stat-value {{ font-size: 1.6rem; font-weight: 700; color: var(--accent); }}
    .stat-label {{ font-size: 0.8rem; color: var(--text-muted); margin-top: 0.25rem; }}

    .success {{ color: var(--green); }}
    .failure {{ color: var(--red); }}

    footer {{
      margin-top: 3rem;
      padding-top: 1.5rem;
      border-top: 1px solid var(--border);
      color: var(--text-muted);
      font-size: 0.85rem;
      text-align: center;
    }}
  </style>
</head>
<body>
  <h1>🏟️ TPTP Parser Arena</h1>
  <p class="subtitle">Benchmarking results for TPTP parsers across languages and techniques</p>

  <!-- Parser Table -->
  <h2>Parsers</h2>
  <div class="card">
    <table>
      <thead>
        <tr><th>Parser</th><th>Language</th><th>Technique</th></tr>
      </thead>
      <tbody>
{parser_rows}
      </tbody>
    </table>
  </div>

  <!-- Parse Time Chart -->
  <h2>Parse Time Comparison</h2>
  <div class="card">
    <div class="chart-controls">
      <label for="fileSelect">Corpus file:</label>
      <select id="fileSelect" onchange="updateCharts()">
{file_options}
      </select>
    </div>
    <div class="chart-container">
      <canvas id="timeChart"></canvas>
    </div>
  </div>

  <!-- Throughput Chart -->
  <h2>Throughput (MB/s)</h2>
  <div class="card">
    <div class="chart-container">
      <canvas id="throughputChart"></canvas>
    </div>
  </div>

  <!-- Detailed Stats -->
  <h2>Detailed Results</h2>
  <div class="card">
    <table id="detailTable">
      <thead>
        <tr>
          <th>Parser</th>
          <th>File</th>
          <th>Mean</th>
          <th>Std Dev</th>
          <th>Throughput</th>
          <th>Status</th>
        </tr>
      </thead>
      <tbody id="detailBody"></tbody>
    </table>
  </div>

  <footer>
    Generated by <a href="https://github.com/newca12/tptp-parser-bench">tptp-parser-bench</a>
  </footer>

  <script>
    // ── Data ──────────────────────────────────────────────────────────────
    const RAW = {results_json};

    const COLORS = [
      '#ff6384', '#36a2eb', '#4bc0c0', '#ff9f40',
      '#9966ff', '#ffcd56', '#c9cbcf', '#4dc9f6',
      '#f67019', '#00a8e8', '#e8006f', '#00c49a',
    ];

    // ── Derived structures ───────────────────────────────────────────────
    const parsers = [...new Set(RAW.map(r => r.display_name))];
    const files   = [...new Set(RAW.map(r => r.file_name))].sort();

    function colorFor(idx) {{ return COLORS[idx % COLORS.length]; }}

    function formatNs(ns) {{
      if (ns >= 1e9) return (ns / 1e9).toFixed(2) + ' s';
      if (ns >= 1e6) return (ns / 1e6).toFixed(2) + ' ms';
      if (ns >= 1e3) return (ns / 1e3).toFixed(2) + ' µs';
      return ns.toFixed(0) + ' ns';
    }}

    // ── Charts ───────────────────────────────────────────────────────────
    let timeChart, throughputChart;

    function buildTimeChart() {{
      const ctx = document.getElementById('timeChart').getContext('2d');
      timeChart = new Chart(ctx, {{
        type: 'bar',
        data: {{ labels: [], datasets: [] }},
        options: {{
          indexAxis: 'y',
          responsive: true,
          maintainAspectRatio: false,
          plugins: {{
            legend: {{ display: false }},
            tooltip: {{
              callbacks: {{
                label: (c) => c.dataset.label + ': ' + formatNs(c.raw)
              }}
            }}
          }},
          scales: {{
            x: {{
              title: {{ display: true, text: 'Parse time', color: '#8b949e' }},
              ticks: {{ color: '#8b949e', callback: (v) => formatNs(v) }},
              grid: {{ color: '#30363d' }}
            }},
            y: {{
              ticks: {{ color: '#c9d1d9' }},
              grid: {{ display: false }}
            }}
          }}
        }}
      }});
    }}

    function buildThroughputChart() {{
      const ctx = document.getElementById('throughputChart').getContext('2d');
      const datasets = parsers.map((p, i) => ({{
        label: p,
        data: files.map(f => {{
          const r = RAW.find(r => r.display_name === p && r.file_name === f);
          return r && r.success ? r.throughput_mb_s : 0;
        }}),
        backgroundColor: colorFor(i),
      }}));

      throughputChart = new Chart(ctx, {{
        type: 'bar',
        data: {{ labels: files, datasets }},
        options: {{
          responsive: true,
          maintainAspectRatio: false,
          plugins: {{
            legend: {{ labels: {{ color: '#c9d1d9' }} }},
            tooltip: {{
              callbacks: {{
                label: (c) => c.dataset.label + ': ' + c.raw.toFixed(2) + ' MB/s'
              }}
            }}
          }},
          scales: {{
            x: {{
              ticks: {{ color: '#8b949e' }},
              grid: {{ color: '#30363d' }}
            }},
            y: {{
              title: {{ display: true, text: 'MB/s', color: '#8b949e' }},
              ticks: {{ color: '#8b949e' }},
              grid: {{ color: '#30363d' }}
            }}
          }}
        }}
      }});
    }}

    function updateCharts() {{
      const file = document.getElementById('fileSelect').value;
      const filtered = RAW.filter(r => r.file_name === file && r.success);

      timeChart.data.labels = filtered.map(r => r.display_name);
      timeChart.data.datasets = [{{
        label: 'Mean parse time',
        data: filtered.map(r => r.mean_ns),
        backgroundColor: filtered.map((_, i) => colorFor(i)),
        borderRadius: 4,
      }}];
      timeChart.update();
    }}

    // ── Detail table ─────────────────────────────────────────────────────
    function populateDetailTable() {{
      const body = document.getElementById('detailBody');
      body.innerHTML = RAW.map(r => `
        <tr>
          <td>${{r.display_name}}</td>
          <td>${{r.file_name}}</td>
          <td>${{r.success ? formatNs(r.mean_ns) : '—'}}</td>
          <td>${{r.success ? '± ' + formatNs(r.stddev_ns) : '—'}}</td>
          <td>${{r.success ? r.throughput_mb_s.toFixed(2) + ' MB/s' : '—'}}</td>
          <td class="${{r.success ? 'success' : 'failure'}}">${{r.success ? '✓' : '✗ ' + (r.error || '')}}</td>
        </tr>
      `).join('');
    }}

    // ── Init ─────────────────────────────────────────────────────────────
    buildTimeChart();
    buildThroughputChart();
    updateCharts();
    populateDetailTable();
  </script>
</body>
</html>"##,
        parser_rows = parser_rows,
        file_options = file_options,
        results_json = results_json,
    )
}

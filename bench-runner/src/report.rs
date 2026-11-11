use crate::runner::BenchResult;
use std::collections::BTreeMap;
use std::path::Path;

/// Generate an interactive HTML report with Chart.js from benchmark results.
pub fn generate_html_report(results: &[BenchResult], output_path: &Path) -> crate::Result<()> {
    let results_json = escape_json_for_html(&serde_json::to_string(results)?);
    let html = build_html(&results_json, results);
    std::fs::write(output_path, html)?;
    Ok(())
}

fn escape_json_for_html(json: &str) -> String {
    json.replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
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
        parsers.entry(r.parser_name.clone()).or_insert((
            &r.display_name,
            &r.language,
            &r.technique,
            &r.parser_name,
        ));
    }

    let parser_rows: String = parsers
        .values()
        .map(|(display, lang, tech, _name)| {
            format!(
                "          <tr><td>{}</td><td><span class=\"badge badge-{}\">{}</span></td><td>{}</td></tr>",
                escape_html(display),
                escape_html(&lang.to_lowercase()),
                escape_html(lang),
                escape_html(tech),
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
            let escaped_file = escape_html(f);
            format!(
                "            <option value=\"{}\"{}>{}</option>",
                escaped_file, sel, escaped_file
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>TPTP Parser Arena</title>
  <script src="https://cdn.jsdelivr.net/npm/chart.js@4"></script>
  <style>
    :root {{
      --bg: #ffffff;
      --surface: #f6f8fa;
      --surface2: #eef1f5;
      --border: #d1d9e0;
      --text: #1f2328;
      --text-muted: #636c76;
      --accent: #0969da;
      --green: #1a7f37;
      --red: #cf222e;
      --gold: #bf8700;
      --gold-bg: #fff8e1;
      --gold-border: #f5c518;
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
    .badge-rust {{ background: #fff0e6; color: #b7410e; border: 1px solid #f9731644; }}
    .badge-scala {{ background: #fde8e8; color: #cf222e; border: 1px solid #dc322f44; }}
    .badge-python {{ background: #fef9e7; color: #7d6608; border: 1px solid #f5c51844; }}
    .badge-java {{ background: #fef3e2; color: #b45309; border: 1px solid #f8981d44; }}
    .badge-c {{ background: #f0f0f0; color: #555555; border: 1px solid #99999944; }}

    .chart-controls {{
      display: flex;
      gap: 1rem;
      align-items: center;
      margin-bottom: 1rem;
      flex-wrap: wrap;
    }}
    .chart-controls select, .chart-controls label {{
      font-size: 0.9rem;
    }}
    .chart-controls select {{
      background: var(--bg);
      color: var(--text);
      border: 1px solid var(--border);
      border-radius: 6px;
      padding: 0.4rem 0.8rem;
    }}
    .chart-controls input[type="checkbox"] {{ margin-right: 0.3rem; }}

    .chart-container {{
      position: relative;
      height: 320px;
      width: 100%;
    }}

    .success {{ color: var(--green); }}
    .failure {{ color: var(--red); }}

    tr.best-row {{
      background: var(--gold-bg);
    }}
    tr.best-row td:first-child {{
      border-left: 3px solid var(--gold-border);
      font-weight: 700;
    }}
    .trophy {{ margin-right: 0.3rem; }}

    .file-header td {{
      background: var(--surface2);
      font-weight: 600;
      color: var(--text-muted);
      font-size: 0.85rem;
      text-transform: uppercase;
      letter-spacing: 0.03em;
      padding: 0.5rem 1rem;
    }}

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
  <h1>TPTP Parser Arena</h1>
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
      <label><input type="checkbox" id="logScale" checked onchange="updateCharts()"> Log scale</label>
    </div>
    <div class="chart-container">
      <canvas id="timeChart"></canvas>
    </div>
  </div>

  <!-- Throughput Chart -->
  <h2>Throughput</h2>
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
          <th>Rank</th>
          <th>Parser</th>
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
    Generated by <a href="https://github.com/newca12/tptp-parser-bench">TPTP Parser Arena</a>
  </footer>

  <script>
    // ── Data ──────────────────────────────────────────────────────────────
    const RAW = {results_json};

    const COLORS = [
      '#4e79a7', '#f28e2b', '#e15759', '#76b7b2',
      '#59a14f', '#edc948', '#b07aa1', '#ff9da7',
      '#9c755f', '#bab0ac',
    ];
    const BEST_COLOR = '#2da44e';
    const BEST_BORDER = '#1a7f37';
    const TROPHY = String.fromCodePoint(0x1F3C6);
    const GOLD   = String.fromCodePoint(0x1F947);
    const SILVER = String.fromCodePoint(0x1F948);
    const BRONZE = String.fromCodePoint(0x1F949);
    const CHECK  = String.fromCodePoint(0x2713);
    const CROSS  = String.fromCodePoint(0x2717);
    const PM     = String.fromCodePoint(0x00B1);

    function escapeHtml(value) {{
      return String(value).replace(/[&<>"']/g, ch => ({{
        '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;'
      }})[ch]);
    }}

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

    function formatThroughput(mb_s) {{
      if (mb_s >= 0.1) return mb_s.toFixed(2) + ' MB/s';
      return (mb_s * 1000).toFixed(2) + ' KB/s';
    }}

    // Find the best (lowest mean_ns) parser for a given file
    function bestForFile(file) {{
      const forFile = RAW.filter(r => r.file_name === file && r.success);
      if (forFile.length === 0) return null;
      return forFile.reduce((a, b) => a.mean_ns < b.mean_ns ? a : b);
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
              type: 'logarithmic',
              title: {{ display: true, text: 'Parse time (log scale)', color: '#636c76' }},
              ticks: {{ color: '#636c76', callback: (v) => formatNs(v) }},
              grid: {{ color: '#e8e8e8' }}
            }},
            y: {{
              ticks: {{ color: '#1f2328' }},
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
            legend: {{ labels: {{ color: '#1f2328' }} }},
            tooltip: {{
              callbacks: {{
                label: (c) => c.dataset.label + ': ' + formatThroughput(c.raw)
              }}
            }}
          }},
          scales: {{
            x: {{
              ticks: {{ color: '#636c76' }},
              grid: {{ color: '#e8e8e8' }}
            }},
            y: {{
              type: 'logarithmic',
              title: {{ display: true, text: 'Throughput (log scale)', color: '#636c76' }},
              ticks: {{ color: '#636c76', callback: (v) => formatThroughput(v) }},
              grid: {{ color: '#e8e8e8' }}
            }}
          }}
        }}
      }});
    }}

    function updateCharts() {{
      const file = document.getElementById('fileSelect').value;
      const useLog = document.getElementById('logScale').checked;
      const filtered = RAW.filter(r => r.file_name === file && r.success);

      const best = bestForFile(file);
      const bgColors = filtered.map((r, i) =>
        best && r.display_name === best.display_name ? BEST_COLOR : colorFor(i)
      );
      const borderColors = filtered.map((r) =>
        best && r.display_name === best.display_name ? BEST_BORDER : 'transparent'
      );

      timeChart.data.labels = filtered.map(r =>
        best && r.display_name === best.display_name ? TROPHY + ' ' + r.display_name : r.display_name
      );
      timeChart.data.datasets = [{{
        label: 'Mean parse time',
        data: filtered.map(r => r.mean_ns),
        backgroundColor: bgColors,
        borderColor: borderColors,
        borderWidth: filtered.map(r => best && r.display_name === best.display_name ? 2 : 0),
        borderRadius: 4,
      }}];

      timeChart.options.scales.x.type = useLog ? 'logarithmic' : 'linear';
      timeChart.options.scales.x.title.text = useLog ? 'Parse time (log scale)' : 'Parse time';
      timeChart.update();
    }}

    // ── Detail table ─────────────────────────────────────────────────────
    function populateDetailTable() {{
      const body = document.getElementById('detailBody');
      let html = '';

      for (const file of files) {{
        const forFile = RAW.filter(r => r.file_name === file);
        const successful = forFile.filter(r => r.success).sort((a, b) => a.mean_ns - b.mean_ns);
        const failed = forFile.filter(r => !r.success);
        const ranked = [...successful, ...failed];
        const bestName = successful.length > 0 ? successful[0].display_name : null;

        html += `<tr class="file-header"><td colspan="6">${{escapeHtml(file)}}</td></tr>`;

        ranked.forEach((r, idx) => {{
          const isBest = r.success && r.display_name === bestName;
          const rank = r.success ? idx + 1 : 0;
          const medal = rank === 1 ? GOLD : rank === 2 ? SILVER : rank === 3 ? BRONZE : rank || '—';
          html += `
            <tr class="${{isBest ? 'best-row' : ''}}">
              <td>${{medal}}</td>
              <td>${{escapeHtml(r.display_name)}}</td>
              <td>${{r.success ? formatNs(r.mean_ns) : '—'}}</td>
              <td>${{r.success ? PM + ' ' + formatNs(r.stddev_ns) : '—'}}</td>
              <td>${{r.success ? formatThroughput(r.throughput_mb_s) : '—'}}</td>
              <td class="${{r.success ? 'success' : 'failure'}}">${{r.success ? CHECK : CROSS + ' ' + escapeHtml(r.error || '')}}</td>
            </tr>`;
        }});
      }}

      body.innerHTML = html;
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

fn escape_html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::{escape_html, escape_json_for_html};

    #[test]
    fn escapes_html_text_and_attribute_characters() {
        assert_eq!(
            escape_html("<script a=\"b\">&'"),
            "&lt;script a=&quot;b&quot;&gt;&amp;&#39;"
        );
    }

    #[test]
    fn prevents_json_from_terminating_the_html_script_element() {
        let json = serde_json::to_string("</script><script>alert(1)</script>").unwrap();
        let escaped = escape_json_for_html(&json);
        assert!(!escaped.contains("</script>"));
        assert!(escaped.contains("\\u003c/script\\u003e"));
    }
}

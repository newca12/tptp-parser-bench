# 🏟️ TPTP Parser Arena

A benchmarking arena for comparing TPTP parser implementations across
languages and techniques.

Built in Rust with [Criterion](https://crates.io/crates/criterion) for
precise micro-benchmarks and a custom runner for cross-language comparison.
Results are published to GitHub Pages as interactive Chart.js dashboards.

## Parsers

| Parser | Language | Technique | Repository |
|--------|----------|-----------|------------|
| **tptp** (nom) | Rust | nom (zero-copy, streaming) | [MichaelRawson/tptp](https://github.com/MichaelRawson/tptp) |
| **mrs-tptp** (winnow) | Rust | winnow parser combinators | [newca12/mrs-tptp-parser](https://github.com/newca12/mrs-tptp-parser) |
| **tptp** (ANTLR4) | Rust | ANTLR4 generated parser | [newca12/playground-rs](https://github.com/newca12/playground-rs/tree/main/tptp) |

## Quick Start

```bash
# Build everything
cargo build --release

# Run Criterion benchmarks (Rust parsers only, statistically rigorous)
cargo bench -p bench-native

# Run cross-parser benchmarks and generate HTML report
cargo run --release -p bench-runner

# Open the report
open results/report.html    # macOS
xdg-open results/report.html  # Linux
```

## Project Structure

```
tptp-parser-bench/
├── Cargo.toml              # Workspace root
├── parsers.toml            # Parser configuration
├── corpus/                 # TPTP test files (.p)
│   ├── SYN000-1.p          # CNF syntax showcase
│   ├── SYN000+1.p          # FOF syntax showcase
│   ├── SYN000_1.p          # TFF syntax showcase
│   └── SYN000=2.p          # TFA (arithmetic) syntax showcase
├── bench-native/           # Criterion micro-benchmarks
│   └── benches/
│       └── tptp_parsers.rs # Rust parser benchmarks
├── bench-runner/           # Cross-language benchmark orchestrator
│   └── src/
│       ├── main.rs         # CLI entry point
│       ├── config.rs       # TOML config parser
│       ├── runner.rs       # Process-based benchmark engine
│       └── report.rs       # HTML + JSON report generator
├── parsers/
│   ├── tptp-nom-wrapper/   # CLI wrapper for MichaelRawson/tptp
│   ├── mrs-tptp-wrapper/   # CLI wrapper for mrs-tptp (winnow)
│   └── tptp-antlr-wrapper/ # CLI wrapper for ANTLR4 parser
├── results/                # Generated reports (git-ignored)
└── .github/workflows/
    └── bench.yml           # CI: run benchmarks → GitHub Pages
```

## Adding a New Parser

1. **Create a CLI** that takes a file path as argument (or reads stdin),
   parses it, and exits 0 on success / non-zero on failure.

2. **Add an entry** to `parsers.toml`:

   ```toml
   [[parsers]]
   name = "my-parser"
   display_name = "My Parser (Python)"
   language = "Python"
   technique = "lark grammar"
   repository = "https://github.com/..."
   run_cmd = ["python3", "parsers/my-parser/parse.py"]
   input_mode = "file_arg"   # or "stdin"
   enabled = true
   ```

3. **Run benchmarks**:

   ```bash
   cargo run --release -p bench-runner
   ```

## Adding Corpus Files

Drop `.p` or `.tptp` files into the `corpus/` directory. The bench-runner
automatically discovers all files in that directory.

For stress testing with large files, download problems from the
[TPTP Problem Library](http://tptp.org).

## Benchmark Modes

### `bench-native` (Criterion)

Statistically rigorous micro-benchmarks for Rust-native parsers.
Uses [Criterion.rs](https://crates.io/crates/criterion) with automatic
outlier detection, confidence intervals, and throughput reporting.

Results: `target/criterion/` (HTML reports per benchmark group).

### `bench-runner` (Cross-language)

Process-based benchmarking that works with any parser exposed as a CLI.
Measures wall-clock time including process startup overhead.

Results: `results/report.html` (interactive Chart.js dashboard).

## Configuration

Edit `parsers.toml` to tune benchmark settings:

```toml
[settings]
iterations = 50           # Timed iterations per file
warmup_iterations = 5     # Warmup iterations (not timed)
timeout_secs = 30         # Per-invocation timeout
```

## GitHub Pages

On push to `main`, the CI workflow:

1. Builds all parsers
2. Runs Criterion and bench-runner benchmarks
3. Publishes time-series charts via
   [github-action-benchmark](https://github.com/benchmark-action/github-action-benchmark)
4. Deploys the HTML report to GitHub Pages

## License

MIT

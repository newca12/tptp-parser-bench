# 🏟️ TPTP Parser Arena

A benchmarking arena for comparing TPTP parser implementations across
languages and techniques.

Built with Rust 2024, [Criterion](https://crates.io/crates/criterion) for
precise micro-benchmarks and a custom runner for cross-language comparison.
Results are published to GitHub Pages as interactive Chart.js dashboards.

## Parsers

| Parser | Language | Technique | Repository |
|--------|----------|-----------|------------|
| **tptp** (nom) | Rust | nom (zero-copy, streaming) | [MichaelRawson/tptp](https://github.com/MichaelRawson/tptp) |
| **mrs-tptp** (winnow) | Rust | winnow parser combinators | [newca12/mrs](https://github.com/newca12/mrs) |
| **tptp** (ANTLR4) | Rust | ANTLR4 generated parser | [newca12/playground-rs](https://github.com/newca12/playground-rs/tree/main/tptp) |
| **Vampire TPTP** (hand-written) | C++ | hand-written recursive descent | [vprover/vampire](https://github.com/vprover/vampire) |
| **tptp** (ANTLR4, Java) | Java | ANTLR4 generated parser | [newca12/playground-rs](https://github.com/newca12/playground-rs/tree/main/tptp) |
| **scala-tptp-parser** (combinators) | Scala | Scala parser combinators | [leoprover/scala-tptp-parser](https://github.com/leoprover/scala-tptp-parser) |

## Quick Start

Clone the repository with its required optional-parser submodule:

```bash
git clone --recurse-submodules https://github.com/newca12/tptp-parser-bench.git
cd tptp-parser-bench
```

### With Nix

On NixOS or any system with the Nix flake command enabled, enter the project
environment with:

```bash
nix develop
```

The shell provides Rust, JDK 25, Maven, sbt, and the native build tools used by
the parser wrappers. Vampire is enabled by default, so its source submodule
must be initialized and its wrapper built before running the cross-parser
benchmark:

```bash
git submodule update --init --recursive
cargo build --release
cargo build --release --manifest-path parsers/vampire-tptp-wrapper/Cargo.toml --target-dir target -j 1
(cd parsers/java-antlr-wrapper && mvn -q package)
(cd parsers/scala-tptp-wrapper && sbt assembly)
cargo run --release -p bench-runner
```

### Without Nix

```bash
# Ensure the Vampire source submodule is initialized (required by default).
git submodule update --init --recursive

# Build Rust workspace tools and Rust parser wrappers
cargo build --release

# Build the Vampire parser wrapper (resource-intensive; use one job)
cargo build --release --manifest-path parsers/vampire-tptp-wrapper/Cargo.toml --target-dir target -j 1

# The JVM wrappers need JDK 11+ both to build and to run. Select JDK 25
# before building them (adjust JAVA_HOME for your installation path).
export JAVA_HOME=/usr/lib/jvm/java-25-openjdk-amd64
export PATH="$JAVA_HOME/bin:$PATH"
java -version

# Build Java ANTLR4 parser wrapper (requires JDK 11+, Maven)
cd parsers/java-antlr-wrapper && mvn -q package && cd ../..

# Build Scala parser wrapper (requires JDK 11+, sbt)
cd parsers/scala-tptp-wrapper && sbt assembly && cd ../..

# Run Criterion benchmarks (Rust parsers only, statistically rigorous)
cargo bench -p bench-native

# Run cross-parser benchmarks and generate the report
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
│   ├── SYN000=2.p          # TFA (arithmetic) syntax showcase
│   ├── Axioms/             # Include fixtures for SYN000-1.p and SYN000_1.p
│   └── examples/           # Include fixture for SYN000+1.p
├── bench-native/           # Criterion micro-benchmarks
│   └── benches/
│       └── tptp_parsers.rs # Rust parser benchmarks
├── bench-runner/           # Cross-language benchmark orchestrator
│   └── src/
│       ├── main.rs         # CLI entry point
│       ├── config.rs       # TOML config parser
│       ├── runner.rs       # Process-based benchmark engine
│       └── report.rs       # HTML + JSON report generator
├── check-correctness/      # Correctness checker (roundtrip + cross-validation)
│   └── src/
│       ├── main.rs         # CLI entry point
│       ├── runner.rs       # Parallel execution engine
│       ├── parsers.rs      # ParserBackend trait + subprocess output parser
│       └── parsers/        # Per-parser backend implementations
├── parsers/
│   ├── tptp-nom-wrapper/   # CLI wrapper for MichaelRawson/tptp
│   ├── mrs-tptp-wrapper/   # CLI wrapper for mrs-tptp (winnow)
│   ├── tptp-antlr-wrapper/ # CLI wrapper for ANTLR4 parser
│   ├── vampire-tptp-wrapper/ # CLI wrapper for Vampire's TPTP parser (C++)
│   ├── java-antlr-wrapper/  # CLI wrapper for ANTLR4 parser (Java)
│   └── scala-tptp-wrapper/ # CLI wrapper for scala-tptp-parser (JVM)
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
[TPTP Problem Library](https://tptp.org).

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
The runner exits non-zero if any enabled parser/file combination fails.

The Vampire C++ wrapper is enabled by default in `parsers.toml`. Its source is
included as a git submodule, and the wrapper is excluded from normal workspace
builds because compiling Vampire's native parser is resource-intensive. The
Quick Start initializes the submodule and builds the wrapper with one job. To
skip it locally, set `enabled = false` for `vampire-tptp` in `parsers.toml`.

```bash
git submodule update --init --recursive
cargo build --release --manifest-path parsers/vampire-tptp-wrapper/Cargo.toml --target-dir target -j 1
```

### `check-correctness` (Manual)

Correctness checker that recursively parses all `.p` files in a folder and
verifies results via roundtrip testing or cross-validation against the
mrs-tptp reference parser.

```bash
# Build
cargo build --release -p check-correctness

# Usage
./target/release/check-correctness <parser> <folder> [--timeout <ms>] [--threads <n>] [--verbose]
```

Available parsers: `mrs-tptp`, `tptp-nom`, `tptp-antlr`, `vampire-tptp`, `scala-tptp`, `java-antlr`.

Parsers with pretty-printing support (mrs-tptp, tptp-nom, scala-tptp) use
roundtrip verification: parse, pretty-print, reparse, compare. The ANTLR
parsers cross-validate formula and include counts against the mrs-tptp
reference. Vampire's internal unit count is not comparable to source-level
formula counts across TPTP dialects, so the checker reports parse success but
does not cross-validate its formula or include counts.

The subprocess-based parsers (vampire-tptp, scala-tptp, java-antlr) require
their wrappers to be built first (see Quick Start).
When `--timeout` is supplied, each file check runs in a worker process; on Unix,
the checker terminates the worker and its subprocesses when the limit is exceeded.

Example:

```bash
./target/release/check-correctness mrs-tptp corpus/
./target/release/check-correctness tptp-nom /path/to/TPTP/Problems --timeout 5000 --threads 4
```

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

1. Builds the Rust workspace, Vampire wrapper, and enabled parser wrappers
2. Runs Criterion and bench-runner benchmarks
3. Publishes time-series charts via
   [github-action-benchmark](https://github.com/benchmark-action/github-action-benchmark)
4. Deploys the HTML report to GitHub Pages

For the first deployment, enable GitHub Pages in the repository settings and
select **GitHub Actions** as the build and deployment source.

## License

The original benchmark harness is licensed under the MIT License; see
[`LICENSE`](LICENSE). This repository also uses separately licensed parser
components, including GPL-3.0 ANTLR parser sources and the optional BSD-3-Clause
Vampire submodule. See [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) before
redistributing components or their built executables.

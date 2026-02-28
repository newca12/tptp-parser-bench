# TPTP Benchmark Corpus

Standard TPTP syntax test problems from the [TPTP Problem Library](http://tptp.org).

| File | Dialect | Description |
|------|---------|-------------|
| `SYN000-1.p` | CNF | Clause normal form syntax showcase |
| `SYN000+1.p` | FOF | First-order formula syntax showcase |
| `SYN000_1.p` | TFF | Typed first-order formula syntax showcase |
| `SYN000=2.p` | TFA | TFF with arithmetic syntax showcase |

These files exercise all major syntactic constructs in each TPTP dialect.

## Adding More Files

To add larger files for stress testing, download problems from the
[TPTP Problem Library](http://www.tptp.org/cgi-bin/SeeTPTP?Category=Problems).

Place `.p` files in this directory — the bench-runner will automatically
pick them up.

For a large-file stress test, consider `CSR002+5.ax` (458 MB) from the
TPTP axiom library.

# Third-party components and licenses

This repository is an aggregate of separately licensed tools. The MIT license
in [`LICENSE`](LICENSE) applies only to original project code and does not
replace the licenses below.

## GPL-3.0 components

- The Rust ANTLR TPTP parser is fetched from
  [`newca12/playground-rs`](https://github.com/newca12/playground-rs) at commit
  `00428cd44348a3e05254e1834ea4a0ae3dba7a5f` (see the Cargo manifests and
  `Cargo.lock`). The `tptp-antlr-wrapper` and `check-correctness` packages use
  this parser.
- `parsers/java-antlr-wrapper/src/main/antlr4/tptp/TPTP.g4` is modified from
  the grammar in that same repository and commit; its generated Java parser
  is part of the Java wrapper build.

The GPL-3.0 terms are in [`LICENSES/GPL-3.0.txt`](LICENSES/GPL-3.0.txt). The
corresponding parser components and executables that combine them with other
code must be redistributed under terms compatible with GPL-3.0.

## Vampire

The optional `parsers/vampire-tptp-wrapper/vampire` submodule points to
[`vprover/vampire`](https://github.com/vprover/vampire), licensed under its
modified BSD 3-Clause license. Its license and notices are in the submodule's
`LICENCE` file; nested components may have their own terms.

## Corpus

The syntax showcase problems are attributed to the
[TPTP Problem Library](https://tptp.org); see [`corpus/README.md`](corpus/README.md).

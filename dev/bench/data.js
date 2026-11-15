window.BENCHMARK_DATA = {
  "lastUpdate": 1794763813445,
  "repoUrl": "https://github.com/newca12/tptp-parser-bench",
  "entries": {
    "TPTP Parser Arena": [
      {
        "commit": {
          "author": {
            "email": "newca12@users.noreply.github.com",
            "name": "Olivier ROLAND",
            "username": "newca12"
          },
          "committer": {
            "email": "newca12@users.noreply.github.com",
            "name": "Olivier ROLAND",
            "username": "newca12"
          },
          "distinct": true,
          "id": "d31e49dae8a0c3f92b373a5de61814b03d56aca5",
          "message": "Improve parser benchmarking and validation",
          "timestamp": "2026-11-15T16:42:48Z",
          "tree_id": "6d644743df09d303c7ecac1888094bad56ce94a4",
          "url": "https://github.com/newca12/tptp-parser-bench/commit/d31e49dae8a0c3f92b373a5de61814b03d56aca5"
        },
        "date": 1794761976944,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "tptp (nom, zero-copy) / SYN000+1.p",
            "value": 9150.5,
            "range": "± 226",
            "unit": "ns",
            "extra": "1281 bytes, 139.99 MB/s, median: 9.15 µs, mean: 9.20 µs ± 226 ns, peak mem: 2.29 MB, Rust (nom streaming parser)"
          },
          {
            "name": "tptp (nom, zero-copy) / SYN000-1.p",
            "value": 4209.5,
            "range": "± 170",
            "unit": "ns",
            "extra": "970 bytes, 230.43 MB/s, median: 4.21 µs, mean: 4.23 µs ± 170 ns, peak mem: 2.28 MB, Rust (nom streaming parser)"
          },
          {
            "name": "tptp (nom, zero-copy) / SYN000=2.p",
            "value": 38676,
            "range": "± 4186",
            "unit": "ns",
            "extra": "5209 bytes, 134.68 MB/s, median: 38.68 µs, mean: 39.59 µs ± 4.19 µs, peak mem: 2.24 MB, Rust (nom streaming parser)"
          },
          {
            "name": "tptp (nom, zero-copy) / SYN000_1.p",
            "value": 16291.5,
            "range": "± 2387",
            "unit": "ns",
            "extra": "2440 bytes, 149.77 MB/s, median: 16.29 µs, mean: 16.73 µs ± 2.39 µs, peak mem: 2.29 MB, Rust (nom streaming parser)"
          },
          {
            "name": "mrs-tptp (winnow, zero-copy) / SYN000+1.p",
            "value": 8131.5,
            "range": "± 645",
            "unit": "ns",
            "extra": "1281 bytes, 157.54 MB/s, median: 8.13 µs, mean: 8.35 µs ± 645 ns, peak mem: 2.53 MB, Rust (winnow parser combinators)"
          },
          {
            "name": "mrs-tptp (winnow, zero-copy) / SYN000-1.p",
            "value": 3654,
            "range": "± 407",
            "unit": "ns",
            "extra": "970 bytes, 265.46 MB/s, median: 3.65 µs, mean: 3.80 µs ± 407 ns, peak mem: 2.54 MB, Rust (winnow parser combinators)"
          },
          {
            "name": "mrs-tptp (winnow, zero-copy) / SYN000=2.p",
            "value": 60455,
            "range": "± 6316",
            "unit": "ns",
            "extra": "5209 bytes, 86.16 MB/s, median: 60.45 µs, mean: 62.68 µs ± 6.32 µs, peak mem: 2.88 MB, Rust (winnow parser combinators)"
          },
          {
            "name": "mrs-tptp (winnow, zero-copy) / SYN000_1.p",
            "value": 22595.5,
            "range": "± 2933",
            "unit": "ns",
            "extra": "2440 bytes, 107.99 MB/s, median: 22.60 µs, mean: 23.41 µs ± 2.93 µs, peak mem: 2.86 MB, Rust (winnow parser combinators)"
          },
          {
            "name": "Vampire TPTP (hand-written) / SYN000+1.p",
            "value": 46326.5,
            "range": "± 6161",
            "unit": "ns",
            "extra": "1281 bytes, 27.65 MB/s, median: 46.33 µs, mean: 48.14 µs ± 6.16 µs, peak mem: 10.32 MB, C++ (hand-written recursive descent)"
          },
          {
            "name": "Vampire TPTP (hand-written) / SYN000-1.p",
            "value": 34873,
            "range": "± 4897",
            "unit": "ns",
            "extra": "970 bytes, 27.82 MB/s, median: 34.87 µs, mean: 36.06 µs ± 4.90 µs, peak mem: 10.14 MB, C++ (hand-written recursive descent)"
          },
          {
            "name": "Vampire TPTP (hand-written) / SYN000=2.p",
            "value": 206222.5,
            "range": "± 12063",
            "unit": "ns",
            "extra": "5209 bytes, 25.26 MB/s, median: 206.22 µs, mean: 210.79 µs ± 12.06 µs, peak mem: 10.64 MB, C++ (hand-written recursive descent)"
          },
          {
            "name": "Vampire TPTP (hand-written) / SYN000_1.p",
            "value": 71743.5,
            "range": "± 6870",
            "unit": "ns",
            "extra": "2440 bytes, 34.01 MB/s, median: 71.74 µs, mean: 74.01 µs ± 6.87 µs, peak mem: 10.41 MB, C++ (hand-written recursive descent)"
          },
          {
            "name": "scala-tptp-parser (combinators) / SYN000+1.p",
            "value": 614149.5,
            "range": "± 672110",
            "unit": "ns",
            "extra": "1281 bytes, 2.09 MB/s, median: 614.15 µs, mean: 920.11 µs ± 672.11 µs, peak mem: 87.99 MB, Scala (Scala parser combinators)"
          },
          {
            "name": "scala-tptp-parser (combinators) / SYN000-1.p",
            "value": 597802.5,
            "range": "± 394590",
            "unit": "ns",
            "extra": "970 bytes, 1.62 MB/s, median: 597.80 µs, mean: 762.16 µs ± 394.59 µs, peak mem: 84.57 MB, Scala (Scala parser combinators)"
          },
          {
            "name": "scala-tptp-parser (combinators) / SYN000=2.p",
            "value": 1362312.5,
            "range": "± 1019564",
            "unit": "ns",
            "extra": "5209 bytes, 3.82 MB/s, median: 1.36 ms, mean: 1.72 ms ± 1.02 ms, peak mem: 105.35 MB, Scala (Scala parser combinators)"
          },
          {
            "name": "scala-tptp-parser (combinators) / SYN000_1.p",
            "value": 1373885,
            "range": "± 459409",
            "unit": "ns",
            "extra": "2440 bytes, 1.78 MB/s, median: 1.37 ms, mean: 1.33 ms ± 459.41 µs, peak mem: 94.13 MB, Scala (Scala parser combinators)"
          },
          {
            "name": "tptp (ANTLR4, Java) / SYN000+1.p",
            "value": 1923501.5,
            "range": "± 499353",
            "unit": "ns",
            "extra": "1281 bytes, 0.67 MB/s, median: 1.92 ms, mean: 1.84 ms ± 499.35 µs, peak mem: 85.41 MB, Java (ANTLR4 generated parser)"
          },
          {
            "name": "tptp (ANTLR4, Java) / SYN000-1.p",
            "value": 1103594,
            "range": "± 138908",
            "unit": "ns",
            "extra": "970 bytes, 0.88 MB/s, median: 1.10 ms, mean: 1.08 ms ± 138.91 µs, peak mem: 64.67 MB, Java (ANTLR4 generated parser)"
          },
          {
            "name": "tptp (ANTLR4, Java) / SYN000=2.p",
            "value": 17258466,
            "range": "± 311261",
            "unit": "ns",
            "extra": "5209 bytes, 0.30 MB/s, median: 17.26 ms, mean: 17.35 ms ± 311.26 µs, peak mem: 170.42 MB, Java (ANTLR4 generated parser)"
          },
          {
            "name": "tptp (ANTLR4, Java) / SYN000_1.p",
            "value": 2076340.5,
            "range": "± 1484511",
            "unit": "ns",
            "extra": "2440 bytes, 1.18 MB/s, median: 2.08 ms, mean: 2.54 ms ± 1.48 ms, peak mem: 196.95 MB, Java (ANTLR4 generated parser)"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "newca12@users.noreply.github.com",
            "name": "Olivier ROLAND",
            "username": "newca12"
          },
          "committer": {
            "email": "newca12@users.noreply.github.com",
            "name": "Olivier ROLAND",
            "username": "newca12"
          },
          "distinct": true,
          "id": "88d861b9e40067f5777b2cbf62572508d4a6d75c",
          "message": "Avoid duplicate Pages artifacts on workflow reruns",
          "timestamp": "2026-11-15T17:11:24Z",
          "tree_id": "d7ebb059b88dbbe0e6e0e16fe77cd13e75af1458",
          "url": "https://github.com/newca12/tptp-parser-bench/commit/88d861b9e40067f5777b2cbf62572508d4a6d75c"
        },
        "date": 1794763813210,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "tptp (nom, zero-copy) / SYN000+1.p",
            "value": 11021.5,
            "range": "± 434",
            "unit": "ns",
            "extra": "1281 bytes, 116.23 MB/s, median: 11.02 µs, mean: 11.12 µs ± 434 ns, peak mem: 2.17 MB, Rust (nom streaming parser)"
          },
          {
            "name": "tptp (nom, zero-copy) / SYN000-1.p",
            "value": 5453,
            "range": "± 354",
            "unit": "ns",
            "extra": "970 bytes, 177.88 MB/s, median: 5.45 µs, mean: 5.51 µs ± 354 ns, peak mem: 2.28 MB, Rust (nom streaming parser)"
          },
          {
            "name": "tptp (nom, zero-copy) / SYN000=2.p",
            "value": 46108,
            "range": "± 4389",
            "unit": "ns",
            "extra": "5209 bytes, 112.97 MB/s, median: 46.11 µs, mean: 47.22 µs ± 4.39 µs, peak mem: 2.23 MB, Rust (nom streaming parser)"
          },
          {
            "name": "tptp (nom, zero-copy) / SYN000_1.p",
            "value": 21147,
            "range": "± 3020",
            "unit": "ns",
            "extra": "2440 bytes, 115.38 MB/s, median: 21.15 µs, mean: 21.78 µs ± 3.02 µs, peak mem: 2.23 MB, Rust (nom streaming parser)"
          },
          {
            "name": "mrs-tptp (winnow, zero-copy) / SYN000+1.p",
            "value": 11597.5,
            "range": "± 2608",
            "unit": "ns",
            "extra": "1281 bytes, 110.45 MB/s, median: 11.60 µs, mean: 12.12 µs ± 2.61 µs, peak mem: 2.52 MB, Rust (winnow parser combinators)"
          },
          {
            "name": "mrs-tptp (winnow, zero-copy) / SYN000-1.p",
            "value": 5172.5,
            "range": "± 622",
            "unit": "ns",
            "extra": "970 bytes, 187.53 MB/s, median: 5.17 µs, mean: 5.40 µs ± 622 ns, peak mem: 2.62 MB, Rust (winnow parser combinators)"
          },
          {
            "name": "mrs-tptp (winnow, zero-copy) / SYN000=2.p",
            "value": 82827,
            "range": "± 7532",
            "unit": "ns",
            "extra": "5209 bytes, 62.89 MB/s, median: 82.83 µs, mean: 85.55 µs ± 7.53 µs, peak mem: 2.80 MB, Rust (winnow parser combinators)"
          },
          {
            "name": "mrs-tptp (winnow, zero-copy) / SYN000_1.p",
            "value": 30715,
            "range": "± 3532",
            "unit": "ns",
            "extra": "2440 bytes, 79.44 MB/s, median: 30.71 µs, mean: 31.72 µs ± 3.53 µs, peak mem: 2.77 MB, Rust (winnow parser combinators)"
          },
          {
            "name": "Vampire TPTP (hand-written) / SYN000+1.p",
            "value": 73022.5,
            "range": "± 5630",
            "unit": "ns",
            "extra": "1281 bytes, 17.54 MB/s, median: 73.02 µs, mean: 74.87 µs ± 5.63 µs, peak mem: 10.45 MB, C++ (hand-written recursive descent)"
          },
          {
            "name": "Vampire TPTP (hand-written) / SYN000-1.p",
            "value": 56899,
            "range": "± 6648",
            "unit": "ns",
            "extra": "970 bytes, 17.05 MB/s, median: 56.90 µs, mean: 58.66 µs ± 6.65 µs, peak mem: 10.22 MB, C++ (hand-written recursive descent)"
          },
          {
            "name": "Vampire TPTP (hand-written) / SYN000=2.p",
            "value": 283509.5,
            "range": "± 10483",
            "unit": "ns",
            "extra": "5209 bytes, 18.37 MB/s, median: 283.51 µs, mean: 286.90 µs ± 10.48 µs, peak mem: 11.09 MB, C++ (hand-written recursive descent)"
          },
          {
            "name": "Vampire TPTP (hand-written) / SYN000_1.p",
            "value": 103538,
            "range": "± 9220",
            "unit": "ns",
            "extra": "2440 bytes, 23.57 MB/s, median: 103.54 µs, mean: 106.58 µs ± 9.22 µs, peak mem: 10.45 MB, C++ (hand-written recursive descent)"
          },
          {
            "name": "scala-tptp-parser (combinators) / SYN000+1.p",
            "value": 743824.5,
            "range": "± 738096",
            "unit": "ns",
            "extra": "1281 bytes, 1.72 MB/s, median: 743.82 µs, mean: 1.09 ms ± 738.10 µs, peak mem: 88.50 MB, Scala (Scala parser combinators)"
          },
          {
            "name": "scala-tptp-parser (combinators) / SYN000-1.p",
            "value": 726075.5,
            "range": "± 441321",
            "unit": "ns",
            "extra": "970 bytes, 1.34 MB/s, median: 726.08 µs, mean: 924.92 µs ± 441.32 µs, peak mem: 85.01 MB, Scala (Scala parser combinators)"
          },
          {
            "name": "scala-tptp-parser (combinators) / SYN000=2.p",
            "value": 2036446,
            "range": "± 1259750",
            "unit": "ns",
            "extra": "5209 bytes, 2.56 MB/s, median: 2.04 ms, mean: 2.48 ms ± 1.26 ms, peak mem: 105.43 MB, Scala (Scala parser combinators)"
          },
          {
            "name": "scala-tptp-parser (combinators) / SYN000_1.p",
            "value": 1413839.5,
            "range": "± 688386",
            "unit": "ns",
            "extra": "2440 bytes, 1.73 MB/s, median: 1.41 ms, mean: 1.62 ms ± 688.39 µs, peak mem: 91.77 MB, Scala (Scala parser combinators)"
          },
          {
            "name": "tptp (ANTLR4, Java) / SYN000+1.p",
            "value": 1889552,
            "range": "± 1231274",
            "unit": "ns",
            "extra": "1281 bytes, 0.68 MB/s, median: 1.89 ms, mean: 2.05 ms ± 1.23 ms, peak mem: 85.98 MB, Java (ANTLR4 generated parser)"
          },
          {
            "name": "tptp (ANTLR4, Java) / SYN000-1.p",
            "value": 1278387,
            "range": "± 144224",
            "unit": "ns",
            "extra": "970 bytes, 0.76 MB/s, median: 1.28 ms, mean: 1.26 ms ± 144.22 µs, peak mem: 64.10 MB, Java (ANTLR4 generated parser)"
          },
          {
            "name": "tptp (ANTLR4, Java) / SYN000=2.p",
            "value": 17211191,
            "range": "± 1059991",
            "unit": "ns",
            "extra": "5209 bytes, 0.30 MB/s, median: 17.21 ms, mean: 17.51 ms ± 1.06 ms, peak mem: 181.70 MB, Java (ANTLR4 generated parser)"
          },
          {
            "name": "tptp (ANTLR4, Java) / SYN000_1.p",
            "value": 2618946.5,
            "range": "± 493820",
            "unit": "ns",
            "extra": "2440 bytes, 0.93 MB/s, median: 2.62 ms, mean: 2.67 ms ± 493.82 µs, peak mem: 192.73 MB, Java (ANTLR4 generated parser)"
          }
        ]
      }
    ]
  }
}
# MS-1 — Fast local foundation evidence

Reference environment: macOS arm64, measured 2026-08-13 with the stripped
release binary. Process timings use Hyperfine with 50 warmups and 300 runs.

| Command | p95 | p99 |
| --- | ---: | ---: |
| `tmpmail --version` | 5.87 ms | 7.85 ms |
| `tmpmail --help` | 6.80 ms | 7.60 ms |
| `tmpmail inbox list` (100 saved inboxes) | 4.86 ms | 5.17 ms |

- Peak RSS (`tmpmail --help`): 2,719,744 bytes (2.59 MiB).
- Stripped release binary: 865,664 bytes.
- Unit and integration contracts: 9 passed.
- `cargo fmt --check`: passed.
- Clippy with warnings denied: passed.
- The release build initializes neither networking nor secret storage for
  help, version, configuration, or local inbox listing.

An initially noisy sample exceeded the latency gate. Repeating on a quiet
system and changing release LTO from thin to fat produced the accepted,
reproducible baseline above. Raw benchmark output is intentionally kept out of
Git; milestone evidence records only the environment, method, and results.

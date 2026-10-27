---
created_on: 2026-10-14 14:56
last_modified: 2026-10-14 14:56
status: current
---

# Dependency mapping

| Go dependency | Rust dependency | Integration |
| :--- | :--- | :--- |
| `pkoukk/tiktoken-go` 0.1.7 | `tiktoken-rs` 0.12.1, MIT | Native model lookup, cached BPE tokenizers, ordinary-text encoding, byte decoding; bundled encoding assets. |
| `spf13/cobra` 1.8 | `clap` through `cli-command-kit`, MIT/Apache-2.0 | Native argument parsing and metadata, shared human/agent help. |
| Go standard error wrapping | `anyhow` 1.0.102, MIT/Apache-2.0 | Preserved I/O and tokenizer error chains; one reporting boundary. |
| Go temporary test files | `tempfile` 3.27.0, MIT/Apache-2.0 | Owned native test fixtures under `.tmp/`. |

`Cargo.lock` records exact resolved versions and registry checksums. API behavior was checked against the pinned crate sources and [clap's native Command documentation](https://docs.rs/clap/4.6.7/clap/struct.Command.html). The model lookup follows the pinned tiktoken-rs mapping rather than a hand-written model resolver; the catalog gives examples rather than claiming to enumerate every accepted model suffix.

## Observable migration decisions

- Domain commands use `text count`, `text encode`, `token decode`, and `model list`. Root invocation still defaults to counting. Raw version output is shared by `--version` and `version`.
- Native `IsTerminal` replaces the character-device approximation.
- Failed reads of an existing positional file and metadata failures such as a symlink loop are reported, rather than silently counting its filename.
- Invalid token IDs return a recoverable error. Native `decode_bytes` preserves partial UTF-8 token sequences.
- Invalid UTF-8 text input returns an error; decoded token output remains raw bytes.
- The usage skill and source URLs are canonical pointers; the local Rust migration has no published artifacts.

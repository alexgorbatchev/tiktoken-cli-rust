---
created_on: 2026-10-14 14:54
last_modified: 2026-10-14 14:54
status: current
---

# tiktoken

Independent Rust CLI for offline token counting, encoding, and byte-preserving decoding. Inherit the parent [AGENTS.md](../AGENTS.md).

## Commands

- Build: `just build` writes `bin/tiktoken`.
- Run: `just run text encode 'hello world!'`; agent mode: `just run-ai model list`.
- Test: `just test`.
- Lint: `just lint`; complete gate: `just check`.
- Coverage: `just coverage`, at least 90% lines.

## Contracts

- Use tiktoken-rs native model resolution, `encode_ordinary`, and `decode_bytes`. Preserve incomplete UTF-8 token bytes on stdout.
- Keep `cl100k_base` as the default; `--model` takes precedence and an unknown model returns an error.
- Explicit file input precedes existing positional files, inline input, and stdin. Remove exactly one final LF from file/stdin input; preserve inline newlines. Existing-file failures must remain errors.
- Root terminal invocation without input prints help. Explicit domain commands without terminal input return an error. Detect terminal state through `IsTerminal`.
- `entry` is the error-reporting boundary; print each failure once, preserving output-write failures. Use cli-command-kit for native clap help and home folding.
- Update `skills/tiktoken/SKILL.md` and its `last_modified` in the same change as any command, argument, flag, default, environment, output, or side-effect change. Test the live command metadata with `skill_omissions`.
- The canonical source skill URL does not establish publication of this Rust migration. Do not advertise existing Go releases as Rust binaries.

## Boundaries

- Runtime changes require a corresponding behavioral test-file change and at least 90% line coverage; scripts are excluded. Prove regressions with a disabled implementation and restore it.
- Scope formatters to this package's files; path dependencies are independent repositories.
- Keep temporary evidence under `.tmp/`. Persist standing rules here and progress outside this file.
- Never publish releases, tags, packages, or deployments without explicit authorization.

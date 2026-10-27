---
name: tiktoken
description: Count, encode, and decode text tokens offline using the tiktoken command.
created_on: 2026-10-14 14:52
last_modified: 2026-10-14 14:53
status: current
---

# tiktoken

Use `tiktoken [INPUT]...` to count tokens. The subject commands are `tiktoken text count [INPUT]...`, `tiktoken text encode [INPUT]...`, `tiktoken token decode [INPUT]...`, and `tiktoken model list`.

## Input and output

`[INPUT]...` accepts inline UTF-8 strings joined with spaces, or one existing non-directory file. `--file` takes precedence over positional input. With neither, read stdin. File/stdin text loses exactly one final LF; inline text preserves its trailing newlines. Root invocation without input on a terminal prints help; an explicit text/token action without input on a terminal returns an error. UTF-8 read errors and existing-file read errors are reported rather than interpreted as literal input.

Count prints one integer and LF. Encode prints space-separated unsigned token IDs and LF. Decode accepts whitespace-separated unsigned 32-bit IDs and writes the original token bytes followed by LF; those bytes can contain incomplete UTF-8. Empty ID input, negative/out-of-range IDs, and IDs absent from the selected encoding return errors. Text special-token markers are ordinary text.

## Options

Root and text/token actions share these flags:

- `--model`, `-m`: string, unset by default; selects the model's native encoding and takes precedence over `--encoding`. Unknown models return errors. Native model prefixes and `ft:` names follow tiktoken-rs 0.12.1.
- `--encoding`, `-e`: string, default `cl100k_base`. Accepts `o200k_harmony`, `o200k_base`, `cl100k_base`, `p50k_base`, `p50k_edit`, `r50k_base`, `gpt2`.
- `--file`, `-f`: path, unset by default; read that UTF-8 file.
- `--help`, `-h`: bool, prints help for the selected command. `tiktoken help [COMMAND]...` prints generated command help; `[COMMAND]...` names a command path.

Root `--version`, `-V`: bool, default false; prints the raw version and LF. `tiktoken version` does the same. Both are offline.

`tiktoken model list` lists all supported encoding names with model examples. Human mode uses readable entries; `AGENT=1`, `true`, or `yes` uses encoding/model TSV. These examples are a curated subset; native model resolution also accepts supported version prefixes.

`tiktoken skill` prints the Skills CLI install command and canonical source URL, byte-identically in both modes. It reads no files or network. This local Rust migration is unpublished; the canonical GitHub repository's published artifacts and skill may still describe Go.

All commands accept `--help`, `-h`; help groups include `tiktoken text`, `tiktoken token`, and `tiktoken model`. Agent help starts with the skill-reading alert. `AGENT` changes help/catalog/error presentation; token data and version bytes are identical. `COLUMNS` or terminal width controls human help clipping. Displayed filesystem paths fold the home directory to `~`.

Exit 0 means success, including help/version. Exit 1 means a parse, input, encoding, decoding, or output error; errors go once to stderr. Runtime errors print no usage screen. Operations are offline and read-only; token ranks are bundled in the tokenizer dependency.

## Generated help paths

Native help accepts the command path after `help`. The generated paths are `tiktoken help text`, `tiktoken help text count`, `tiktoken help text encode`, `tiktoken help token`, `tiktoken help token decode`, `tiktoken help model`, `tiktoken help model list`, `tiktoken help skill`, `tiktoken help version`, and `tiktoken help help`.

Subject help also accepts `tiktoken text help`, `tiktoken text help count`, `tiktoken text help encode`, `tiktoken text help help`, `tiktoken token help`, `tiktoken token help decode`, `tiktoken token help help`, `tiktoken model help`, `tiktoken model help list`, and `tiktoken model help help`.

## Examples

```sh
tiktoken 'Hello, world!'
tiktoken text encode 'hello world!'
printf '15339 1917 0\n' | tiktoken token decode
tiktoken text count -m gpt-4o --file article.txt
AGENT=1 tiktoken model list
```

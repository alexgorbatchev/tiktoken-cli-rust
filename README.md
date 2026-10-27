`tiktoken` counts text tokens, converts text to token IDs, and decodes IDs back to bytes. It runs offline for people preparing or inspecting text for supported language models.

# What It Does

- **Count text:** Print a token count from inline text, a file, or stdin.
- **Inspect token IDs:** Print IDs and decode them with the selected encoding.
- **Select an encoding:** Choose an encoding directly or use its model name.

# How It Works

- Pass text to `tiktoken`, or select `text count`, `text encode`, or `token decode`.
- Use `--file` for an explicit file, or pipe input through stdin.
- Read plain results from stdout; failures appear once on stderr.

# How it Really Works

- Token data is bundled; counting, encoding, decoding, and model listing make no network requests.
- The default encoding is `cl100k_base`. `--model` takes precedence over `--encoding`; an unknown model returns an error.
- File and stdin input lose exactly one final newline. Inline input keeps its trailing newlines. Existing-file read failures and filesystem inspection failures remain errors.
- Special-token markers in text are encoded as ordinary text. Decoding writes original token bytes, including bytes that form incomplete UTF-8, followed by a newline.
- `AGENT=1` makes help and catalog output compact. Counts, token IDs, decoded bytes, and version output remain the same.
- Exit 0 indicates success; exit 1 indicates an argument, input, encoding, decoding, or output failure. Commands read input and write stdout without changing files.

# Installation

Rust release binaries are unpublished. Local contributor instructions are in [AGENTS.md](AGENTS.md). The canonical source and skill URLs refer to [alexgorbatchev/tiktoken-cli](https://github.com/alexgorbatchev/tiktoken-cli); they do not establish publication of the Rust migration.

# Quick Start

```sh
tiktoken 'Hello, world!'
tiktoken text encode 'hello world!'
printf '15339 1917 0\n' | tiktoken token decode
tiktoken text count --model gpt-4o --file article.txt
AGENT=1 tiktoken model list
```

Sample Output for `tiktoken 'Hello, world!'`:

```text
4
```

# Options & Flags

Root and text/token actions:

| Flag | Short | Default | Description |
| :--- | :--- | :--- | :--- |
| `--model <name>` | `-m` | unset | Choose a model's encoding; takes precedence |
| `--encoding <name>` | `-e` | `cl100k_base` | Choose an encoding directly |
| `--file <path>` | `-f` | unset | Read a UTF-8 file |
| `--help` | `-h` | `false` | Print command help |

Root metadata:

| Flag | Short | Default | Description |
| :--- | :--- | :--- | :--- |
| `--version` | `-V` | `false` | Print the raw version |

Supported encoding names: `o200k_harmony`, `o200k_base`, `cl100k_base`, `p50k_base`, `p50k_edit`, `r50k_base`, and `gpt2`. `model list` prints model examples. The operational reference is [skills/tiktoken/SKILL.md](skills/tiktoken/SKILL.md).

# License

[MIT License](LICENSE), copyright 2026 Alex Gorbatchev.

use anyhow::{Context, Result, bail};
use cli_command_kit::{
    HelpOptions, Mode,
    clap::{Arg, ArgAction, ArgMatches, Command},
};
use std::{
    ffi::OsString,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use tiktoken_rs::{
    CoreBPE,
    tokenizer::{Tokenizer, get_tokenizer},
};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const SKILL: &str = "Install the tiktoken skill using the Skills CLI:\n\n  npx skills add alexgorbatchev/tiktoken-cli --skill tiktoken\n\nSkill:\n\n  https://github.com/alexgorbatchev/tiktoken-cli/tree/main/skills/tiktoken\n";
const ENCODINGS: &[(&str, Tokenizer)] = &[
    ("o200k_harmony", Tokenizer::O200kHarmony),
    ("o200k_base", Tokenizer::O200kBase),
    ("cl100k_base", Tokenizer::Cl100kBase),
    ("p50k_base", Tokenizer::P50kBase),
    ("p50k_edit", Tokenizer::P50kEdit),
    ("r50k_base", Tokenizer::R50kBase),
    ("gpt2", Tokenizer::Gpt2),
];
const MODELS: &[&str] = &[
    "gpt-5",
    "gpt-4.1",
    "gpt-4o",
    "gpt-4",
    "gpt-3.5-turbo",
    "text-embedding-ada-002",
    "text-embedding-3-small",
    "text-embedding-3-large",
    "text-davinci-002",
    "text-davinci-003",
    "code-davinci-002",
    "code-cushman-001",
    "davinci",
    "curie",
    "babbage",
    "ada",
    "gpt2",
];

fn data_args(command: Command) -> Command {
    command
        .arg(
            Arg::new("model")
                .long("model")
                .short('m')
                .help("Model name; takes precedence over encoding"),
        )
        .arg(
            Arg::new("encoding")
                .long("encoding")
                .short('e')
                .default_value("cl100k_base")
                .help("Encoding name"),
        )
        .arg(
            Arg::new("file")
                .long("file")
                .short('f')
                .value_parser(cli_command_kit::clap::value_parser!(PathBuf))
                .help("Read input from this file"),
        )
        .arg(
            Arg::new("INPUT")
                .num_args(0..)
                .value_parser(cli_command_kit::clap::value_parser!(OsString))
                .allow_negative_numbers(true)
                .help("Inline input or a single existing file"),
        )
}

pub fn command() -> Command {
    data_args(Command::new("tiktoken").about("Count, encode, and decode tokens offline"))
        .subcommand_precedence_over_arg(true)
        .arg(
            Arg::new("version")
                .long("version")
                .short('V')
                .action(ArgAction::SetTrue)
                .exclusive(true)
                .help("Print the raw version"),
        )
        .subcommand(
            Command::new("text")
                .about("Text tokenization")
                .subcommand_required(true)
                .subcommand(data_args(Command::new("count").about("Count text tokens")))
                .subcommand(data_args(
                    Command::new("encode").about("Print space-separated token IDs"),
                )),
        )
        .subcommand(
            Command::new("token")
                .about("Token IDs")
                .subcommand_required(true)
                .subcommand(data_args(
                    Command::new("decode").about("Decode IDs to original bytes"),
                )),
        )
        .subcommand(
            Command::new("model")
                .about("Model and encoding catalog")
                .subcommand_required(true)
                .subcommand(Command::new("list").about("List encodings and model examples")),
        )
        .subcommand(Command::new("version").about("Print the raw version"))
}

pub fn help_options() -> HelpOptions {
    HelpOptions {
        skill: Some(SKILL.into()),
        ..HelpOptions::default()
    }
}

fn resolve(matches: &ArgMatches) -> Result<&'static CoreBPE> {
    if let Some(model) = matches.get_one::<String>("model") {
        return tiktoken_rs::bpe_for_model(model)
            .with_context(|| format!("unknown model {model:?}"));
    }
    let name = matches
        .get_one::<String>("encoding")
        .expect("parser provides default");
    let tokenizer = ENCODINGS
        .iter()
        .find(|(encoding, _)| encoding == name)
        .map(|(_, tokenizer)| *tokenizer)
        .with_context(|| format!("unknown encoding {name:?}"))?;
    tiktoken_rs::bpe_for_tokenizer(tokenizer)
}

fn read_file(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)
        .with_context(|| format!("read {}", cli_command_kit::display_path(path)))
}

fn positional_file(values: &[OsString]) -> Result<Option<&Path>> {
    if values.len() != 1 {
        return Ok(None);
    }
    let path = Path::new(&values[0]);
    match std::fs::metadata(path) {
        Ok(metadata) => Ok((!metadata.is_dir()).then_some(path)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => {
            Err(error).with_context(|| format!("inspect {}", cli_command_kit::display_path(path)))
        }
    }
}

fn input(matches: &ArgMatches, reader: &mut impl Read, terminal: bool) -> Result<String> {
    let values = matches
        .get_many::<OsString>("INPUT")
        .map(|values| values.cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    let mut text = if let Some(path) = matches.get_one::<PathBuf>("file") {
        read_file(path)?
    } else if let Some(path) = positional_file(&values)? {
        read_file(path)?
    } else if !values.is_empty() {
        return values
            .into_iter()
            .map(|value| {
                value
                    .into_string()
                    .map_err(|_| anyhow::anyhow!("inline input is not UTF-8"))
            })
            .collect::<Result<Vec<_>>>()
            .map(|values| values.join(" "));
    } else {
        if terminal {
            bail!("no input provided; pass an argument, --file, or pipe stdin");
        }
        let mut text = String::new();
        reader.read_to_string(&mut text).context("read stdin")?;
        text
    };
    if text.ends_with('\n') {
        text.pop();
    }
    Ok(text)
}

fn catalog(output: &mut impl Write, mode: Mode) -> Result<()> {
    for (name, tokenizer) in ENCODINGS {
        let models = MODELS
            .iter()
            .filter(|model| get_tokenizer(model) == Some(*tokenizer))
            .copied()
            .collect::<Vec<_>>()
            .join(", ");
        match mode {
            Mode::Human => writeln!(
                output,
                "{name}\n  Model examples: {}",
                if models.is_empty() { "none" } else { &models }
            )?,
            Mode::Agent => writeln!(output, "{name}\t{models}")?,
        }
    }
    Ok(())
}

fn perform(
    matches: &ArgMatches,
    root: &Command,
    options: &HelpOptions,
    reader: &mut impl Read,
    terminal: bool,
    output: &mut impl Write,
    mode: Mode,
) -> Result<()> {
    if cli_command_kit::write_skill(matches, options, output)? {
        return Ok(());
    }
    if matches.get_flag("version") || matches.subcommand_name() == Some("version") {
        writeln!(output, "{VERSION}")?;
        return Ok(());
    }
    let (operation, data) = match matches.subcommand() {
        Some(("model", _)) => return catalog(output, mode),
        Some(("text", subject)) | Some(("token", subject)) => {
            subject.subcommand().expect("parser requires action")
        }
        None => ("count", matches),
        _ => unreachable!("clap handles help and rejects unknown commands"),
    };
    if matches.subcommand().is_none()
        && terminal
        && !data.contains_id("file")
        && !data.contains_id("INPUT")
    {
        output.write_all(root.clone().render_help().to_string().as_bytes())?;
        return Ok(());
    }
    let bpe = resolve(data)?;
    let text = input(data, reader, terminal)?;
    match operation {
        "decode" => {
            let ids = text
                .split_whitespace()
                .map(|value| {
                    value
                        .parse::<u32>()
                        .with_context(|| format!("invalid token ID {value:?}"))
                })
                .collect::<Result<Vec<_>>>()?;
            if ids.is_empty() {
                bail!("no token IDs provided");
            }
            let bytes = bpe
                .decode_bytes(&ids)
                .context("invalid token ID for this encoding")?;
            output.write_all(&bytes)?;
            output.write_all(b"\n")?;
        }
        "encode" => {
            let ids = bpe.encode_ordinary(&text);
            let line = ids
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" ");
            writeln!(output, "{line}")?;
        }
        "count" => writeln!(output, "{}", bpe.encode_ordinary(&text).len())?,
        _ => unreachable!("clap restricts domain operations"),
    }
    Ok(())
}

/// CLI boundary: parsing is native clap; every error is printed once here.
pub fn entry(
    args: impl IntoIterator<Item = OsString>,
    reader: &mut impl Read,
    terminal: bool,
    output: &mut impl Write,
    diagnostics: &mut impl Write,
    mode: Mode,
) -> u8 {
    let options = help_options();
    let root =
        cli_command_kit::setup(command(), mode, &options).expect("valid static command tree");
    let result = match root.clone().try_get_matches_from(args) {
        Ok(matches) => perform(&matches, &root, &options, reader, terminal, output, mode),
        Err(error) if !error.use_stderr() => output
            .write_all(error.to_string().as_bytes())
            .map_err(Into::into),
        Err(error) => {
            let _ = write!(diagnostics, "{error}");
            return 1;
        }
    };
    match result {
        Ok(()) => 0,
        Err(error) => {
            let prefix = if mode == Mode::Agent {
                "ERR:"
            } else {
                "[ERROR]"
            };
            let _ = writeln!(diagnostics, "{prefix} {error:#}");
            1
        }
    }
}

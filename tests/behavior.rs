use cli_command_kit::Mode;
use std::{
    ffi::OsString,
    io::{self, Read, Write},
};
use tiktoken_cli::{command, entry, help_options};

fn invoke(args: &[&str], input: &[u8], tty: bool, mode: Mode) -> (u8, Vec<u8>, String) {
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = entry(
        args.iter().map(OsString::from),
        &mut &input[..],
        tty,
        &mut out,
        &mut err,
        mode,
    );
    (code, out, String::from_utf8(err).unwrap())
}

#[test]
fn counts_encodes_decodes_and_preserves_bytes() {
    for mode in [Mode::Human, Mode::Agent] {
        assert_eq!(
            invoke(&["tiktoken", "Hello, world!"], b"ignored", false, mode),
            (0, b"4\n".to_vec(), String::new())
        );
        assert_eq!(
            invoke(
                &["tiktoken", "text", "count", "Hello,", "world!"],
                b"",
                false,
                mode
            )
            .1,
            b"4\n"
        );
        assert_eq!(
            invoke(
                &["tiktoken", "text", "encode", "hello world!"],
                b"",
                false,
                mode
            )
            .1,
            b"15339 1917 0\n"
        );
        assert_eq!(
            invoke(
                &["tiktoken", "token", "decode", "15339", "1917", "0"],
                b"",
                false,
                mode
            )
            .1,
            b"hello world!\n"
        );
        assert_eq!(
            invoke(
                &["tiktoken", "token", "decode"],
                b"15339\n1917\t0\n",
                false,
                mode
            )
            .1,
            b"hello world!\n"
        );
        let bpe = tiktoken_rs::bpe_for_model("gpt-4").unwrap();
        let id = (0..256)
            .find(|id| {
                bpe.decode_bytes(&[*id])
                    .is_ok_and(|bytes| std::str::from_utf8(&bytes).is_err())
            })
            .unwrap();
        let result = invoke(
            &["tiktoken", "token", "decode", &id.to_string()],
            b"",
            false,
            mode,
        );
        let mut expected = bpe.decode_bytes(&[id]).unwrap();
        expected.push(b'\n');
        assert_eq!(result, (0, expected, String::new()));
    }
}

#[test]
fn file_and_stdin_resolution_trim_exactly_one_newline() {
    let dir = tempfile::tempdir_in(".tmp").unwrap();
    let path = dir.path().join("text");
    std::fs::write(&path, b"hello\n\n").unwrap();
    let file = path.to_str().unwrap();
    let bpe = tiktoken_rs::bpe_for_model("gpt-4").unwrap();
    let encoded = format!(
        "{}\n",
        bpe.encode_ordinary("hello\n")
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ")
    )
    .into_bytes();
    assert_eq!(
        invoke(
            &["tiktoken", "text", "encode", "-f", file],
            b"",
            false,
            Mode::Human
        )
        .1,
        encoded
    );
    assert_eq!(
        invoke(
            &["tiktoken", "text", "encode"],
            b"hello\n\n",
            false,
            Mode::Human
        )
        .1,
        encoded
    );
    let expected = format!("{}\n", bpe.encode_ordinary("hello\n").len()).into_bytes();
    assert_eq!(
        invoke(&["tiktoken", file], b"", false, Mode::Human).1,
        expected
    );
    assert_eq!(
        invoke(
            &["tiktoken", "text", "count", "-f", file, "ignored"],
            b"",
            false,
            Mode::Human
        )
        .1,
        expected
    );
    assert_eq!(
        invoke(&["tiktoken"], b"hello\n\n", false, Mode::Human).1,
        expected
    );
    assert_eq!(
        invoke(
            &["tiktoken", "text", "count", "hello\n\n"],
            b"",
            false,
            Mode::Human
        )
        .1,
        format!("{}\n", bpe.encode_ordinary("hello\n\n").len()).into_bytes()
    );
    std::fs::write(&path, b"15339 1917 0\n").unwrap();
    assert_eq!(
        invoke(
            &["tiktoken", "token", "decode", file],
            b"",
            false,
            Mode::Human
        )
        .1,
        b"hello world!\n"
    );
}

#[test]
fn native_encodings_models_and_ordinary_special_markers() {
    for encoding in [
        "cl100k_base",
        "o200k_base",
        "o200k_harmony",
        "p50k_base",
        "p50k_edit",
        "r50k_base",
        "gpt2",
    ] {
        let encoded = invoke(
            &[
                "tiktoken",
                "text",
                "encode",
                "-e",
                encoding,
                "世界 <|endoftext|>",
            ],
            b"",
            false,
            Mode::Human,
        );
        assert_eq!(encoded.0, 0, "{encoding}: {}", encoded.2);
        let decoded = invoke(
            &["tiktoken", "token", "decode", "-e", encoding],
            &encoded.1,
            false,
            Mode::Human,
        );
        assert_eq!(decoded.1, "世界 <|endoftext|>\n".as_bytes());
    }
    for model in ["gpt-4o", "gpt-5", "ft:gpt-4o:org:name:id", "gpt-4-0314"] {
        assert_eq!(
            invoke(
                &["tiktoken", "-m", model, "-e", "invalid", "hello"],
                b"",
                false,
                Mode::Human
            )
            .1,
            b"1\n"
        );
    }
    assert_eq!(invoke(&["tiktoken", ""], b"", false, Mode::Human).1, b"0\n");
}

#[test]
fn errors_are_single_and_do_not_print_runtime_usage() {
    for args in [
        vec!["tiktoken", "-m", "unknown", "hello"],
        vec!["tiktoken", "-e", "unknown", "hello"],
        vec!["tiktoken", "token", "decode", "4294967295"],
        vec!["tiktoken", "token", "decode", "-1"],
        vec!["tiktoken", "token", "decode", "oops"],
        vec!["tiktoken", "token", "decode"],
    ] {
        let (code, out, err) = invoke(&args, b"", false, Mode::Agent);
        assert_eq!(code, 1, "{args:?}");
        assert!(out.is_empty());
        assert!(!err.contains("Usage:"));
        assert_eq!(err.lines().count(), 1, "{err}");
    }
    let error = invoke(
        &["tiktoken", "-f", ".tmp/absent-file"],
        b"",
        false,
        Mode::Human,
    );
    assert_eq!(error.0, 1);
    assert!(error.2.contains("absent-file"));
    assert_eq!(
        invoke(&["tiktoken", "text", "count"], b"", true, Mode::Human).0,
        1
    );
    assert_eq!(invoke(&["tiktoken"], b"", true, Mode::Human).0, 0);
    assert_eq!(invoke(&["tiktoken"], b"\xff", false, Mode::Human).0, 1);
    assert_eq!(invoke(&["tiktoken", "--bad"], b"", false, Mode::Human).0, 1);
}

#[test]
fn existing_unreadable_positional_file_is_never_counted_as_literal() {
    #[cfg(unix)]
    {
        use std::os::unix::net::UnixListener;
        let directory = tempfile::tempdir_in(".tmp").unwrap();
        let path = directory.path().join("socket");
        let _socket = UnixListener::bind(&path).unwrap();
        let result = invoke(
            &["tiktoken", path.to_str().unwrap()],
            b"",
            false,
            Mode::Human,
        );
        assert_eq!(result.0, 1);
        assert!(result.1.is_empty());
        assert!(result.2.contains("socket"));
    }
}

#[test]
fn catalogs_help_versions_and_skill_are_operational() {
    for mode in [Mode::Human, Mode::Agent] {
        let catalog = invoke(&["tiktoken", "model", "list"], b"", false, mode);
        assert_eq!(catalog.0, 0);
        assert!(String::from_utf8(catalog.1).unwrap().contains("gpt-4o"));
        assert_eq!(
            invoke(&["tiktoken", "--version"], b"", false, mode).1,
            b"0.1.0\n"
        );
        assert_eq!(
            invoke(&["tiktoken", "version"], b"", false, mode).1,
            b"0.1.0\n"
        );
        let skill = invoke(&["tiktoken", "skill"], b"", false, mode);
        assert_eq!(skill.1, help_options().skill.unwrap().as_bytes());
        assert_eq!(
            invoke(&["tiktoken", "skill", "extra"], b"", false, mode).0,
            1
        );
        for args in [
            vec!["tiktoken", "--help"],
            vec!["tiktoken", "text", "--help"],
            vec!["tiktoken", "text", "encode", "--help"],
            vec!["tiktoken", "token", "--help"],
            vec!["tiktoken", "token", "decode", "--help"],
            vec!["tiktoken", "model", "--help"],
            vec!["tiktoken", "model", "list", "--help"],
            vec!["tiktoken", "skill", "--help"],
            vec!["tiktoken", "version", "--help"],
            vec!["tiktoken", "help", "text", "count"],
        ] {
            let result = invoke(&args, b"", false, mode);
            assert_eq!(result.0, 0, "{args:?}: {}", result.2);
            let text = String::from_utf8(result.1).unwrap();
            assert_eq!(
                text.starts_with("ALERT:"),
                mode == Mode::Agent,
                "{args:?}: {text}"
            );
        }
    }
    let options = help_options();
    let text = std::fs::read_to_string("skills/tiktoken/SKILL.md").unwrap();
    assert!(text.contains("name: tiktoken"));
    assert!(
        cli_command_kit::skill_omissions(&command(), &options, &text).is_empty(),
        "{:?}",
        cli_command_kit::skill_omissions(&command(), &options, &text)
    );
}

struct Broken;
impl Read for Broken {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other("reader failed"))
    }
}
impl Write for Broken {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("writer failed"))
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn propagates_native_input_and_output_failures() {
    let args = [OsString::from("tiktoken")];
    let mut out = Vec::new();
    let mut err = Vec::new();
    assert_eq!(
        entry(
            args.clone(),
            &mut Broken,
            false,
            &mut out,
            &mut err,
            Mode::Human
        ),
        1
    );
    assert!(String::from_utf8(err).unwrap().contains("reader failed"));
    for args in [
        vec!["tiktoken", "hello"],
        vec!["tiktoken", "skill"],
        vec!["tiktoken", "--help"],
        vec!["tiktoken", "--version"],
        vec!["tiktoken", "model", "list"],
    ] {
        let mut err = Vec::new();
        assert_eq!(
            entry(
                args.into_iter().map(OsString::from),
                &mut &b""[..],
                false,
                &mut Broken,
                &mut err,
                Mode::Human
            ),
            1
        );
        assert!(String::from_utf8(err).unwrap().contains("writer failed"));
    }
    assert_eq!(
        entry(
            [OsString::from("tiktoken"), OsString::from("--bad")],
            &mut &b""[..],
            false,
            &mut out,
            &mut Broken,
            Mode::Human
        ),
        1
    );
}

#[test]
fn native_executable_streams_results_and_errors_once() {
    use std::process::{Command, Stdio};
    let mut child = Command::new(env!("CARGO_BIN_EXE_tiktoken"))
        .args(["text", "encode"])
        .env("AGENT", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"hello world!\n")
        .unwrap();
    let result = child.wait_with_output().unwrap();
    assert!(result.status.success());
    assert_eq!(result.stdout, b"15339 1917 0\n");
    assert!(result.stderr.is_empty());
    let result = Command::new(env!("CARGO_BIN_EXE_tiktoken"))
        .args(["token", "decode", "4294967295"])
        .env("AGENT", "1")
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    assert_eq!(String::from_utf8(result.stderr).unwrap().lines().count(), 1);
}

#[test]
fn native_path_handling_and_inline_utf8_validation() {
    let directory = tempfile::tempdir_in(".tmp").unwrap();
    let path = directory.path().to_str().unwrap();
    assert_eq!(invoke(&["tiktoken", path], b"", false, Mode::Human).0, 0);
    assert_eq!(
        invoke(&["tiktoken", "-f", path], b"", false, Mode::Human).0,
        1
    );
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        let args = [OsString::from("tiktoken"), OsString::from_vec(vec![0xff])];
        let mut out = Vec::new();
        let mut err = Vec::new();
        assert_eq!(
            entry(args, &mut &b""[..], false, &mut out, &mut err, Mode::Human),
            1
        );
        assert!(String::from_utf8(err).unwrap().contains("not UTF-8"));
        let path = directory.path().join(OsString::from_vec(vec![b'f', 0xff]));
        std::fs::write(&path, b"hello").unwrap();
        let mut out = Vec::new();
        let mut err = Vec::new();
        assert_eq!(
            entry(
                [OsString::from("tiktoken"), path.into_os_string()],
                &mut &b""[..],
                false,
                &mut out,
                &mut err,
                Mode::Agent
            ),
            0
        );
        assert_eq!(out, b"1\n");
        let loop_path = directory.path().join("loop");
        std::os::unix::fs::symlink("loop", &loop_path).unwrap();
        let result = invoke(
            &["tiktoken", loop_path.to_str().unwrap()],
            b"",
            false,
            Mode::Human,
        );
        assert_eq!(
            result.0, 1,
            "metadata errors must not turn a path into literal input"
        );
    }
    let home = std::env::home_dir().unwrap();
    for (path, expected) in [
        (home.clone(), "~".to_owned()),
        (home.join("document"), "~/document".to_owned()),
        (
            home.with_file_name(format!(
                "{}-sibling",
                home.file_name().unwrap().to_string_lossy()
            )),
            format!("{}-sibling", home.display()),
        ),
        (std::path::PathBuf::from("/outside"), "/outside".into()),
    ] {
        assert_eq!(cli_command_kit::fold_home(&path, &home), expected);
    }
}

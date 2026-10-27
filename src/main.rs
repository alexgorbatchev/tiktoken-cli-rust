use std::io::IsTerminal;

fn main() -> std::process::ExitCode {
    let stdin = std::io::stdin();
    std::process::ExitCode::from(tiktoken_cli::entry(
        std::env::args_os(),
        &mut stdin.lock(),
        stdin.is_terminal(),
        &mut std::io::stdout().lock(),
        &mut std::io::stderr().lock(),
        cli_command_kit::Mode::from_env(),
    ))
}

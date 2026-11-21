use pbcp::cli::parse_args;
use pbcp::config::resolve;
use pbcp::detect::Environment;
use pbcp::input::read_input_with_limit;
use pbcp::osc52;
use pbcp::selection::{select_backend, Backend};
use pbcp::terminal::{self, select_transport};
use pbcp::wsl;
use std::io;

fn main() {
    if let Err(error) = run() {
        eprintln!("pbcp: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let options = parse_args(std::env::args_os().skip(1))?;
    let resolution = resolve(&options)?;
    for warning in &resolution.warnings {
        eprintln!("pbcp: warning: {warning}");
    }
    let environment = Environment::capture();
    let requested_backend = resolution.settings.backend.value.requested();

    if options.diagnose {
        print!(
            "{}",
            pbcp::diagnose::report_with_config(&environment, &resolution)
        );
        return Ok(());
    }

    let backend = select_backend(requested_backend, &environment)?;
    let transport = match backend {
        Backend::Osc52 => select_transport(&environment)?,
        Backend::WslClip => terminal::TerminalTransport::Direct,
    };
    let input = read_input_with_limit(&mut io::stdin().lock(), resolution.settings.max_bytes.value)
        .map_err(|error| format!("cannot read standard input: {error}"))?;

    match backend {
        Backend::Osc52 => {
            let sequence = osc52::generate(&input);
            let sequence = terminal::wrap(&sequence, transport);
            terminal::direct::write_to_controlling_terminal(&sequence)
                .map_err(|error| error.to_string())
        }
        Backend::WslClip => wsl::copy(&input).map_err(|error| error.to_string()),
    }
}

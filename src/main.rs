use pbcp::cli::parse_args;
use pbcp::detect::Environment;
use pbcp::input::read_input;
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
    let environment = Environment::capture();

    if options.diagnose {
        print!(
            "{}",
            pbcp::diagnose::report(&environment, options.backend.as_deref())
        );
        return Ok(());
    }

    let backend = select_backend(options.backend.as_deref(), &environment)?;

    match backend {
        Backend::Osc52 => {
            let transport = select_transport(&environment)?;
            let input = read_input(&mut io::stdin().lock())
                .map_err(|error| format!("cannot read standard input: {error}"))?;
            let sequence = osc52::generate(&input);
            let sequence = terminal::wrap(&sequence, transport);
            terminal::direct::write_to_controlling_terminal(&sequence)
                .map_err(|error| error.to_string())
        }
        Backend::WslClip => {
            let input = read_input(&mut io::stdin().lock())
                .map_err(|error| format!("cannot read standard input: {error}"))?;
            wsl::copy(&input).map_err(|error| error.to_string())
        }
    }
}

use crate::detect::Environment;
use crate::selection::{select_backend, Backend};
use crate::terminal::{select_transport, TerminalTransport};

pub fn report(environment: &Environment, requested_backend: Option<&str>) -> String {
    let mut output = String::new();
    push_signal(&mut output, "SSH session", environment.is_ssh_session());
    push_signal(&mut output, "WSL", environment.is_wsl());
    push_signal(&mut output, "tmux", environment.tmux);
    push_signal(&mut output, "screen", environment.screen);
    push_signal(&mut output, "Wayland display", environment.wayland_display);
    push_signal(&mut output, "X11 display", environment.x11_display);

    match select_backend(requested_backend, environment) {
        Ok(backend) => {
            output.push_str(&format!("Backend: {}\n", backend.name()));
            output.push_str(&format!(
                "Selection: {}\n",
                if requested_backend.is_some() {
                    "explicit"
                } else {
                    "automatic"
                }
            ));
            match transport_description(backend, environment) {
                Ok(description) => output.push_str(&format!("Transport: {description}\n")),
                Err(error) => output.push_str(&format!("Transport error: {error}\n")),
            }
        }
        Err(error) => {
            output.push_str("Backend: unavailable\n");
            output.push_str("Selection: failed\n");
            output.push_str(&format!("Selection error: {error}\n"));
        }
    }

    output
}

fn transport_description(
    backend: Backend,
    environment: &Environment,
) -> Result<&'static str, String> {
    match backend {
        Backend::Osc52 => match select_transport(environment)? {
            TerminalTransport::Direct => Ok("direct terminal"),
            TerminalTransport::Tmux => Ok("tmux passthrough"),
            TerminalTransport::Screen => Ok("screen passthrough"),
        },
        Backend::WslClip => Ok("Windows clip.exe fallback"),
    }
}

fn push_signal(output: &mut String, name: &str, present: bool) {
    output.push_str(&format!("{name}: {}\n", if present { "yes" } else { "no" }));
}

#[cfg(test)]
mod tests {
    use super::report;
    use crate::detect::Environment;

    #[test]
    fn reports_presence_only_and_selected_wsl_route() {
        let environment = Environment {
            wsl_interop: true,
            ..Environment::default()
        };
        assert_eq!(
            report(&environment, None),
            "SSH session: no\n\
             WSL: yes\n\
             tmux: no\n\
             screen: no\n\
             Wayland display: no\n\
             X11 display: no\n\
             Backend: wsl-clip\n\
             Selection: automatic\n\
             Transport: Windows clip.exe fallback\n"
        );
    }

    #[test]
    fn reports_missing_backend_without_environment_values() {
        let report = report(&Environment::default(), None);
        assert!(report.contains("Backend: unavailable\n"));
        assert!(report.contains("use --backend=osc52 to force OSC 52"));
    }

    #[test]
    fn reports_an_explicit_backend_even_when_automatic_detection_would_fail() {
        let report = report(&Environment::default(), Some("osc52"));
        assert!(report.contains("Backend: osc52\n"));
        assert!(report.contains("Selection: explicit\n"));
    }

    #[test]
    fn reports_unsupported_nested_multiplexers() {
        let environment = Environment {
            ssh_tty: true,
            tmux: true,
            screen: true,
            ..Environment::default()
        };
        assert!(report(&environment, None)
            .contains("Transport error: simultaneous tmux and screen nesting is not supported"));
    }
}

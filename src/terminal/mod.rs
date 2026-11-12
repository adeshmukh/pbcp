pub mod direct;
pub mod screen;
pub mod tmux;

use crate::detect::Environment;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalTransport {
    Direct,
    Tmux,
    Screen,
}

pub fn select_transport(environment: &Environment) -> Result<TerminalTransport, String> {
    match (environment.tmux, environment.screen) {
        (true, true) => Err("simultaneous tmux and screen nesting is not supported".to_owned()),
        (true, false) => Ok(TerminalTransport::Tmux),
        (false, true) => Ok(TerminalTransport::Screen),
        (false, false) => Ok(TerminalTransport::Direct),
    }
}

pub fn wrap(sequence: &[u8], transport: TerminalTransport) -> Vec<u8> {
    match transport {
        TerminalTransport::Direct => sequence.to_vec(),
        TerminalTransport::Tmux => tmux::wrap(sequence),
        TerminalTransport::Screen => screen::wrap(sequence),
    }
}

#[cfg(test)]
mod tests {
    use super::{select_transport, wrap, TerminalTransport};
    use crate::detect::Environment;

    #[test]
    fn selects_direct_or_single_multiplexer_transport() {
        assert_eq!(
            select_transport(&Environment::default()),
            Ok(TerminalTransport::Direct)
        );
        assert_eq!(
            select_transport(&Environment {
                tmux: true,
                ..Environment::default()
            }),
            Ok(TerminalTransport::Tmux)
        );
        assert_eq!(
            select_transport(&Environment {
                screen: true,
                ..Environment::default()
            }),
            Ok(TerminalTransport::Screen)
        );
    }

    #[test]
    fn rejects_ambiguous_multiplexer_nesting() {
        assert_eq!(
            select_transport(&Environment {
                tmux: true,
                screen: true,
                ..Environment::default()
            })
            .unwrap_err(),
            "simultaneous tmux and screen nesting is not supported"
        );
    }

    #[test]
    fn leaves_direct_sequences_unchanged() {
        assert_eq!(
            wrap(b"\x1b]52;c;AA==\x07", TerminalTransport::Direct),
            b"\x1b]52;c;AA==\x07"
        );
    }
}

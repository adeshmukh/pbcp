#[derive(Debug, Default, PartialEq, Eq)]
pub struct Environment {
    pub ssh_connection: bool,
    pub ssh_client: bool,
    pub ssh_tty: bool,
    pub wsl_interop: bool,
    pub wsl_distro_name: bool,
    pub tmux: bool,
    pub screen: bool,
    pub wayland_display: bool,
    pub x11_display: bool,
}

impl Environment {
    pub fn capture() -> Self {
        Self::from_presence(|name| std::env::var_os(name).is_some_and(|value| !value.is_empty()))
    }

    fn from_presence(mut is_present: impl FnMut(&str) -> bool) -> Self {
        Self {
            ssh_connection: is_present("SSH_CONNECTION"),
            ssh_client: is_present("SSH_CLIENT"),
            ssh_tty: is_present("SSH_TTY"),
            wsl_interop: is_present("WSL_INTEROP"),
            wsl_distro_name: is_present("WSL_DISTRO_NAME"),
            tmux: is_present("TMUX"),
            screen: is_present("STY"),
            wayland_display: is_present("WAYLAND_DISPLAY"),
            x11_display: is_present("DISPLAY"),
        }
    }

    pub fn is_ssh_session(&self) -> bool {
        self.ssh_connection || self.ssh_client || self.ssh_tty
    }

    pub fn is_wsl(&self) -> bool {
        self.wsl_interop || self.wsl_distro_name
    }
}

#[cfg(test)]
mod tests {
    use super::Environment;
    use std::collections::HashSet;

    #[test]
    fn captures_supported_environment_signals() {
        let variables = HashSet::from(["SSH_TTY", "WSL_INTEROP", "TMUX", "WAYLAND_DISPLAY"]);
        let environment = Environment::from_presence(|name| variables.contains(name));

        assert!(environment.ssh_tty);
        assert!(environment.is_ssh_session());
        assert!(environment.wsl_interop);
        assert!(environment.is_wsl());
        assert!(environment.tmux);
        assert!(environment.wayland_display);
        assert!(!environment.screen);
        assert!(!environment.x11_display);
    }

    #[test]
    fn does_not_infer_sessions_without_matching_signals() {
        let environment = Environment::from_presence(|_| false);
        assert_eq!(environment, Environment::default());
        assert!(!environment.is_ssh_session());
        assert!(!environment.is_wsl());
    }
}

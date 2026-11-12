use crate::detect::Environment;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Osc52,
    WslClip,
}

impl Backend {
    pub fn name(self) -> &'static str {
        match self {
            Self::Osc52 => "osc52",
            Self::WslClip => "wsl-clip",
        }
    }
}

pub fn select_backend(
    requested: Option<&str>,
    environment: &Environment,
) -> Result<Backend, String> {
    if let Some(name) = requested {
        return match name {
            "osc52" => Ok(Backend::Osc52),
            "wsl-clip" if environment.is_wsl() => Ok(Backend::WslClip),
            "wsl-clip" => Err("--backend=wsl-clip is only available in WSL".to_owned()),
            _ => Err(format!("unsupported backend: {name}")),
        };
    }

    if environment.is_ssh_session() {
        return Ok(Backend::Osc52);
    }
    if environment.is_wsl() {
        return Ok(Backend::WslClip);
    }

    Err("no supported clipboard backend detected; use --backend=osc52 to force OSC 52".to_owned())
}

#[cfg(test)]
mod tests {
    use super::{select_backend, Backend};
    use crate::detect::Environment;

    fn environment(ssh: bool, wsl: bool) -> Environment {
        Environment {
            ssh_connection: ssh,
            wsl_interop: wsl,
            ..Environment::default()
        }
    }

    #[test]
    fn explicit_backend_overrides_environment() {
        assert_eq!(
            select_backend(Some("osc52"), &environment(false, true)),
            Ok(Backend::Osc52)
        );
        assert_eq!(
            select_backend(Some("wsl-clip"), &environment(true, true)),
            Ok(Backend::WslClip)
        );
    }

    #[test]
    fn ssh_prefers_osc52_even_inside_wsl() {
        assert_eq!(
            select_backend(None, &environment(true, true)),
            Ok(Backend::Osc52)
        );
    }

    #[test]
    fn wsl_selects_clip_fallback() {
        assert_eq!(
            select_backend(None, &environment(false, true)),
            Ok(Backend::WslClip)
        );
    }

    #[test]
    fn explicit_wsl_backend_fails_outside_wsl() {
        assert_eq!(
            select_backend(Some("wsl-clip"), &environment(false, false)).unwrap_err(),
            "--backend=wsl-clip is only available in WSL"
        );
    }

    #[test]
    fn unsupported_or_undetected_backends_fail_explicitly() {
        assert_eq!(
            select_backend(Some("x11"), &environment(false, false)).unwrap_err(),
            "unsupported backend: x11"
        );
        assert_eq!(
            select_backend(None, &environment(false, false)).unwrap_err(),
            "no supported clipboard backend detected; use --backend=osc52 to force OSC 52"
        );
    }
}

use std::ffi::OsString;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Options {
    pub backend: Option<String>,
    pub diagnose: bool,
}

pub fn parse_args(args: impl IntoIterator<Item = OsString>) -> Result<Options, String> {
    let mut options = Options::default();

    for arg in args {
        let arg = arg
            .into_string()
            .map_err(|_| "arguments must be valid UTF-8".to_owned())?;

        if let Some(backend) = arg.strip_prefix("--backend=") {
            if backend.is_empty() {
                return Err("--backend requires a non-empty name".to_owned());
            }
            if options.backend.replace(backend.to_owned()).is_some() {
                return Err("--backend may only be specified once".to_owned());
            }
        } else if arg == "--diagnose" {
            options.diagnose = true;
        } else {
            return Err(format!("unsupported argument: {arg}"));
        }
    }

    Ok(options)
}

#[cfg(test)]
mod tests {
    use super::{parse_args, Options};

    fn args(values: &[&str]) -> Vec<std::ffi::OsString> {
        values.iter().map(std::ffi::OsString::from).collect()
    }

    #[test]
    fn parses_backend_and_diagnose_options() {
        assert_eq!(
            parse_args(args(&["--backend=osc52", "--diagnose"])).unwrap(),
            Options {
                backend: Some("osc52".to_owned()),
                diagnose: true,
            }
        );
    }

    #[test]
    fn rejects_empty_or_duplicate_backend_options() {
        assert_eq!(
            parse_args(args(&["--backend="])).unwrap_err(),
            "--backend requires a non-empty name"
        );
        assert_eq!(
            parse_args(args(&["--backend=osc52", "--backend=wsl"])).unwrap_err(),
            "--backend may only be specified once"
        );
    }

    #[test]
    fn rejects_unsupported_arguments() {
        assert_eq!(
            parse_args(args(&["file.txt"])).unwrap_err(),
            "unsupported argument: file.txt"
        );
    }

    #[cfg(unix)]
    #[test]
    fn rejects_non_utf8_arguments() {
        use std::os::unix::ffi::OsStringExt;

        assert_eq!(
            parse_args([std::ffi::OsString::from_vec(vec![0xff])]).unwrap_err(),
            "arguments must be valid UTF-8"
        );
    }
}

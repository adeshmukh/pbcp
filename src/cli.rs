use std::ffi::OsString;
use std::path::PathBuf;

use crate::config::{parse_backend, parse_max_bytes, BackendSetting};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Options {
    pub backend: Option<BackendSetting>,
    pub max_bytes: Option<usize>,
    pub config: Option<PathBuf>,
    pub no_config: bool,
    pub diagnose: bool,
}

pub fn parse_args(args: impl IntoIterator<Item = OsString>) -> Result<Options, String> {
    let mut options = Options::default();

    for arg in args {
        if let Some(path) = config_path(&arg) {
            if options.config.replace(path?).is_some() {
                return Err("--config may only be specified once".to_owned());
            }
            continue;
        }
        let arg = arg
            .into_string()
            .map_err(|_| "arguments must be valid UTF-8".to_owned())?;

        if let Some(backend) = arg.strip_prefix("--backend=") {
            if backend.is_empty() {
                return Err("--backend requires a non-empty name".to_owned());
            }
            if options.backend.is_some() {
                return Err("--backend may only be specified once".to_owned());
            }
            options.backend = Some(parse_backend(backend)?);
        } else if let Some(value) = arg.strip_prefix("--max-bytes=") {
            if options.max_bytes.is_some() {
                return Err("--max-bytes may only be specified once".to_owned());
            }
            options.max_bytes = Some(parse_max_bytes(value)?);
        } else if arg == "--no-config" {
            options.no_config = true;
        } else if arg == "--diagnose" {
            options.diagnose = true;
        } else {
            return Err(format!("unsupported argument: {}", arg.escape_debug()));
        }
    }

    if options.config.is_some() && options.no_config {
        return Err("--config and --no-config are mutually exclusive".to_owned());
    }
    Ok(options)
}

fn config_path(arg: &std::ffi::OsStr) -> Option<Result<PathBuf, String>> {
    #[cfg(unix)]
    let path = {
        use std::os::unix::ffi::{OsStrExt, OsStringExt};
        OsString::from_vec(arg.as_bytes().strip_prefix(b"--config=")?.to_vec())
    };
    #[cfg(not(unix))]
    let path = OsString::from(arg.to_str()?.strip_prefix("--config=")?);
    Some(if path.is_empty() {
        Err("--config requires a non-empty path".to_owned())
    } else {
        Ok(PathBuf::from(path))
    })
}

#[cfg(test)]
mod tests {
    use super::{parse_args, Options};
    use std::ffi::OsString;
    use std::path::PathBuf;

    fn args(values: &[&str]) -> Vec<std::ffi::OsString> {
        values.iter().map(std::ffi::OsString::from).collect()
    }

    #[test]
    fn parses_backend_and_diagnose_options() {
        assert_eq!(
            parse_args(args(&["--backend=osc52", "--diagnose"])).unwrap(),
            Options {
                backend: Some(crate::config::BackendSetting::Osc52),
                diagnose: true,
                ..Options::default()
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

    #[test]
    fn validates_limits_backends_duplicates_and_conflicts() {
        for values in [
            vec!["--config=a", "--no-config"],
            vec!["--no-config", "--config=a"],
            vec!["--config=a", "--config=b"],
            vec!["--config="],
            vec!["--max-bytes=1", "--max-bytes=2"],
            vec!["--max-bytes=0"],
            vec!["--max-bytes="],
            vec!["--max-bytes=unlimited"],
            vec!["--backend=unknown"],
        ] {
            assert!(parse_args(args(&values)).is_err(), "{values:?}");
        }
        let first = parse_args(args(&["--config=a", "--backend=auto", "--max-bytes=5"])).unwrap();
        let second = parse_args(args(&["--max-bytes=5", "--backend=auto", "--config=a"])).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.max_bytes, Some(5));
        assert_eq!(first.backend, Some(crate::config::BackendSetting::Auto));
    }

    #[cfg(unix)]
    #[test]
    fn accepts_native_non_utf8_explicit_paths() {
        use std::os::unix::ffi::OsStringExt;
        let options = parse_args([OsString::from_vec(b"--config=file-\xff".to_vec())]).unwrap();
        assert_eq!(
            options.config,
            Some(PathBuf::from(OsString::from_vec(b"file-\xff".to_vec())))
        );
    }
}

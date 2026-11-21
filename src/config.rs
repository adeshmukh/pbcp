use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const DEFAULT_MAX_BYTES: usize = 65_536;
// Bound UTF-16 and worst-case Base64/OSC 52/mux expansion to Rust's allocation range.
pub const MAX_SAFE_BYTES: usize = ((isize::MAX as usize - 26) / 8) * 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendSetting {
    Auto,
    Osc52,
    WslClip,
}

impl BackendSetting {
    pub fn name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Osc52 => "osc52",
            Self::WslClip => "wsl-clip",
        }
    }

    pub fn requested(self) -> Option<&'static str> {
        match self {
            Self::Auto => None,
            _ => Some(self.name()),
        }
    }
}

pub fn parse_backend(value: &str) -> Result<BackendSetting, String> {
    match value {
        "auto" => Ok(BackendSetting::Auto),
        "osc52" => Ok(BackendSetting::Osc52),
        "wsl-clip" => Ok(BackendSetting::WslClip),
        _ => Err("backend must be auto, osc52, or wsl-clip".to_owned()),
    }
}

pub fn parse_max_bytes(value: &str) -> Result<usize, String> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("max_bytes must be a positive decimal integer".to_owned());
    }
    match value.parse::<usize>() {
        Ok(number) if (1..=MAX_SAFE_BYTES).contains(&number) && expansions_fit(number) => {
            Ok(number)
        }
        _ => Err(format!("max_bytes must be between 1 and {MAX_SAFE_BYTES}")),
    }
}

fn expansions_fit(bytes: usize) -> bool {
    let allocation_limit = isize::MAX as usize;
    let utf16 = bytes.checked_mul(2);
    let wrapped = bytes
        .checked_add(2)
        .map(|bytes| bytes / 3)
        .and_then(|groups| groups.checked_mul(4))
        .and_then(|base64| base64.checked_add(8))
        .and_then(|osc52| osc52.checked_mul(2))
        .and_then(|escaped| escaped.checked_add(10));
    utf16.is_some_and(|size| size <= allocation_limit)
        && wrapped.is_some_and(|size| size <= allocation_limit)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Provenance {
    BuiltIn,
    File { path: PathBuf, line: usize },
    Cli,
}

impl std::fmt::Display for Provenance {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BuiltIn => write!(output, "built-in"),
            Self::File { path, line } => write!(output, "{}:{line}", identifier(path)),
            Self::Cli => write!(output, "CLI"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Setting<T> {
    pub value: T,
    pub source: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub backend: Setting<BackendSetting>,
    pub max_bytes: Setting<usize>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            backend: Setting {
                value: BackendSetting::Auto,
                source: Provenance::BuiltIn,
            },
            max_bytes: Setting {
                value: DEFAULT_MAX_BYTES,
                source: Provenance::BuiltIn,
            },
        }
    }
}

#[derive(Debug)]
enum Assignment {
    Backend(BackendSetting),
    MaxBytes(usize),
}

#[derive(Debug)]
pub struct ParsedFile {
    path: PathBuf,
    assignments: Vec<(usize, Assignment)>,
    pub warnings: Vec<String>,
}

impl Settings {
    pub fn merge(&mut self, file: &ParsedFile) {
        for (line, assignment) in &file.assignments {
            let source = Provenance::File {
                path: file.path.clone(),
                line: *line,
            };
            match assignment {
                Assignment::Backend(value) => {
                    self.backend = Setting {
                        value: *value,
                        source,
                    };
                }
                Assignment::MaxBytes(value) => {
                    self.max_bytes = Setting {
                        value: *value,
                        source,
                    };
                }
            }
        }
    }
}

pub fn identifier(path: &Path) -> String {
    format!("{:?}", path.as_os_str())
}

pub fn parse_file(path: &Path, bytes: &[u8]) -> Result<ParsedFile, String> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| format!("{}: configuration must be valid UTF-8", identifier(path)))?;
    let mut file = ParsedFile {
        path: path.to_owned(),
        assignments: Vec::new(),
        warnings: Vec::new(),
    };
    let mut seen = HashMap::new();
    for (index, raw_line) in text.split('\n').enumerate() {
        let line = index + 1;
        let location = format!("{}:{line}", identifier(path));
        let content = raw_line.trim_matches(|character: char| character.is_ascii_whitespace());
        if content.is_empty() || content.starts_with('#') {
            continue;
        }
        let (key, value) = content
            .split_once('=')
            .ok_or_else(|| format!("{location}: expected key=value"))?;
        let key = key.trim_matches(|character: char| character.is_ascii_whitespace());
        let value = value.trim_matches(|character: char| character.is_ascii_whitespace());
        if key.is_empty()
            || !key
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            return Err(format!(
                "{location}: key must be a non-empty ASCII identifier"
            ));
        }
        if value.is_empty() {
            return Err(format!("{location}: key {key}: value must be non-empty"));
        }
        if let Some(earlier) = seen.insert(key, line) {
            file.warnings.push(format!(
                "{location}: duplicate key {key}; overrides line {earlier}"
            ));
        }
        let assignment = match key {
            "backend" => parse_backend(value).map(Assignment::Backend),
            "max_bytes" => parse_max_bytes(value).map(Assignment::MaxBytes),
            _ => {
                file.warnings
                    .push(format!("{location}: unknown key {key}; ignored"));
                continue;
            }
        }
        .map_err(|error| format!("{location}: key {key}: {error}"))?;
        file.assignments.push((line, assignment));
    }
    Ok(file)
}

#[derive(Debug)]
pub struct Resolution {
    pub settings: Settings,
    pub warnings: Vec<String>,
    pub sources: Vec<String>,
}

pub fn resolve(options: &crate::cli::Options) -> Result<Resolution, String> {
    resolve_with(
        options,
        std::env::var_os("XDG_CONFIG_HOME").as_deref(),
        std::env::var_os("HOME").as_deref(),
        |path| std::fs::read(path),
    )
}

pub fn resolve_with(
    options: &crate::cli::Options,
    xdg: Option<&std::ffi::OsStr>,
    home: Option<&std::ffi::OsStr>,
    mut load: impl FnMut(&Path) -> std::io::Result<Vec<u8>>,
) -> Result<Resolution, String> {
    if options.config.is_some() && options.no_config {
        return Err("--config and --no-config are mutually exclusive".to_owned());
    }
    let mut resolution = Resolution {
        settings: Settings::default(),
        warnings: Vec::new(),
        sources: Vec::new(),
    };
    let mut files = Vec::new();
    if options.no_config {
        resolution
            .sources
            .push("system/user discovery: skipped (--no-config)".to_owned());
    } else if let Some(path) = &options.config {
        resolution
            .sources
            .push("system/user discovery: skipped (--config replacement)".to_owned());
        files.push((path.clone(), true));
    } else {
        files.push((PathBuf::from("/etc/pbcp/config"), false));
        let root = if let Some(xdg) = xdg.filter(|value| !value.is_empty()) {
            let root = PathBuf::from(xdg);
            if !root.is_absolute() {
                return Err(
                    "XDG_CONFIG_HOME must be an absolute path when discovery is enabled".to_owned(),
                );
            }
            Some(root)
        } else {
            home.filter(|value| !value.is_empty())
                .map(PathBuf::from)
                .filter(|path| path.is_absolute())
                .map(|path| path.join(".config"))
        };
        if let Some(root) = root {
            files.push((root.join("pbcp/config"), false));
        } else {
            resolution.warnings.push(
                "cannot determine user configuration directory; user discovery skipped".to_owned(),
            );
            resolution
                .sources
                .push("user discovery: skipped (no directory)".to_owned());
        }
    }
    for (path, required) in files {
        match load(&path) {
            Ok(bytes) => {
                let file = parse_file(&path, &bytes)?;
                resolution.settings.merge(&file);
                resolution.warnings.extend(file.warnings);
                resolution
                    .sources
                    .push(format!("{}: loaded", identifier(&path)));
            }
            Err(error) if !required && error.kind() == std::io::ErrorKind::NotFound => {
                resolution
                    .sources
                    .push(format!("{}: skipped (missing)", identifier(&path)));
            }
            Err(error) => {
                return Err(format!(
                    "cannot load configuration {}: {:?}",
                    identifier(&path),
                    error.kind()
                ));
            }
        }
    }
    if let Some(value) = options.backend {
        resolution.settings.backend = Setting {
            value,
            source: Provenance::Cli,
        };
    }
    if let Some(value) = options.max_bytes {
        // Keep this boundary valid even for callers that construct Options directly.
        let value = parse_max_bytes(&value.to_string())?;
        resolution.settings.max_bytes = Setting {
            value,
            source: Provenance::Cli,
        };
    }
    Ok(resolution)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Options;
    use std::ffi::OsStr;
    use std::io;

    fn parse(text: &str) -> Result<ParsedFile, String> {
        parse_file(Path::new("test-config"), text.as_bytes())
    }

    #[test]
    fn parses_whitespace_crlf_and_comments() {
        let file = parse(" \t# comment\r\n\r\n max_bytes \t= 12 \r\nbackend=osc52\n").unwrap();
        let mut settings = Settings::default();
        settings.merge(&file);
        assert_eq!(settings.max_bytes.value, 12);
        assert_eq!(settings.backend.value, BackendSetting::Osc52);
        assert_eq!(
            settings.max_bytes.source,
            Provenance::File {
                path: "test-config".into(),
                line: 3
            }
        );
        assert!(file.warnings.is_empty());
    }

    #[test]
    fn unknown_and_duplicate_keys_warn_and_later_wins() {
        let file = parse("future=anything=goes\nfuture=other\nmax_bytes=1\nmax_bytes=2").unwrap();
        let mut settings = Settings::default();
        settings.merge(&file);
        assert_eq!(settings.max_bytes.value, 2);
        assert_eq!(file.warnings.len(), 4);
        assert!(file.warnings[1].contains("duplicate key future; overrides line 1"));
        assert!(file.warnings[3].contains("duplicate key max_bytes; overrides line 3"));
        assert!(!file.warnings.join("\n").contains("anything"));
    }

    #[test]
    fn merge_is_per_key_and_auto_resets_backend() {
        let mut settings = Settings::default();
        settings.merge(&parse("max_bytes=9\nbackend=osc52").unwrap());
        settings.merge(&parse("backend=auto").unwrap());
        assert_eq!(settings.max_bytes.value, 9);
        assert_eq!(settings.backend.value.requested(), None);
        assert!(parse("").unwrap().warnings.is_empty());
    }

    #[test]
    fn rejects_malformed_syntax_and_invalid_shadowed_values() {
        for text in [
            "[section]",
            "=value",
            "bad-key=value",
            "é=value",
            "backend=",
            "backend=\"auto\"",
            "backend=auto # comment",
            "max_bytes=0\nmax_bytes=1",
            "backend=invalid\nbackend=auto",
            "max_bytes=+1",
            "max_bytes=1KiB",
            "max_bytes=unlimited",
            "max_bytes=١",
            "future= ",
        ] {
            assert!(parse(text).is_err(), "{text:?}");
        }
        assert!(parse_file(Path::new("test"), &[0xff]).is_err());
        assert!(parse("future=a=b # uninterpreted").is_ok());
    }

    #[test]
    fn limits_are_finite_and_safe_for_all_encoding_expansions() {
        assert_eq!(parse_max_bytes("0001").unwrap(), 1);
        let maximum = parse_max_bytes(&MAX_SAFE_BYTES.to_string()).unwrap();
        let base64 = maximum.checked_add(2).unwrap() / 3 * 4;
        assert!((base64 + 8) * 2 + 10 <= isize::MAX as usize);
        assert!(maximum * 2 <= isize::MAX as usize);
        assert!(parse_max_bytes(&(maximum + 1).to_string()).is_err());
        assert!(parse_max_bytes(&usize::MAX.to_string()).is_err());
        assert!(parse_max_bytes("999999999999999999999999999999999").is_err());
    }

    #[test]
    fn diagnostics_escape_paths_and_never_echo_invalid_values() {
        let error = parse_file(Path::new("config\n\x1b[31m"), b"backend=secret\x1b").unwrap_err();
        assert!(!error.contains('\x1b'));
        assert!(!error.contains('\n'));
        assert!(!error.contains("secret"));
        assert!(error.contains("\\n\\u{1b}"));
    }

    #[test]
    fn discovery_layers_each_key_and_records_loaded_and_missing_sources() {
        let mut paths = Vec::new();
        let resolution = resolve_with(
            &Options {
                max_bytes: Some(7),
                ..Options::default()
            },
            Some(OsStr::new("/xdg")),
            Some(OsStr::new("/home/test")),
            |path| {
                paths.push(path.to_owned());
                Ok(if path == Path::new("/etc/pbcp/config") {
                    b"backend=osc52\nmax_bytes=3".to_vec()
                } else {
                    b"backend=auto".to_vec()
                })
            },
        )
        .unwrap();
        assert_eq!(
            paths,
            [
                PathBuf::from("/etc/pbcp/config"),
                PathBuf::from("/xdg/pbcp/config")
            ]
        );
        assert_eq!(resolution.settings.max_bytes.value, 7);
        assert_eq!(resolution.settings.max_bytes.source, Provenance::Cli);
        assert_eq!(resolution.settings.backend.value, BackendSetting::Auto);
        assert_eq!(
            resolution.settings.backend.source,
            Provenance::File {
                path: "/xdg/pbcp/config".into(),
                line: 1
            }
        );
        assert!(resolution.warnings.is_empty());
        assert!(resolution
            .sources
            .iter()
            .all(|source| source.ends_with("loaded")));

        let resolution = resolve_with(
            &Options::default(),
            None,
            Some(OsStr::new("/home/test")),
            |path| {
                if path == Path::new("/etc/pbcp/config") {
                    Ok(b"max_bytes=11".to_vec())
                } else {
                    Err(io::ErrorKind::NotFound.into())
                }
            },
        )
        .unwrap();
        assert_eq!(resolution.settings.max_bytes.value, 11);
        assert_eq!(resolution.settings.backend.source, Provenance::BuiltIn);
        assert!(resolution.sources[1].contains("/home/test/.config/pbcp/config"));
        assert!(resolution.sources[1].contains("skipped (missing)"));
    }

    #[test]
    fn explicit_file_replaces_discovery_and_disabled_mode_never_loads() {
        let mut calls = 0;
        let resolution = resolve_with(
            &Options {
                config: Some("relative-config".into()),
                backend: Some(BackendSetting::Osc52),
                ..Options::default()
            },
            Some(OsStr::new("invalid-relative-xdg")),
            None,
            |path| {
                calls += 1;
                assert_eq!(path, Path::new("relative-config"));
                Ok(b"max_bytes=8".to_vec())
            },
        )
        .unwrap();
        assert_eq!(calls, 1);
        assert_eq!(resolution.settings.max_bytes.value, 8);
        assert_eq!(resolution.settings.backend.source, Provenance::Cli);
        assert!(resolution.sources[0].contains("replacement"));
        let resolution = resolve_with(
            &Options {
                no_config: true,
                max_bytes: Some(2),
                ..Options::default()
            },
            Some(OsStr::new("invalid")),
            None,
            |_| panic!("discovery disabled"),
        )
        .unwrap();
        assert_eq!(resolution.settings.max_bytes.value, 2);
        assert_eq!(resolution.settings.backend.source, Provenance::BuiltIn);
        assert!(resolution.warnings.is_empty());
        assert!(resolution.sources[0].contains("--no-config"));
    }

    #[test]
    fn discovery_rejects_relative_xdg_and_warns_when_home_is_unusable() {
        assert!(resolve_with(
            &Options::default(),
            Some(OsStr::new("relative")),
            None,
            |_| panic!("invalid discovery")
        )
        .is_err());
        for home in [None, Some(OsStr::new("")), Some(OsStr::new("relative"))] {
            let resolution =
                resolve_with(&Options::default(), Some(OsStr::new("")), home, |path| {
                    assert_eq!(path, Path::new("/etc/pbcp/config"));
                    Err(io::ErrorKind::NotFound.into())
                })
                .unwrap();
            assert_eq!(resolution.warnings.len(), 1);
            assert_eq!(resolution.settings, Settings::default());
            assert!(resolution
                .sources
                .iter()
                .any(|source| source.contains("no directory")));
        }
        let mut paths = Vec::new();
        resolve_with(
            &Options::default(),
            Some(OsStr::new("")),
            Some(OsStr::new("/home/test")),
            |path| {
                paths.push(path.to_owned());
                Err(io::ErrorKind::NotFound.into())
            },
        )
        .unwrap();
        assert_eq!(paths[1], Path::new("/home/test/.config/pbcp/config"));
    }

    #[test]
    fn loading_errors_and_shadowed_invalid_files_are_fatal() {
        for kind in [
            io::ErrorKind::PermissionDenied,
            io::ErrorKind::InvalidInput,
            io::ErrorKind::IsADirectory,
        ] {
            assert!(resolve_with(&Options::default(), None, None, |_| Err(kind.into())).is_err());
        }
        assert!(resolve_with(
            &Options {
                config: Some("missing".into()),
                ..Options::default()
            },
            None,
            None,
            |_| Err(io::ErrorKind::NotFound.into())
        )
        .is_err());
        for bytes in [
            b"max_bytes=0".as_slice(),
            b"backend=bad",
            b"[section]",
            &[0xff],
        ] {
            let mut calls = 0;
            let result = resolve_with(
                &Options {
                    backend: Some(BackendSetting::Auto),
                    max_bytes: Some(5),
                    ..Options::default()
                },
                Some(OsStr::new("/xdg")),
                None,
                |_| {
                    calls += 1;
                    Ok(if calls == 1 {
                        bytes.to_vec()
                    } else {
                        b"backend=auto\nmax_bytes=5".to_vec()
                    })
                },
            );
            assert!(result.is_err());
        }
    }

    #[cfg(unix)]
    #[test]
    fn native_discovery_paths_are_preserved_and_escaped_in_reports() {
        use std::os::unix::ffi::OsStringExt;
        let root = std::ffi::OsString::from_vec(b"/xdg-\xff\n\x1b".to_vec());
        let resolution = resolve_with(&Options::default(), Some(&root), None, |path| {
            if path == Path::new("/etc/pbcp/config") {
                Err(io::ErrorKind::NotFound.into())
            } else {
                assert_eq!(path, PathBuf::from(&root).join("pbcp/config"));
                Ok(b"backend=auto".to_vec())
            }
        })
        .unwrap();
        let report = crate::diagnose::report_with_config(
            &crate::detect::Environment::default(),
            &resolution,
        );
        assert!(report.contains("\\xFF"));
        assert!(!report.contains('\x1b'));
        assert!(report.contains("\\n"));
    }
}

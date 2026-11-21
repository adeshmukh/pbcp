Linux x86-64 and ARM64 binaries for WSL and SSH hosts, statically linked with musl.

CI runs unit/CLI tests and static-binary diagnostic smoke checks on both Linux
architectures. This does not validate real clipboard delivery: OSC 52 requires a
compatible local terminal, including any multiplexer configuration. Native
macOS, Windows, Wayland, and X11 clipboard backends are not included.

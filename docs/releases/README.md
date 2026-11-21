# CI and releases

GitHub Actions runs formatting, all existing Rust tests, Clippy with warnings
denied, the zero-Cargo-dependency check, and release-planning tests on Linux
x86-64 and ARM64. Both architectures also run the tests against the release-mode
musl target, verify the binary has no ELF dynamic interpreter, and run
`--no-config --backend=osc52 --diagnose`. Real clipboard delivery is not a CI
claim. macOS and native Windows builds will be added when their backends exist.

Pull requests and manual workflow runs upload preview archives to the Actions
run's **Artifacts** section (14-day retention); they never publish releases.
These previews use the source Cargo version and carry their exact commit and
target in `BUILD.txt`. No `gh` installation is needed to download published
assets through GitHub's web UI or HTTPS.

## Automatic releases

A push to `main` publishes a GitHub Release only after all checks and both static
builds succeed. With no canonical version tags, the first release is `v0.0.1`.
Later releases increment the patch of the highest numeric `vMAJOR.MINOR.PATCH`
tag. Prerelease and unrelated tags are ignored. Major/minor bumps can be
introduced deliberately with a version tag; they are not automatic.

The release version is stamped into `Cargo.toml` and the generated lockfile only
in the disposable build checkout. No version-bump commits or extra release loops
are created. An existing version tag on the checked-out commit is reused on
retries, and already published releases are left unchanged. Interrupted draft
uploads can be resumed by rerunning the same workflow commit.
If an unpublished draft reserves the next version for another commit, publishing
fails explicitly instead of combining one commit's tag with another's binaries.
Rerun that draft's failed workflow, or remove the unpublished draft before retrying.

Main runs are serialized so concurrent builds cannot allocate the same patch
version. GitHub concurrency keeps one running and one pending run; rapid pushes
can replace a pending run. Thus releases represent successful processed commits,
not a guarantee of one release per push. Generated notes cover the interval
since the preceding version tag.

Each release contains:

- `pbcp-VERSION-x86_64-unknown-linux-musl.tar.gz`
- `pbcp-VERSION-aarch64-unknown-linux-musl.tar.gz`
- `SHA256SUMS`

Each archive contains the executable and `BUILD.txt` with its version, source
commit, and target. Short release notes prepend `RELEASE_NOTES.md` to GitHub's
generated change list and comparison link. Edit that template when supported
platforms or validation claims change. GitHub Releases, rather than GitHub
Packages, are the persistent distribution channel for these standalone binaries.

## Installation

Download the archive for your architecture and `SHA256SUMS` from the desired
[release](https://github.com/adeshmukh/pbcp/releases). In the download directory,
substitute your version and target:

```sh
archive=pbcp-0.0.1-x86_64-unknown-linux-musl.tar.gz
grep "  ./$archive$" SHA256SUMS | sha256sum --check --strict -
tar -xzf "$archive" pbcp
mkdir -p "$HOME/.local/bin"
install -m 755 pbcp "$HOME/.local/bin/pbcp"
```

Ensure `$HOME/.local/bin` is on `PATH`. Checksums detect corrupted downloads, not
independent authenticity. The `curl | sh` installer discussed for broader
distribution is separate work and is not supplied by this CI setup.

## Workflow permissions and recovery

Checks and builds have read-only repository access. Only the main-push publishing
job receives `contents: write`; pull requests never run that job. Actions are
pinned to immutable commits. No extra repository secret is required.

To retry a failed publication, use **Re-run all jobs** on the failed main run.
Do not delete or move published tags to reuse a version. A manual workflow run
validates/builds only; it is not a release trigger. Changes to this workflow
should be reviewed like release credentials because main builds execute code
that ultimately runs with release permissions.

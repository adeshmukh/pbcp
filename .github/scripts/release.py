"""Plan patch versions from Git tags and stamp the disposable release checkout."""

import argparse
import os
from pathlib import Path
import re
import subprocess
import tomllib


TAG = re.compile(r"v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)")


def plan_release(tags, head_tags):
    versions = {
        tag: tuple(map(int, TAG.fullmatch(tag).groups()))
        for tag in tags
        if TAG.fullmatch(tag)
    }
    existing = [tag for tag in head_tags if tag in versions]
    if existing:
        tag = max(existing, key=versions.get)
        version = versions[tag]
    elif versions:
        major, minor, patch = max(versions.values())
        version = (major, minor, patch + 1)
        tag = "v" + ".".join(map(str, version))
    else:
        tag, version = "v0.0.1", (0, 0, 1)

    previous = [name for name, value in versions.items() if value < version]
    return tag.removeprefix("v"), max(previous, key=versions.get, default="")


def stamp_manifest(path, version):
    if TAG.fullmatch("v" + version) is None:
        raise ValueError(f"invalid release version: {version}")
    text = path.read_text()
    current = tomllib.loads(text)["package"]["version"]
    pattern = re.compile(r'^version = "' + re.escape(current) + r'"$', re.MULTILINE)
    if len(pattern.findall(text)) != 1:
        raise ValueError("expected exactly one package version line in Cargo.toml")
    path.write_text(pattern.sub(f'version = "{version}"', text))


def git_lines(*args):
    return subprocess.check_output(["git", *args], text=True).splitlines()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["plan", "stamp"])
    parser.add_argument("--version")
    args = parser.parse_args()
    if args.command == "stamp":
        if args.version is None:
            parser.error("stamp requires --version")
        stamp_manifest(Path("Cargo.toml"), args.version)
        return

    version, previous = plan_release(
        git_lines("tag", "--list"),
        git_lines("tag", "--points-at", "HEAD"),
    )
    output = f"version={version}\nprevious={previous}\n"
    print(output, end="")
    with open(os.environ["GITHUB_OUTPUT"], "a") as destination:
        destination.write(output)


if __name__ == "__main__":
    main()

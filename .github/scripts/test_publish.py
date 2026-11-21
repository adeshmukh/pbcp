import os
from pathlib import Path
import subprocess
import tempfile
import unittest


SCRIPT = Path(__file__).with_name("publish.sh").resolve()


class PublishTests(unittest.TestCase):
    def publish(self, state="", previous="", fail_api=False):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            gh = root / "gh"
            gh.write_text(
                "#!/bin/sh\n"
                'printf "%s\\n" "$*" >> "$MOCK_LOG"\n'
                'if [ "$1" = api ]; then\n'
                '  [ "$MOCK_FAIL_API" = false ] || exit 23\n'
                '  printf "%s" "$MOCK_RELEASE_STATE"\n'
                "fi\n"
            )
            gh.chmod(0o755)
            (root / "pbcp-0.0.1-test.tar.gz").touch()
            (root / "SHA256SUMS").touch()
            log = root / "calls"
            environment = dict(
                os.environ,
                PATH=f"{root}:{os.environ['PATH']}",
                GH_REPO="example/pbcp",
                GITHUB_SHA="abc123",
                RELEASE_TAG="v0.0.1",
                VERSION="0.0.1",
                PREVIOUS_TAG=previous,
                MOCK_LOG=str(log),
                MOCK_RELEASE_STATE=state,
                MOCK_FAIL_API=str(fail_api).lower(),
            )
            result = subprocess.run(
                ["bash", str(SCRIPT)],
                cwd=root,
                env=environment,
                capture_output=True,
                text=True,
            )
            return result, log.read_text().splitlines()

    def test_new_release_is_draft_until_assets_are_attached(self):
        result, calls = self.publish()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(calls), 3)
        self.assertIn("release create v0.0.1", calls[1])
        self.assertIn("--target abc123", calls[1])
        self.assertIn("--draft --generate-notes", calls[1])
        self.assertNotIn("--notes-start-tag", calls[1])
        self.assertEqual(calls[2], "release edit v0.0.1 --draft=false --latest")

    def test_subsequent_release_generates_notes_from_previous_tag(self):
        result, calls = self.publish(previous="v0.0.0")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("--notes-start-tag v0.0.0", calls[1])

    def test_published_release_is_not_modified(self):
        result, calls = self.publish(state="published")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(calls), 1)

    def test_draft_upload_is_resumed_before_publication(self):
        result, calls = self.publish(state="draft")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("release upload v0.0.1", calls[1])
        self.assertIn("--clobber", calls[1])
        self.assertEqual(calls[2], "release edit v0.0.1 --draft=false --latest")

    def test_conflicting_draft_and_api_errors_never_publish(self):
        for arguments in [
            {"state": "conflicting-draft"},
            {"state": "unexpected"},
            {"fail_api": True},
        ]:
            with self.subTest(arguments=arguments):
                result, calls = self.publish(**arguments)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(len(calls), 1)


if __name__ == "__main__":
    unittest.main()

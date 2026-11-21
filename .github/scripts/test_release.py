from pathlib import Path
import tempfile
import unittest

from release import plan_release, stamp_manifest


class ReleaseTests(unittest.TestCase):
    def test_first_release_ignores_unrelated_and_noncanonical_tags(self):
        self.assertEqual(
            plan_release(["nightly", "v0.0.1-rc.1", "v00.0.9"], []),
            ("0.0.1", ""),
        )

    def test_patch_increment_uses_numeric_order_across_version_lines(self):
        self.assertEqual(
            plan_release(["v0.0.9", "v0.0.10"], []),
            ("0.0.11", "v0.0.10"),
        )
        self.assertEqual(
            plan_release(["v0.0.99", "v0.1.2", "v1.0.0"], []),
            ("1.0.1", "v1.0.0"),
        )

    def test_retry_reuses_existing_tag_instead_of_incrementing(self):
        self.assertEqual(
            plan_release(["v0.0.1", "v0.0.2"], ["v0.0.2", "nightly"]),
            ("0.0.2", "v0.0.1"),
        )
        self.assertEqual(
            plan_release(["v0.0.1", "v0.0.2"], ["v0.0.1"]),
            ("0.0.1", ""),
        )

    def test_stamps_only_package_version_and_rejects_invalid_versions(self):
        with tempfile.TemporaryDirectory() as directory:
            manifest = Path(directory) / "Cargo.toml"
            manifest.write_text('[package]\nname = "pbcp"\nversion = "0.0.1"\n')
            stamp_manifest(manifest, "0.0.12")
            self.assertIn('version = "0.0.12"', manifest.read_text())
            before = manifest.read_text()
            for version in ["bad", "00.0.1", '0.0.1"\nmalicious']:
                with self.assertRaises(ValueError):
                    stamp_manifest(manifest, version)
                self.assertEqual(manifest.read_text(), before)


if __name__ == "__main__":
    unittest.main()

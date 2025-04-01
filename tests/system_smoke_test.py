#!/usr/bin/python3

import subprocess
import unittest  # To get pretty error messages.

class SystemSmokeTest(unittest.TestCase):
    """The simplest possible system test."""

    dut = "target/debug/toyql"

    def test_help(self):
        """Checks that we can run as far as the help text."""
        output = subprocess.run(
            [self.dut, "--help"],
            capture_output=True,
            encoding="UTF-8")
        self.assertEqual(output.returncode, 0)
        self.assertIn("ToyQL query engine", output.stdout)
        self.assertEqual(output.stderr.rstrip(), "")

    def test_files(self):
        """Test that files on the command line are processed."""
        output = subprocess.check_output(
            [self.dut, "-f", "file_1", "-f", "file_2"],
            encoding="UTF-8")
        self.assertIn("file_1", output)
        self.assertIn("file_2", output)

    def test_queries(self):
        """Test that query text on the command line is processed."""
        output = subprocess.check_output(
            [self.dut, '"test string"', "7"],
            encoding="UTF-8")
        self.assertIn('"test string"', output)
        self.assertIn("7", output)


if __name__ == "__main__":
    unittest.main()

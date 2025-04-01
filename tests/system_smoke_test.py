#!/usr/bin/python3

import subprocess
import unittest  # To get pretty error messages.

class SystemSmokeTest(unittest.TestCase):
    """The simplest possible system test."""

    def test_smoke(self):
        """Does this target run at all?"""
        output = subprocess.check_output(["cargo", "run"], encoding="UTF-8")
        self.assertEqual(output, "Hello, World!\n")


if __name__ == "__main__":
    unittest.main()

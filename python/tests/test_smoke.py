import unittest

import hephaestus_workers as hw


class TestWorkerSkeleton(unittest.TestCase):
    def test_package_imports(self):
        self.assertEqual(hw.__version__, "0.1.0")


if __name__ == "__main__":
    unittest.main()

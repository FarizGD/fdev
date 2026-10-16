import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
from fdev_core.__main__ import context, endpoint, main

class CoreTests(unittest.TestCase):
    def test_endpoints(self):
        self.assertEqual(endpoint('http://localhost:11434'), 'http://localhost:11434/v1/chat/completions')
        self.assertEqual(endpoint('https://example.com/v1'), 'https://example.com/v1/chat/completions')
        self.assertEqual(endpoint('https://example.com/v1/chat/completions'), 'https://example.com/v1/chat/completions')
    def test_context_stays_in_project(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            (root / 'hello.py').write_text('print(1)')
            self.assertEqual(context('hello.py', root)[1], 'print(1)')
            with self.assertRaises(ValueError):
                context('../outside.py', root)
    @patch('fdev_core.__main__.chat', return_value='hello!')
    def test_one_shot(self, mocked):
        self.assertEqual(main(['--model', 'test-model', 'hello']), 0)
        self.assertEqual(mocked.call_args.args[1], 'test-model')
if __name__ == '__main__':
    unittest.main()

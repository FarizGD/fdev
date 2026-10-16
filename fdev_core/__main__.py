"""FDev minimal cross-platform coding agent core (stdlib only)."""
import argparse
import json
import os
from pathlib import Path
import sys
import urllib.error
import urllib.request
from . import __version__

SYSTEM = "You are FDev, a coding assistant. Be precise; explain changes and ask before modifying files."

def endpoint(base):
    base = base.rstrip('/')
    if base.endswith('/chat/completions'):
        return base
    if base.endswith('/v1'):
        return base + '/chat/completions'
    return base + '/v1/chat/completions'

def chat(messages, model, base, key, timeout=120):
    payload = json.dumps({'model': model, 'messages': messages, 'stream': False}).encode()
    headers = {'Content-Type': 'application/json'}
    if key:
        headers['Authorization'] = f'Bearer {key}'
    req = urllib.request.Request(endpoint(base), payload, headers=headers, method='POST')
    with urllib.request.urlopen(req, timeout=timeout) as response:
        result = json.load(response)
    return result['choices'][0]['message']['content']

def context(path, root):
    root = root.resolve()
    selected = (root / path).resolve()
    if not selected.is_relative_to(root) or not selected.is_file():
        raise ValueError('File must exist inside the current project')
    if selected.stat().st_size > 100_000:
        raise ValueError('File is too large (max 100 KB)')
    return selected.relative_to(root).as_posix(), selected.read_text(encoding='utf-8')

def main(argv=None):
    parser = argparse.ArgumentParser(prog='fdev', description='FDev coding agent, write code. yes.')
    parser.add_argument('prompt', nargs='*', help='One-shot prompt, or leave empty for interactive mode')
    parser.add_argument('--model', default=os.getenv('FDEV_MODEL', 'qwen2.5-coder:7b'))
    parser.add_argument('--base-url', default=os.getenv('FDEV_BASE_URL', 'http://127.0.0.1:11434'))
    parser.add_argument('--api-key', default=os.getenv('FDEV_API_KEY', ''))
    parser.add_argument('--file', action='append', default=[], help='Attach local project file (repeatable)')
    parser.add_argument('--version', action='version', version=__version__)
    args = parser.parse_args(argv)
    messages = [{'role': 'system', 'content': SYSTEM}]
    root = Path.cwd()
    try:
        for name in args.file:
            relative, data = context(name, root)
            messages.append({'role': 'user', 'content': f'Project file: {relative}\n'+chr(96)*3+f'\n{data}\n'+chr(96)*3})
    except (OSError, UnicodeError, ValueError) as exc:
        parser.error(str(exc))

    def ask(question):
        messages.append({'role': 'user', 'content': question})
        try:
            answer = chat(messages, args.model, args.base_url, args.api_key)
        except (urllib.error.URLError, TimeoutError, KeyError, ValueError) as exc:
            messages.pop()
            print(f'FDev request failed: {exc}', file=sys.stderr)
            return 1
        print(answer)
        messages.append({'role': 'assistant', 'content': answer})
        return 0

    if args.prompt:
        return ask(' '.join(args.prompt))
    print(f'FDev {__version__} | {args.model} | /exit to quit')
    while True:
        try:
            question = input('fdev> ').strip()
        except (EOFError, KeyboardInterrupt):
            print()
            return 0
        if question.lower() in ('/exit', '/quit', 'exit', 'quit'):
            return 0
        if question:
            ask(question)

if __name__ == '__main__':
    sys.exit(main())

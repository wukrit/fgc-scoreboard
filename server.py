#!/usr/bin/env python3
"""FGC Scoreboard LAN Server — zero-dependency local server for tournament use."""

import argparse
import hmac
import json
import os
import re
import secrets
import signal
import socket
import sys
import tempfile
import time
from collections import defaultdict
from http.server import HTTPServer, SimpleHTTPRequestHandler

SCOREBOARD_FILE = 'scoreboard.json'
MAX_BODY_SIZE = 65536  # 64KB — generous for scoreboard JSON
ALLOWED_PREFIXES = ('/_overlays/', '/css/', '/fonts/', '/js/')
ALLOWED_FILES = ('/controller.html', '/scoreboard.json')
AUTH_REALM = 'FGC Scoreboard'
RATE_LIMIT_WINDOW = 60  # seconds
DEFAULT_DATA = {
    "p1Name": "", "p1Team": "", "p1Score": "0",
    "p2Name": "", "p2Team": "", "p2Score": "0",
    "round": "", "game": "",
    "timestamp": ""
}


def generate_token():
    return secrets.token_urlsafe(32)


def parse_bearer(auth_header):
    if not auth_header:
        return None
    m = re.match(r'^Bearer\s+(\S+)\s*$', auth_header, re.I)
    return m.group(1) if m else None


def check_auth(auth_header, expected):
    if not expected:
        return True
    token = parse_bearer(auth_header)
    if not token:
        return False
    return hmac.compare_digest(token.encode('utf-8'), expected.encode('utf-8'))


def get_lan_ip():
    try:
        s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        s.connect(('8.8.8.8', 80))
        ip = s.getsockname()[0]
        s.close()
        return ip
    except OSError:
        return '127.0.0.1'


class ScoreboardHTTPServer(HTTPServer):
    def __init__(self, *args, auth_token=None, rate_limit=60, trust_proxy=False, **kwargs):
        self.auth_token = auth_token
        self.rate_limit = rate_limit
        self.trust_proxy = trust_proxy
        self.post_history = defaultdict(list)
        super().__init__(*args, **kwargs)


class ScoreboardHandler(SimpleHTTPRequestHandler):
    server_version = 'FGCScoreboard/1.0'

    def end_headers(self):
        if getattr(self.server, 'auth_token', None):
            self.send_header('X-FGC-Auth-Required', '1')
        super().end_headers()

    def _client_ip(self):
        if self.server.trust_proxy:
            forwarded = self.headers.get('X-Forwarded-For', '')
            if forwarded:
                return forwarded.split(',')[0].strip()
        return self.client_address[0]

    def _send_unauthorized(self):
        self.send_response(401)
        self.send_header(
            'WWW-Authenticate',
            f'Bearer realm="{AUTH_REALM}", error="invalid_token"'
        )
        self.send_header('Content-Type', 'application/json')
        self.end_headers()
        self.wfile.write(b'{"error":"invalid_token"}')

    def _check_rate_limit(self):
        limit = self.server.rate_limit
        if not limit:
            return True, 0
        ip = self._client_ip()
        now = time.monotonic()
        cutoff = now - RATE_LIMIT_WINDOW
        history = [t for t in self.server.post_history[ip] if t > cutoff]
        if len(history) >= limit:
            retry = max(1, int(RATE_LIMIT_WINDOW - (now - history[0])))
            return False, retry
        history.append(now)
        self.server.post_history[ip] = history
        return True, 0

    def do_GET(self):
        path = self.path.split('?')[0]
        if path == '/health':
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.end_headers()
            self.wfile.write(b'{"ok":true}')
        elif path == '/auth/check':
            if not check_auth(self.headers.get('Authorization'), self.server.auth_token):
                self._send_unauthorized()
                return
            self.send_response(204)
            self.end_headers()
        elif path == '/' + SCOREBOARD_FILE:
            try:
                with open(SCOREBOARD_FILE, 'r') as f:
                    data = f.read()
            except FileNotFoundError:
                data = json.dumps(DEFAULT_DATA, indent=2)
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.send_header('Cache-Control', 'no-store')
            self.end_headers()
            self.wfile.write(data.encode())
        elif path == '/':
            query = self.path.split('?', 1)
            self.path = '/controller.html' + ('?' + query[1] if len(query) > 1 else '')
            super().do_GET()
        elif path in ALLOWED_FILES or any(path.startswith(p) for p in ALLOWED_PREFIXES):
            super().do_GET()
        else:
            self.send_response(404)
            self.end_headers()

    def do_POST(self):
        if self.path.split('?')[0] != '/' + SCOREBOARD_FILE:
            self.send_response(404)
            self.end_headers()
            return
        if not check_auth(self.headers.get('Authorization'), self.server.auth_token):
            self._send_unauthorized()
            return
        allowed, retry_after = self._check_rate_limit()
        if not allowed:
            self.send_response(429)
            self.send_header('Retry-After', str(retry_after))
            self.send_header('Content-Type', 'application/json')
            self.end_headers()
            self.wfile.write(b'{"error":"rate_limit_exceeded"}')
            return
        try:
            length = int(self.headers.get('Content-Length', 0))
        except (ValueError, TypeError):
            self.send_response(400)
            self.end_headers()
            self.wfile.write(b'Invalid Content-Length')
            return
        if length <= 0 or length > MAX_BODY_SIZE:
            self.send_response(413 if length > MAX_BODY_SIZE else 400)
            self.end_headers()
            return
        body = self.rfile.read(length)
        try:
            data = json.loads(body)
        except (json.JSONDecodeError, ValueError):
            self.send_response(400)
            self.end_headers()
            self.wfile.write(b'Invalid JSON')
            return
        if not isinstance(data, dict) or not data.keys() <= set(DEFAULT_DATA.keys()):
            self.send_response(400)
            self.end_headers()
            self.wfile.write(b'Invalid schema')
            return
        if any(not isinstance(v, str) or len(v) > 128 for v in data.values()):
            self.send_response(400)
            self.end_headers()
            self.wfile.write(b'Invalid field values')
            return
        fd, tmp = tempfile.mkstemp(dir='.', suffix='.tmp')
        try:
            os.write(fd, body)
        finally:
            os.close(fd)
        try:
            os.replace(tmp, SCOREBOARD_FILE)
        except Exception:
            os.unlink(tmp)
            self.send_response(500)
            self.end_headers()
            return
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.end_headers()
        self.wfile.write(b'{"ok":true}')

    def handle(self):
        try:
            super().handle()
        except BrokenPipeError:
            pass

    def log_message(self, format, *args):
        if 'POST' in str(args):
            super().log_message(format, *args)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='FGC Scoreboard LAN Server')
    parser.add_argument('--port', type=int, default=8080,
                        help='Port to listen on (default: 8080; Railway uses PORT env)')
    parser.add_argument('--bind', default='0.0.0.0', help='Bind address (default: 0.0.0.0)')
    parser.add_argument('--token', help='Bearer token for POST auth (overrides FGC_AUTH_TOKEN env)')
    parser.add_argument('--generate-token', action='store_true',
                        help='Print a new token and exit')
    parser.add_argument('--trust-proxy', action='store_true',
                        help='Use X-Forwarded-For for rate limiting (only behind trusted proxy)')
    args = parser.parse_args()

    if args.generate_token:
        print(generate_token())
        sys.exit(0)

    auth_token = args.token or os.environ.get('FGC_AUTH_TOKEN') or None
    if auth_token is not None and len(auth_token) < 32:
        print('Error: auth token must be at least 32 characters', file=sys.stderr)
        sys.exit(1)

    rate_limit = int(os.environ.get('FGC_RATE_LIMIT', '60'))
    port = int(os.environ.get('PORT', args.port))
    bind = os.environ.get('FGC_BIND', args.bind)

    if not os.path.exists(SCOREBOARD_FILE):
        with open(SCOREBOARD_FILE, 'w') as f:
            json.dump(DEFAULT_DATA, f, indent=2)

    ip = get_lan_ip()

    print('\nFGC Scoreboard Server')
    if auth_token:
        print('  Auth:       enabled (token from FGC_AUTH_TOKEN or --token)')
    else:
        print('  Auth:       disabled')
    print(f'  Data:       {SCOREBOARD_FILE} (ephemeral on redeploy when hosted)')
    print(f'  Controller: http://{ip}:{port}/')
    print(f'  Overlay:    http://{ip}:{port}/_overlays/scoreboard.html')
    print(f'\nListening on {bind}:{port} (Ctrl+C to stop)')
    print('  Generate token: python3 server.py --generate-token\n')

    ScoreboardHTTPServer.allow_reuse_address = True
    server = ScoreboardHTTPServer(
        (bind, port),
        ScoreboardHandler,
        auth_token=auth_token,
        rate_limit=rate_limit,
        trust_proxy=args.trust_proxy,
    )

    def handle_signal(sig, frame):
        server.shutdown()

    signal.signal(signal.SIGTERM, handle_signal)

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
        print('\nServer stopped.')

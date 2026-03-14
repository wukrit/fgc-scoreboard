#!/usr/bin/env python3
"""FGC Scoreboard LAN Server — zero-dependency local server for tournament use."""

import argparse
import json
import os
import signal
import socket
import tempfile
from http.server import HTTPServer, SimpleHTTPRequestHandler

SCOREBOARD_FILE = 'scoreboard.json'
MAX_BODY_SIZE = 65536  # 64KB — generous for scoreboard JSON
ALLOWED_PREFIXES = ('/_overlays/', '/css/', '/fonts/', '/js/')
ALLOWED_FILES = ('/controller.html', '/scoreboard.json', '/')
DEFAULT_DATA = {
    "p1Name": "", "p1Team": "", "p1Score": "0",
    "p2Name": "", "p2Team": "", "p2Score": "0",
    "round": "", "game": "",
    "timestamp": ""
}


def get_lan_ip():
    try:
        s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        s.connect(('8.8.8.8', 80))
        ip = s.getsockname()[0]
        s.close()
        return ip
    except OSError:
        return '127.0.0.1'


class ScoreboardHandler(SimpleHTTPRequestHandler):

    def do_GET(self):
        path = self.path.split('?')[0]
        if path == '/' + SCOREBOARD_FILE:
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
        elif path in ALLOWED_FILES or any(path.startswith(p) for p in ALLOWED_PREFIXES):
            super().do_GET()
        else:
            self.send_response(404)
            self.end_headers()

    def do_POST(self):
        if self.path.split('?')[0] == '/' + SCOREBOARD_FILE:
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
            # Atomic write: temp file then rename
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
        else:
            self.send_response(404)
            self.end_headers()

    def handle(self):
        try:
            super().handle()
        except BrokenPipeError:
            pass

    def log_message(self, format, *args):
        # Only log POST requests to reduce noise during polling
        if 'POST' in str(args):
            super().log_message(format, *args)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='FGC Scoreboard LAN Server')
    parser.add_argument('--port', type=int, default=8080, help='Port to listen on (default: 8080)')
    args = parser.parse_args()

    # Create default scoreboard.json if missing
    if not os.path.exists(SCOREBOARD_FILE):
        with open(SCOREBOARD_FILE, 'w') as f:
            json.dump(DEFAULT_DATA, f, indent=2)

    ip = get_lan_ip()
    port = args.port

    print(f'\nFGC Scoreboard Server')
    print(f'Controller: http://{ip}:{port}/controller.html')
    print(f'Overlay:    http://{ip}:{port}/_overlays/scoreboard.html')
    print(f'\nListening on 0.0.0.0:{port} (Ctrl+C to stop)\n')

    HTTPServer.allow_reuse_address = True
    server = HTTPServer(('0.0.0.0', port), ScoreboardHandler)

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

#!/usr/bin/env python3
"""
Hello Flask - A simple Harbor example application

This Flask app demonstrates running a web backend over Unix Domain Sockets
for use with Harbor.

Run with:
    harbor examples/hello-flask/app.toml
"""

from flask import Flask, jsonify, render_template_string
import os
import socket
import datetime

app = Flask(__name__)

# HTML template with modern styling
INDEX_HTML = """
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Hello Harbor!</title>
    <style>
        * {
            box-sizing: border-box;
            margin: 0;
            padding: 0;
        }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto,
                         'Helvetica Neue', Arial, sans-serif;
            min-height: 100vh;
            display: flex;
            flex-direction: column;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #0f3460 100%);
            color: #eee;
        }
        header {
            padding: 2rem;
            text-align: center;
            background: rgba(0, 0, 0, 0.2);
        }
        .logo {
            font-size: 4rem;
            margin-bottom: 0.5rem;
        }
        h1 {
            font-size: 2.5rem;
            font-weight: 300;
            margin-bottom: 0.5rem;
        }
        .tagline {
            opacity: 0.7;
            font-size: 1.1rem;
        }
        main {
            flex: 1;
            padding: 2rem;
            max-width: 800px;
            margin: 0 auto;
            width: 100%;
        }
        .card {
            background: rgba(255, 255, 255, 0.1);
            border-radius: 12px;
            padding: 1.5rem;
            margin-bottom: 1.5rem;
            backdrop-filter: blur(10px);
            border: 1px solid rgba(255, 255, 255, 0.1);
        }
        .card h2 {
            font-size: 1.2rem;
            margin-bottom: 1rem;
            color: #64b5f6;
        }
        .info-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1rem;
        }
        .info-item {
            padding: 1rem;
            background: rgba(0, 0, 0, 0.2);
            border-radius: 8px;
        }
        .info-label {
            font-size: 0.8rem;
            text-transform: uppercase;
            opacity: 0.6;
            margin-bottom: 0.25rem;
        }
        .info-value {
            font-size: 1.1rem;
            font-family: 'Monaco', 'Menlo', monospace;
        }
        .button {
            display: inline-block;
            padding: 0.75rem 1.5rem;
            background: #64b5f6;
            color: #1a1a2e;
            border: none;
            border-radius: 8px;
            font-size: 1rem;
            cursor: pointer;
            text-decoration: none;
            transition: transform 0.2s, box-shadow 0.2s;
        }
        .button:hover {
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(100, 181, 246, 0.4);
        }
        footer {
            text-align: center;
            padding: 1.5rem;
            opacity: 0.6;
            font-size: 0.9rem;
        }
        footer a {
            color: #64b5f6;
        }
        #counter {
            font-size: 2rem;
            font-weight: bold;
            color: #64b5f6;
        }
    </style>
</head>
<body>
    <header>
        <div class="logo">⚓</div>
        <h1>Hello, Harbor!</h1>
        <p class="tagline">Your Flask app is running over Unix Domain Socket</p>
    </header>

    <main>
        <div class="card">
            <h2>System Information</h2>
            <div class="info-grid">
                <div class="info-item">
                    <div class="info-label">Socket Path</div>
                    <div class="info-value">{{ socket_path }}</div>
                </div>
                <div class="info-item">
                    <div class="info-label">Hostname</div>
                    <div class="info-value">{{ hostname }}</div>
                </div>
                <div class="info-item">
                    <div class="info-label">Server Time</div>
                    <div class="info-value">{{ server_time }}</div>
                </div>
                <div class="info-item">
                    <div class="info-label">Python Version</div>
                    <div class="info-value">{{ python_version }}</div>
                </div>
            </div>
        </div>

        <div class="card">
            <h2>Interactive Demo</h2>
            <p style="margin-bottom: 1rem;">Click the button to fetch data from the API:</p>
            <button class="button" onclick="fetchCounter()">Increment Counter</button>
            <p style="margin-top: 1rem;">Counter: <span id="counter">0</span></p>
        </div>

        <div class="card">
            <h2>About Harbor</h2>
            <p>Harbor is a local desktop app framework that connects web frontends
               to backend servers over Unix Domain Sockets. This provides:</p>
            <ul style="margin-top: 1rem; margin-left: 1.5rem;">
                <li>No network exposure - backend only accessible locally</li>
                <li>Lower latency than TCP loopback</li>
                <li>Familiar web technologies for desktop apps</li>
            </ul>
        </div>
    </main>

    <footer>
        <p>Built with <a href="https://github.com/marctjones/harbor">Harbor</a>
           and <a href="https://flask.palletsprojects.com/">Flask</a></p>
    </footer>

    <script>
        let count = 0;
        async function fetchCounter() {
            try {
                const response = await fetch('/api/increment');
                const data = await response.json();
                count = data.count;
                document.getElementById('counter').textContent = count;
            } catch (e) {
                console.error('Error:', e);
            }
        }
    </script>
</body>
</html>
"""

# Simple in-memory counter
counter = 0


@app.route('/')
def index():
    """Render the main page."""
    import sys
    return render_template_string(
        INDEX_HTML,
        socket_path=os.environ.get('HARBOR_SOCKET', '/tmp/hello-harbor.sock'),
        hostname=socket.gethostname(),
        server_time=datetime.datetime.now().strftime('%Y-%m-%d %H:%M:%S'),
        python_version=f"{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}"
    )


@app.route('/api/increment', methods=['GET', 'POST'])
def increment():
    """Increment and return the counter."""
    global counter
    counter += 1
    return jsonify({'count': counter})


@app.route('/api/status')
def status():
    """Return server status."""
    return jsonify({
        'status': 'ok',
        'counter': counter,
        'hostname': socket.gethostname(),
        'timestamp': datetime.datetime.now().isoformat()
    })


if __name__ == '__main__':
    # Get socket path from environment or use default
    socket_path = os.environ.get('HARBOR_SOCKET', '/tmp/hello-harbor.sock')

    # Remove existing socket file if present
    if os.path.exists(socket_path):
        os.remove(socket_path)

    print(f"Starting Flask app on unix://{socket_path}")
    app.run(host=f'unix://{socket_path}', debug=False)

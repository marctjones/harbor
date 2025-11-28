# Harbor Implementation Plan

## Phase 1: Core Framework (Current)

### 1.1 Configuration
- [x] TOML configuration parsing
- [x] AppConfig struct
- [x] BackendConfig struct
- [x] FrontendConfig struct
- [x] SettingsConfig struct
- [x] Default values
- [ ] Configuration validation
- [ ] Path expansion (~/...)

### 1.2 Backend Manager
- [x] Process spawning
- [x] Socket existence waiting
- [x] Socket connectivity check
- [x] Graceful shutdown (SIGTERM)
- [x] Force kill fallback
- [x] Socket cleanup
- [ ] Stdout/stderr capture
- [ ] Log forwarding

### 1.3 HarborApp
- [x] Load from file
- [x] Start backend
- [x] Stop backend
- [x] Return run config
- [ ] Health check loop
- [ ] Event callbacks

## Phase 2: Robustness

### 2.1 Error Handling
- [x] Basic error types
- [ ] Detailed error messages
- [ ] Error recovery suggestions
- [ ] Structured error logging

### 2.2 Process Management
- [ ] PID file management
- [ ] Orphan process cleanup
- [ ] Signal handling (SIGINT, SIGTERM)
- [ ] Restart throttling

### 2.3 Socket Management
- [ ] Socket permission setting
- [ ] Abstract socket support (Linux)
- [ ] Socket in XDG runtime dir

## Phase 3: Windows Support

### 3.1 Named Pipe Backend
- [ ] Named pipe creation
- [ ] Pipe security descriptor
- [ ] Pipe connectivity check
- [ ] Windows process management

### 3.2 Platform Abstraction
- [ ] Unified socket/pipe interface
- [ ] Platform-specific paths
- [ ] Process signals on Windows

## Phase 4: Advanced Features

### 4.1 Multiple Backends
- [ ] Multiple backend config sections
- [ ] Dependency ordering
- [ ] Combined health checking

### 4.2 Backend Communication
- [ ] Health check endpoint
- [ ] Reload signal
- [ ] Metrics endpoint

### 4.3 Logging
- [ ] Backend log capture
- [ ] Log rotation
- [ ] Log level filtering
- [ ] Structured logging

## Phase 5: Integration

### 5.1 Servo Integration
- [ ] Window creation helpers
- [ ] Event loop integration
- [ ] Navigation handling

### 5.2 Packaging
- [ ] Linux: AppImage helper
- [ ] macOS: .app bundle helper
- [ ] Windows: MSIX helper

### 5.3 Examples
- [x] Flask example
- [ ] FastAPI example
- [ ] Node.js example
- [ ] Nginx static site example

## Milestones

### v0.1.0 - Basic Framework
- Configuration parsing
- Single backend management
- Socket-based connectivity

### v0.2.0 - Production Ready
- Robust error handling
- Log capture
- Health monitoring

### v0.3.0 - Windows Support
- Named pipe backend
- Cross-platform API

### v0.4.0 - Advanced Features
- Multiple backends
- Metrics/health endpoints
- Reload support

### v1.0.0 - Stable Release
- Full documentation
- Packaging helpers
- Comprehensive examples

## Technical Debt

1. **Socket Permissions**: Need explicit mode setting
2. **Log Handling**: Backend logs currently lost
3. **Timeout Config**: Should be per-operation

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| serde | 1.x | Serialization |
| toml | 0.8.x | Config parsing |
| thiserror | 1.x | Error handling |
| log | 0.4.x | Logging |
| nix | 0.29.x | Unix signals |
| rigging | git | Transport URLs |

## Example Applications

### Hello Flask
```
examples/hello-flask/
├── app.toml           # Harbor configuration
├── app.py             # Flask application
└── requirements.txt   # Python dependencies
```

### Static Site (Nginx)
```
examples/static-nginx/
├── app.toml           # Harbor configuration
├── nginx.conf         # Nginx configuration
└── public/            # Static files
    └── index.html
```

### Full Stack (FastAPI + React)
```
examples/fullstack/
├── app.toml           # Harbor configuration
├── backend/           # FastAPI backend
│   └── main.py
└── frontend/          # React frontend
    └── build/
```

## Testing Strategy

### Unit Tests
- Configuration parsing
- Default value application
- Path handling

### Integration Tests
- Backend start/stop
- Socket connectivity
- Health checking

### Manual Tests
- Various backend types
- Long-running stability
- Resource cleanup

## Contributing

See AGENTS.md for AI assistant guidelines and coding standards.

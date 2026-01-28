# Textual Playground - Harbor TUI→GUI Testing App

A toy Textual application demonstrating various widgets and interactions. Perfect for testing Harbor's TUI-to-GUI rendering capabilities.

## Features

This app demonstrates:

### Interactive Widgets
- **Buttons** - Click events, variants (primary, success, warning)
- **Text Input** - Live text input with echo
- **Checkboxes** - Toggle states
- **Switches** - Animated toggles for settings

### Display Widgets
- **Labels** - Static and dynamic text updates
- **Progress Bar** - Value updates
- **Data Table** - Structured data display

### Interactions Tested
- Button clicks and counter updates
- Text input and live echo
- Checkbox/switch state changes
- Progress bar manipulation
- Dark mode toggle
- Auto-increment timer
- Keyboard shortcuts

### Keyboard Shortcuts
- `q` - Quit application
- `d` - Toggle dark mode
- `r` - Reset all values

## Running in Terminal (Traditional TUI)

```bash
# Install dependencies
pip install -r requirements.txt

# Run directly in terminal
python3 playground.py
```

## Running in Harbor (GUI Mode)

**Note**: This requires Harbor's Servo integration (Phase 5) and TUI-to-GUI rendering (Epic #12) to be implemented.

```bash
# Once Harbor TUI rendering is ready:
harbor app.toml
```

## Widget Coverage

| Widget Type | Textual Widget | Status | Notes |
|-------------|----------------|--------|-------|
| Button | `Button` | ✓ | Multiple variants tested |
| Input | `Input` | ✓ | With placeholder and live updates |
| Label | `Label` | ✓ | Static and dynamic content |
| Checkbox | `Checkbox` | ✓ | State changes |
| Switch | `Switch` | ✓ | Animated toggles |
| Progress Bar | `ProgressBar` | ✓ | Value updates |
| Data Table | `DataTable` | ✓ | Multi-column display |
| Container | `Container` | ✓ | Layout sections |
| Horizontal | `Horizontal` | ✓ | Row layout |
| Vertical | `Vertical` | ✓ | Column layout (implicit) |
| Scrollable | `ScrollableContainer` | ✓ | Main container |
| Header | `Header` | ✓ | App header |
| Footer | `Footer` | ✓ | Key bindings display |

## Testing Scenarios

### Manual Testing Checklist

- [ ] App launches successfully
- [ ] All widgets render correctly
- [ ] "Click Me!" button shows output
- [ ] Counter increments on button click
- [ ] Counter resets on "Reset Counter" button
- [ ] Text input shows live echo
- [ ] Checkboxes toggle state
- [ ] Dark mode switch changes theme
- [ ] Auto-increment switch starts timer
- [ ] Progress bar increases/decreases
- [ ] Progress bar fills to 100%
- [ ] Data table displays correctly
- [ ] Keyboard shortcut `d` toggles dark mode
- [ ] Keyboard shortcut `r` resets all values
- [ ] Keyboard shortcut `q` quits app

### Automated Testing

```python
# Future: Add pytest-textual tests
# Test button interactions
# Test input validation
# Test state management
```

## Protocol Testing

This app is ideal for testing the TUI→GUI protocol because it:

1. **Widget Diversity** - Covers most common Textual widgets
2. **State Changes** - Frequent UI updates to test diff/patch
3. **Events** - Multiple event types (click, input, toggle)
4. **Styling** - Custom CSS to test style conversion
5. **Layout** - Nested containers to test hierarchy
6. **Reactivity** - Timer-based updates (auto-increment)

## Architecture

```
playground.py (Textual App)
    ↓
textual-harbor adapter (future)
    ↓
JSON protocol over UDS
    ↓
Harbor widget renderer
    ↓
HTML/GUI display
```

## Future Enhancements

Ideas for extending this test app:

- [ ] Add Tree widget (nested structure)
- [ ] Add Select/Dropdown widget
- [ ] Add Modal/Dialog testing
- [ ] Add Chart/Sparkline widget
- [ ] Add Log/Console widget
- [ ] Add File picker simulation
- [ ] Add Color picker
- [ ] Add Slider widget
- [ ] Add Multi-page navigation (screens)
- [ ] Add Context menu testing

## Related Issues

- Epic #12 - Harbor as Universal TUI-to-GUI Renderer
- Issue #14 - Implement terminal emulation approach (PoC)
- Issue #15 - Implement widget protocol approach
- Issue #16 - Create textual-harbor Python package
- Issue #19 - Create demo Textual application for Harbor

## License

MPL-2.0 (same as Harbor)

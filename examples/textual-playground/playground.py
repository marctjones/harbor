#!/usr/bin/env python3
"""
Textual Playground - A toy application for testing Harbor's TUI-to-GUI rendering.

This app demonstrates various Textual widgets and interactions:
- Buttons and clicks
- Text input
- Labels and dynamic text
- Checkboxes and switches
- Tables
- Progress bars
- Layouts and containers
"""

from textual.app import App, ComposeResult
from textual.containers import Container, Horizontal, Vertical, ScrollableContainer
from textual.widgets import (
    Button,
    Checkbox,
    DataTable,
    Footer,
    Header,
    Input,
    Label,
    ProgressBar,
    Static,
    Switch,
)
from textual.binding import Binding


class TextualPlayground(App):
    """A playground app demonstrating various Textual widgets."""

    CSS = """
    Screen {
        background: $surface;
    }

    #main-container {
        width: 100%;
        height: 100%;
        padding: 1;
    }

    .section {
        border: solid $primary;
        margin: 1;
        padding: 1;
        height: auto;
    }

    .section-title {
        text-style: bold;
        color: $accent;
        margin-bottom: 1;
    }

    #button-section {
        height: auto;
    }

    #input-section {
        height: auto;
    }

    #checkbox-section {
        height: auto;
    }

    #table-section {
        height: 20;
    }

    #progress-section {
        height: auto;
    }

    Button {
        margin: 0 1;
    }

    Input {
        margin: 1 0;
        width: 50;
    }

    Checkbox {
        margin: 0 1;
    }

    Switch {
        margin: 0 2;
    }

    DataTable {
        height: 10;
    }

    ProgressBar {
        margin: 1 0;
    }

    #output-label {
        margin-top: 1;
        color: $success;
        text-style: bold;
    }

    #counter-label {
        color: $warning;
        text-style: bold;
    }
    """

    BINDINGS = [
        Binding("q", "quit", "Quit", show=True),
        Binding("d", "toggle_dark", "Toggle Dark Mode", show=True),
        Binding("r", "reset", "Reset", show=True),
    ]

    def __init__(self):
        super().__init__()
        self.counter = 0
        self.progress_value = 0

    def compose(self) -> ComposeResult:
        """Create child widgets."""
        yield Header()

        with ScrollableContainer(id="main-container"):
            # Button Section
            with Container(id="button-section", classes="section"):
                yield Label("Buttons & Clicks", classes="section-title")
                with Horizontal():
                    yield Button("Click Me!", id="btn-click", variant="primary")
                    yield Button("Increment", id="btn-increment", variant="success")
                    yield Button("Reset Counter", id="btn-reset", variant="warning")
                yield Label("Counter: 0", id="counter-label")
                yield Label("Last action: None", id="output-label")

            # Input Section
            with Container(id="input-section", classes="section"):
                yield Label("Text Input", classes="section-title")
                yield Input(placeholder="Type something...", id="text-input")
                yield Label("You typed: ", id="input-echo")

            # Checkbox/Switch Section
            with Container(id="checkbox-section", classes="section"):
                yield Label("Checkboxes & Switches", classes="section-title")
                with Horizontal():
                    yield Checkbox("Option 1", id="check1")
                    yield Checkbox("Option 2", id="check2")
                    yield Checkbox("Option 3", id="check3")
                with Horizontal():
                    yield Label("Dark Mode: ")
                    yield Switch(animate=True, id="dark-mode-switch")
                    yield Label("   Auto-increment: ")
                    yield Switch(animate=True, id="auto-increment-switch")

            # Progress Bar Section
            with Container(id="progress-section", classes="section"):
                yield Label("Progress Bars", classes="section-title")
                yield ProgressBar(total=100, show_eta=False, id="progress-bar")
                with Horizontal():
                    yield Button("+10%", id="btn-progress-up", variant="success")
                    yield Button("-10%", id="btn-progress-down", variant="error")
                    yield Button("Fill", id="btn-progress-fill")

            # Table Section
            with Container(id="table-section", classes="section"):
                yield Label("Data Table", classes="section-title")
                yield DataTable(id="data-table")

        yield Footer()

    def on_mount(self) -> None:
        """Initialize the app when mounted."""
        # Setup the data table
        table = self.query_one("#data-table", DataTable)
        table.add_columns("Widget", "Type", "Status")
        table.add_rows([
            ("Button", "Interactive", "✓ Working"),
            ("Input", "Interactive", "✓ Working"),
            ("Checkbox", "Interactive", "✓ Working"),
            ("Switch", "Interactive", "✓ Working"),
            ("Table", "Display", "✓ Working"),
            ("ProgressBar", "Display", "✓ Working"),
        ])

        # Set initial progress
        progress = self.query_one("#progress-bar", ProgressBar)
        progress.update(progress=0)

    def on_button_pressed(self, event: Button.Pressed) -> None:
        """Handle button press events."""
        output = self.query_one("#output-label", Label)

        if event.button.id == "btn-click":
            output.update("Last action: Button clicked!")

        elif event.button.id == "btn-increment":
            self.counter += 1
            self.update_counter()
            output.update("Last action: Counter incremented")

        elif event.button.id == "btn-reset":
            self.counter = 0
            self.update_counter()
            output.update("Last action: Counter reset")

        elif event.button.id == "btn-progress-up":
            self.progress_value = min(100, self.progress_value + 10)
            self.update_progress()
            output.update("Last action: Progress increased")

        elif event.button.id == "btn-progress-down":
            self.progress_value = max(0, self.progress_value - 10)
            self.update_progress()
            output.update("Last action: Progress decreased")

        elif event.button.id == "btn-progress-fill":
            self.progress_value = 100
            self.update_progress()
            output.update("Last action: Progress filled")

    def on_input_changed(self, event: Input.Changed) -> None:
        """Handle input changes."""
        echo = self.query_one("#input-echo", Label)
        echo.update(f"You typed: {event.value}")

    def on_checkbox_changed(self, event: Checkbox.Changed) -> None:
        """Handle checkbox changes."""
        output = self.query_one("#output-label", Label)
        status = "checked" if event.value else "unchecked"
        output.update(f"Last action: {event.checkbox.label} {status}")

    def on_switch_changed(self, event: Switch.Changed) -> None:
        """Handle switch changes."""
        output = self.query_one("#output-label", Label)

        if event.switch.id == "dark-mode-switch":
            self.dark = event.value
            status = "enabled" if event.value else "disabled"
            output.update(f"Last action: Dark mode {status}")

        elif event.switch.id == "auto-increment-switch":
            status = "enabled" if event.value else "disabled"
            output.update(f"Last action: Auto-increment {status}")
            if event.value:
                self.set_interval(0.5, self.auto_increment)

    def auto_increment(self) -> None:
        """Auto-increment counter if switch is on."""
        switch = self.query_one("#auto-increment-switch", Switch)
        if switch.value:
            self.counter += 1
            self.update_counter()

    def update_counter(self) -> None:
        """Update the counter label."""
        counter_label = self.query_one("#counter-label", Label)
        counter_label.update(f"Counter: {self.counter}")

    def update_progress(self) -> None:
        """Update the progress bar."""
        progress = self.query_one("#progress-bar", ProgressBar)
        progress.update(progress=self.progress_value)

    def action_toggle_dark(self) -> None:
        """Toggle dark mode."""
        self.dark = not self.dark
        switch = self.query_one("#dark-mode-switch", Switch)
        switch.value = self.dark

    def action_reset(self) -> None:
        """Reset all values."""
        self.counter = 0
        self.progress_value = 0
        self.update_counter()
        self.update_progress()

        # Reset checkboxes
        for checkbox in self.query(Checkbox):
            checkbox.value = False

        # Reset switches
        for switch in self.query(Switch):
            switch.value = False

        # Clear input
        text_input = self.query_one("#text-input", Input)
        text_input.value = ""

        output = self.query_one("#output-label", Label)
        output.update("Last action: All reset")


def main():
    """Run the playground app."""
    app = TextualPlayground()
    app.run()


if __name__ == "__main__":
    main()

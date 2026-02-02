#!/usr/bin/env rust-script
//! Simple standalone window test - proves winit works
//!
//! Run with: cargo run --example test-window

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

#[derive(Default)]
struct App {
    window: Option<Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("📱 Creating test window...");

        let window_attrs = Window::default_attributes()
            .with_title("Test Window - If you see this, winit works!")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));

        // On Linux, set app_id for Wayland
        #[cfg(target_os = "linux")]
        let window_attrs = {
            use winit::platform::wayland::WindowAttributesExtWayland;
            window_attrs.with_name("org.test.TestWindow", "TestWindow")
        };

        let window = event_loop
            .create_window(window_attrs)
            .expect("Failed to create window");

        println!("✅ Window created with ID: {:?}", window.id());
        println!("   Visibility: {:?}", window.is_visible());
        println!("   Has focus: {}", window.has_focus());

        // Make it visible and bring to front
        window.set_visible(true);
        window.focus_window();

        println!("   Set visible and focused");
        println!("\n🎯 Look for a window titled 'Test Window'");

        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("🚪 Window close requested");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                println!("🎨 RedrawRequested");
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::Resized(new_size) => {
                println!("📐 Resized to {:?}", new_size);
            }
            WindowEvent::Focused(focused) => {
                println!("👁️  Focus changed: {}", focused);
            }
            _ => {}
        }
    }
}

fn main() {
    println!("===========================================");
    println!("  Test Window - Minimal Winit Example");
    println!("===========================================\n");

    // Force X11 backend on Linux for better GNOME compatibility
    #[cfg(target_os = "linux")]
    std::env::set_var("WINIT_UNIX_BACKEND", "x11");

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app).expect("Failed to run app");
}

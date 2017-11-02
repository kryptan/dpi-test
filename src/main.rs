extern crate glutin;
extern crate gl;

use std::ffi::CStr;
use std::os::raw::c_char;
use gl::types::*;
use glutin::GlContext;

mod render;
use render::Renderer;

fn main() {
    let mut events_loop = glutin::EventsLoop::new();

    // Print info about all monitors.
    for (i, monitor) in events_loop.get_available_monitors().enumerate() {
        println!("Monitor #{}", i);
        println!("monitor.get_name() = {:?}", monitor.get_name());
        #[cfg(windows)] {
            use glutin::os::windows::MonitorIdExt;
            println!("#[cfg(windows)] monitor.native_id() = {:?}", monitor.native_id());
        }
        println!("monitor.get_dimensions() = {:?}", monitor.get_dimensions());
        println!("monitor.get_hidpi_factor() = {}", monitor.get_hidpi_factor());
        println!("monitor.get_position() = {:?}",  monitor.get_position());
        println!();
    }

    let mut hidpi_factor = events_loop.get_primary_monitor().get_hidpi_factor();
    println!("Primary monitor DPI-factor = {}", hidpi_factor);
    let initial_width = (1024 as f32*hidpi_factor) as u32;
    let initial_height = (728 as f32*hidpi_factor) as u32;
    let builder = glutin::WindowBuilder::new()
        .with_title("Winit DPI OpenGL Test")
        .with_dimensions(initial_width, initial_height);

    println!("Creating window");
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let window = glutin::GlWindow::new(builder, context, &events_loop).unwrap();
    let mut size = window.get_inner_size().unwrap();
    println!("window.get_inner_size() = {:?}", size);
    hidpi_factor = window.hidpi_factor();
    println!("window.hidpi_factor() = {}", hidpi_factor);
    println!();

    // Load OpenGL and print info about it.
    println!("OpenGL");
    unsafe { window.make_current() }.unwrap();
    gl::load_with(|name| window.context().get_proc_address(name) as *const _);
    for &(string_id, string_name) in &[(gl::VENDOR, "VENDOR"), (gl::RENDERER, "RENDERER"), (gl::VERSION, "VERSION")] {
        let string = unsafe { CStr::from_ptr(gl::GetString(string_id) as *const c_char) }.to_str().unwrap();
        println!("gl::GetString(gl::{}) = {:?}", string_name, string);
    }
    println!();

    let mut renderer = unsafe { Renderer::new() };

    events_loop.run_forever(|event| {
        match event {
            glutin::Event::WindowEvent { event: glutin::WindowEvent::Closed, .. } => {
                println!("Closing");
                return glutin::ControlFlow::Break;
            }
            glutin::Event::WindowEvent { event: glutin::WindowEvent::Resized(new_width, new_height), .. } => {
                size.0 = new_width;
                size.1 = new_height;
            }
            glutin::Event::WindowEvent { event: glutin::WindowEvent::HiDPIFactorChanged(new_hidpi_factor), .. } => {
                hidpi_factor = new_hidpi_factor;
            }
            _ => return glutin::ControlFlow::Continue,
        }

        println!("Event {:?}", event);
        println!("Resizing to {:?} with DPI-factor = {}", size, hidpi_factor);
        println!("window.get_inner_size() = {:?} (should be {:?})", window.get_inner_size().unwrap(), size);
        println!("window hidpi_factor = {} (should be {})", window.hidpi_factor(), hidpi_factor);
        println!();

        window.context().resize(size.0, size.1);
        unsafe { gl::Viewport(0, 0, size.0 as GLsizei, size.1 as GLsizei) };

        unsafe {
            gl::ClearColor(0.5, 0.5, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        unsafe { renderer.draw(size, hidpi_factor) };
        window.context().swap_buffers().unwrap();

        glutin::ControlFlow::Continue
    });

    unsafe { renderer.cleanup() };
}

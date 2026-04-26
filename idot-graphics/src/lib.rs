use std::sync::Mutex;

#[derive(Clone, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn from_hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        let r = u8::from_str_radix(&hex.get(0..2).unwrap_or("00"), 16).unwrap_or(0) as f32 / 255.0;
        let g = u8::from_str_radix(&hex.get(2..4).unwrap_or("00"), 16).unwrap_or(0) as f32 / 255.0;
        let b = u8::from_str_radix(&hex.get(4..6).unwrap_or("00"), 16).unwrap_or(0) as f32 / 255.0;
        Color { r, g, b, a: 1.0 }
    }

    pub fn to_hex(&self) -> String {
        let r = (self.r * 255.0) as u8;
        let g = (self.g * 255.0) as u8;
        let b = (self.b * 255.0) as u8;
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }

    pub fn white() -> Self {
        Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }
    }

    pub fn black() -> Self {
        Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }
    }

    pub fn red() -> Self {
        Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }
    }

    pub fn green() -> Self {
        Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 }
    }

    pub fn blue() -> Self {
        Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 }
    }
}

#[derive(Clone, Debug)]
pub struct DrawCommand {
    pub command_type: CommandType,
    pub color: Color,
}

#[derive(Clone, Debug)]
pub enum CommandType {
    Clear,
    Rect { x: f32, y: f32, width: f32, height: f32 },
    Circle { x: f32, y: f32, radius: f32 },
    Line { x1: f32, y1: f32, x2: f32, y2: f32, width: f32 },
}

#[derive(Clone, Debug)]
pub struct GraphicsState {
    pub commands: Vec<DrawCommand>,
    pub background_color: Color,
    pub window_width: u32,
    pub window_height: u32,
}

impl Default for GraphicsState {
    fn default() -> Self {
        GraphicsState {
            commands: Vec::new(),
            background_color: Color::white(),
            window_width: 800,
            window_height: 600,
        }
    }
}

lazy_static::lazy_static! {
    static ref GRAPHICS_STATE: Mutex<GraphicsState> = Mutex::new(GraphicsState::default());
}

pub fn create_window(width: u32, height: u32) {
    let mut state = GRAPHICS_STATE.lock().unwrap();
    state.window_width = width;
    state.window_height = height;
}

pub fn draw_rect(x: f32, y: f32, width: f32, height: f32, color: &str) {
    let mut state = GRAPHICS_STATE.lock().unwrap();
    state.commands.push(DrawCommand {
        command_type: CommandType::Rect { x, y, width, height },
        color: Color::from_hex(color),
    });
}

pub fn draw_circle(x: f32, y: f32, radius: f32, color: &str) {
    let mut state = GRAPHICS_STATE.lock().unwrap();
    state.commands.push(DrawCommand {
        command_type: CommandType::Circle { x, y, radius },
        color: Color::from_hex(color),
    });
}

pub fn draw_line(x1: f32, y1: f32, x2: f32, y2: f32, color: &str) {
    let mut state = GRAPHICS_STATE.lock().unwrap();
    state.commands.push(DrawCommand {
        command_type: CommandType::Line { x1, y1, x2, y2, width: 1.0 },
        color: Color::from_hex(color),
    });
}

pub fn draw_line_with_width(x1: f32, y1: f32, x2: f32, y2: f32, width: f32, color: &str) {
    let mut state = GRAPHICS_STATE.lock().unwrap();
    state.commands.push(DrawCommand {
        command_type: CommandType::Line { x1, y1, x2, y2, width },
        color: Color::from_hex(color),
    });
}

pub fn clear_window(color: &str) {
    let mut state = GRAPHICS_STATE.lock().unwrap();
    state.background_color = Color::from_hex(color);
}

pub fn clear_commands() {
    let mut state = GRAPHICS_STATE.lock().unwrap();
    state.commands.clear();
}

pub fn get_graphics_state() -> GraphicsState {
    let state = GRAPHICS_STATE.lock().unwrap();
    state.clone()
}

pub fn render_text() -> String {
    let state = GRAPHICS_STATE.lock().unwrap();
    let mut output = format!(
        "Window: {}x{}\nBackground: {}\nCommands:\n",
        state.window_width, state.window_height,
        state.background_color.to_hex()
    );
    
    for (i, cmd) in state.commands.iter().enumerate() {
        output.push_str(&format!("  [{}] ", i + 1));
        match &cmd.command_type {
            CommandType::Clear => output.push_str("Clear\n"),
            CommandType::Rect { x, y, width, height } => {
                output.push_str(&format!(
                    "Rect at ({:.0}, {:.0}) size {}x{} color {}\n",
                    x, y, *width as u32, *height as u32, cmd.color.to_hex()
                ));
            },
            CommandType::Circle { x, y, radius } => {
                output.push_str(&format!(
                    "Circle at ({:.0}, {:.0}) radius {:.0} color {}\n",
                    x, y, radius, cmd.color.to_hex()
                ));
            },
            CommandType::Line { x1, y1, x2, y2, width } => {
                output.push_str(&format!(
                    "Line from ({:.0}, {:.0}) to ({:.0}, {:.0}) width {:.1} color {}\n",
                    x1, y1, x2, y2, width, cmd.color.to_hex()
                ));
            },
        }
    }
    output
}

pub fn render_svg() -> String {
    let state = GRAPHICS_STATE.lock().unwrap();
    let mut svg = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
  <rect width="{}" height="{}" fill="{}"/>
"#,
        state.window_width, state.window_height,
        state.window_width, state.window_height,
        state.window_width, state.window_height,
        state.background_color.to_hex()
    );
    
    for cmd in &state.commands {
        match &cmd.command_type {
            CommandType::Clear => {},
            CommandType::Rect { x, y, width, height } => {
                svg.push_str(&format!(
                    r#"  <rect x="{:.0}" y="{:.0}" width="{:.0}" height="{:.0}" fill="{}"/>
"#,
                    x, y, width, height, cmd.color.to_hex()
                ));
            },
            CommandType::Circle { x, y, radius } => {
                svg.push_str(&format!(
                    r#"  <circle cx="{:.0}" cy="{:.0}" r="{:.0}" fill="{}"/>
"#,
                    x, y, radius, cmd.color.to_hex()
                ));
            },
            CommandType::Line { x1, y1, x2, y2, width } => {
                svg.push_str(&format!(
                    r#"  <line x1="{:.0}" y1="{:.0}" x2="{:.0}" y2="{:.0}" stroke="{}" stroke-width="{:.1}"/>
"#,
                    x1, y1, x2, y2, cmd.color.to_hex(), width
                ));
            },
        }
    }
    svg.push_str("</svg>");
    svg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_hex() {
        let color = Color::from_hex("#FF0000");
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.0);
        assert_eq!(color.b, 0.0);
    }

    #[test]
    fn test_color_to_hex() {
        let color = Color::red();
        assert_eq!(color.to_hex(), "#ff0000");
    }

    #[test]
    fn test_create_window() {
        clear_commands();
        create_window(1024, 768);
        let state = get_graphics_state();
        assert_eq!(state.window_width, 1024);
        assert_eq!(state.window_height, 768);
    }

    #[test]
    fn test_draw_commands() {
        clear_commands();
        create_window(800, 600);
        draw_rect(100.0, 100.0, 50.0, 50.0, "#FF0000");
        draw_circle(200.0, 200.0, 30.0, "#00FF00");
        draw_line(0.0, 0.0, 100.0, 100.0, "#0000FF");

        let state = get_graphics_state();
        assert_eq!(state.commands.len(), 3);
    }

    #[test]
    fn test_render_text() {
        clear_commands();
        create_window(800, 600);
        draw_rect(10.0, 10.0, 100.0, 100.0, "#FF0000");
        
        let text = render_text();
        assert!(text.contains("800x600"));
        assert!(text.contains("Rect"));
        assert!(text.contains("#ff0000"));
    }

    #[test]
    fn test_render_svg() {
        clear_commands();
        create_window(400, 300);
        draw_rect(50.0, 50.0, 100.0, 100.0, "#0000FF");
        
        let svg = render_svg();
        assert!(svg.contains("svg"));
        assert!(svg.contains("400"));
        assert!(svg.contains("300"));
        assert!(svg.contains("rect"));
    }
}

// C FFI exports for integration with other languages and the Idot interpreter
#[no_mangle]
pub extern "C" fn graphics_create_window(width: u32, height: u32) {
    create_window(width, height);
}

#[no_mangle]
pub extern "C" fn graphics_draw_rect(x: f32, y: f32, width: f32, height: f32, color: *const u8) {
    if color.is_null() {
        return;
    }
    let color_str = unsafe {
        std::ffi::CStr::from_ptr(color as *const i8)
            .to_string_lossy()
            .to_string()
    };
    draw_rect(x, y, width, height, &color_str);
}

#[no_mangle]
pub extern "C" fn graphics_draw_circle(x: f32, y: f32, radius: f32, color: *const u8) {
    if color.is_null() {
        return;
    }
    let color_str = unsafe {
        std::ffi::CStr::from_ptr(color as *const i8)
            .to_string_lossy()
            .to_string()
    };
    draw_circle(x, y, radius, &color_str);
}

#[no_mangle]
pub extern "C" fn graphics_draw_line(x1: f32, y1: f32, x2: f32, y2: f32, color: *const u8) {
    if color.is_null() {
        return;
    }
    let color_str = unsafe {
        std::ffi::CStr::from_ptr(color as *const i8)
            .to_string_lossy()
            .to_string()
    };
    draw_line(x1, y1, x2, y2, &color_str);
}

#[no_mangle]
pub extern "C" fn graphics_clear_window(color: *const u8) {
    if color.is_null() {
        return;
    }
    let color_str = unsafe {
        std::ffi::CStr::from_ptr(color as *const i8)
            .to_string_lossy()
            .to_string()
    };
    clear_window(&color_str);
}

#[no_mangle]
pub extern "C" fn graphics_render_text() -> *mut u8 {
    let output = render_text();
    let c_str = std::ffi::CString::new(output).unwrap();
    c_str.into_raw() as *mut u8
}

#[no_mangle]
pub extern "C" fn graphics_render_svg() -> *mut u8 {
    let output = render_svg();
    let c_str = std::ffi::CString::new(output).unwrap();
    c_str.into_raw() as *mut u8
}

#[no_mangle]
pub extern "C" fn graphics_free_string(ptr: *mut u8) {
    if !ptr.is_null() {
        unsafe {
            let _ = std::ffi::CString::from_raw(ptr as *mut i8);
        }
    }
}

# Idot Graphics Library

A graphics library for the Idot language that provides basic drawing capabilities including shapes, colors, and window management.

## Features

- **Window Creation**: Create custom-sized windows
- **Shape Drawing**: Draw rectangles, circles, and lines with customizable colors
- **Color Support**: Full hex color support (#RRGGBB format)
- **Cross-platform**: Designed for Windows, macOS, and Linux

## Available Functions

### Window Management

#### `create_window(width, height)`
Creates a graphics window with the specified dimensions.

**Parameters:**
- `width` (number): Window width in pixels
- `height` (number): Window height in pixels

**Example:**
```idot
create_window(800, 600);
```

### Drawing Functions

#### `draw_rect(x, y, width, height, color)`
Draws a filled rectangle at the specified position.

**Parameters:**
- `x` (number): X coordinate of top-left corner
- `y` (number): Y coordinate of top-left corner
- `width` (number): Rectangle width in pixels
- `height` (number): Rectangle height in pixels
- `color` (string): Hex color code (#RRGGBB)

**Example:**
```idot
draw_rect(100, 100, 200, 150, "#FF0000");
```

#### `draw_circle(x, y, radius, color)`
Draws a filled circle at the specified position.

**Parameters:**
- `x` (number): X coordinate of center
- `y` (number): Y coordinate of center
- `radius` (number): Circle radius in pixels
- `color` (string): Hex color code (#RRGGBB)

**Example:**
```idot
draw_circle(400, 300, 50, "#00FF00");
```

#### `draw_line(x1, y1, x2, y2, color)`
Draws a line from one point to another.

**Parameters:**
- `x1` (number): Starting X coordinate
- `y1` (number): Starting Y coordinate
- `x2` (number): Ending X coordinate
- `y2` (number): Ending Y coordinate
- `color` (string): Hex color code (#RRGGBB)

**Example:**
```idot
draw_line(0, 0, 800, 600, "#0000FF");
```

#### `clear_window(color)`
Clears the window and sets the background color.

**Parameters:**
- `color` (string): Hex color code (#RRGGBB)

**Example:**
```idot
clear_window("#FFFFFF");
```

## Color Reference

Colors are specified as hexadecimal strings in the format `#RRGGBB` where:
- `RR` = Red component (00-FF)
- `GG` = Green component (00-FF)
- `BB` = Blue component (00-FF)

**Common Colors:**
- Red: `#FF0000`
- Green: `#00FF00`
- Blue: `#0000FF`
- White: `#FFFFFF`
- Black: `#000000`
- Yellow: `#FFFF00`
- Cyan: `#00FFFF`
- Magenta: `#FF00FF`

## Complete Example

```idot
// Create a 800x600 window
create_window(800, 600);

// Draw a white background
clear_window("#FFFFFF");

// Draw some shapes
draw_rect(100, 100, 200, 150, "#FF0000");
draw_circle(400, 300, 75, "#00FF00");
draw_line(0, 0, 800, 600, "#0000FF");
draw_rect(600, 400, 100, 100, "#FFFF00");
```

## Architecture

The graphics library consists of two main components:

### 1. **idot-graphics (Rust Library)**
A lightweight graphics library that:
- Manages drawing commands in a command queue
- Supports multiple rendering backends (text, SVG)
- Provides FFI (Foreign Function Interface) for language integration
- Uses a global graphics state for thread-safe operations

### 2. **Idot Interpreter Integration**
The interpreter includes:
- Function call support in the language parser
- Built-in function handling for all graphics operations
- Direct integration with the graphics library

## Implementation Details

### Graphics State Management
The library uses a global `GraphicsState` protected by a mutex to safely manage:
- Window dimensions
- Background color
- Drawing commands (queue of shapes to render)

### Rendering
Two rendering modes are available:

1. **Text Rendering** (`render_text()`)
   - Human-readable text description of all drawing commands
   - Useful for debugging and testing

2. **SVG Rendering** (`render_svg()`)
   - Valid SVG (Scalable Vector Graphics) output
   - Can be saved and viewed in any browser or image viewer
   - Includes full color support

### Color Handling
- Hex color parsing from `#RRGGBB` format
- Automatic conversion between string and RGBA float representations
- Support for common color constants

## Testing

Run the graphics test suite:
```bash
python test_graphics.py
```

This will test:
- Function calls and argument handling
- SVG generation and output
- Color parsing for various color codes

## Future Enhancements

Potential additions for future versions:
- Text rendering
- Image loading and display
- Event handling (mouse, keyboard)
- Animation support
- Polygon drawing
- Gradient and pattern fills
- Layer support

## Technical Notes

- The graphics library is intentionally simple and focused on core drawing primitives
- It's designed as a plugin/extension to the Idot interpreter
- The FFI layer allows for eventual integration with GUI frameworks or web-based rendering
- Current implementation uses command queuing for flexible multi-backend rendering

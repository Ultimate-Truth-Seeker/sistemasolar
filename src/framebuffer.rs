use raylib::prelude::*;
pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub color_buffer: Image,
    background_color: Color,
    current_color: Color,
    texture: Option<Texture2D>,
    depth_buffer: Vec<f32>
}

impl Framebuffer {
    pub fn new(width: u32, height: u32, background_color: Color) -> Self {
        let color_buffer = Image::gen_image_color(width as i32, height as i32, background_color);
        let depth_buffer = vec![f32::INFINITY; (width*height) as usize];
        Framebuffer {
            width,
            height,
            color_buffer,
            background_color,
            current_color: Color::WHITE,
            texture: None,
            depth_buffer
        }
    }

    pub fn init_texture(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        self.texture = Some(rl.load_texture_from_image(thread, &self.color_buffer).unwrap());
    }

    /// Clears the framebuffer by regenerating the color buffer with the background color
    pub fn clear(&mut self) {
        self.color_buffer = Image::gen_image_color(self.width as i32, self.height as i32, self.background_color);
        self.depth_buffer.fill(f32::INFINITY);
    }

    /// Sets a single pixel in the buffer to the current color, if within bounds
    pub fn set_pixel(&mut self, x: u32, y: u32, depth: f32) {
        if x < self.width && y < self.height && depth < self.depth_buffer[(y*self.width + x) as usize] {
            // Calculate the offset into the Image data (raylib stores pixels in row-major order)
            self.depth_buffer[(y*self.width + x) as usize] = depth;
            self.color_buffer.draw_pixel(x as i32, y as i32, self.current_color);
            
        }
    }
    pub fn get_color(&mut self, x: u32, y: u32) {
        self.color_buffer.get_color(x as i32, y as i32);
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    /// Exports the framebuffer to an image file (BMP/PNG/etc.) using raylib's FFI
    pub fn render_to_file(&self, file_path: &str) {
        self.color_buffer.export_image(file_path);
    }

    pub fn swap_buffers(
        &self,
        window: &mut RaylibHandle,
        raylib_thread: &RaylibThread,
    ) {
        if let Ok(texture) = window.load_texture_from_image(raylib_thread, &self.color_buffer) {
            let mut renderer = window.begin_drawing(raylib_thread);
            renderer.draw_texture(&texture, 0, 0, Color::WHITE);
        }
    }
}
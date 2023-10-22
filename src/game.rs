use wasm_bindgen::prelude::*;
use web_sys::HtmlImageElement;
use web_sys::{ HtmlCanvasElement, WebGlRenderingContext, WebGlShader, WebGlProgram };
use std::convert::TryInto;
use std::convert::TryFrom;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ($($t:tt)*) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    };
}

#[wasm_bindgen(module = "game")]
pub struct Game {
    frames: i64,
    gl: WebGlRenderingContext,
    shader_program: WebGlProgram,
    canvas: HtmlCanvasElement,
    mouse_pos: (f32, f32),
    tile_spacing: f32,
    scale: (f32, f32),
    grid_size: (i32, i32),
    image: HtmlImageElement,
}
#[wasm_bindgen(module = "game")]
impl Game {
    pub fn new(canvas_id: &str) -> Game {
        let gl: WebGlRenderingContext = Game::init_webgl_context(&canvas_id);
        let shader_program: WebGlProgram = Game::setup_shaders(&gl).unwrap();
        let canvas = web_sys
            ::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id(&canvas_id)
            .unwrap();

        let image = web_sys::HtmlImageElement::new().unwrap();
        image.set_src("texture.png");

        let instance = Game {
            frames: 0,
            gl: gl,
            shader_program: shader_program,
            canvas: canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap(),
            mouse_pos: (0.0, 0.0),
            tile_spacing: 0.1,
            scale: (1.0, 1.0),
            grid_size: (10, 10),
            image: image,
        };

        instance
    }

    pub fn render(&mut self) {
        self.clear();
        self.update_viewport();

        self.draw_grid();

        self.frames += 1;
        if self.frames % 100 == 0 {
            log!("FRAME: {:?}", self.frames);
        }
    }

    pub fn init_webgl_context(canvas_id: &str) -> WebGlRenderingContext {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id(canvas_id).unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();

        let gl: WebGlRenderingContext = canvas
            .get_context("webgl")
            .unwrap()
            .unwrap()
            .dyn_into::<WebGlRenderingContext>()
            .unwrap();

        gl.viewport(0, 0, canvas.width().try_into().unwrap(), canvas.height().try_into().unwrap());

        gl
    }

    pub fn create_shader(
        gl: &WebGlRenderingContext,
        shader_type: u32,
        source: &str
    ) -> WebGlShader {
        let shader = gl.create_shader(shader_type).expect("Unable to create shader object");

        gl.shader_source(&shader, source);
        gl.compile_shader(&shader);

        if
            gl
                .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
                .as_bool()
                .unwrap_or(false)
        {
            shader
        } else {
            panic!(
                "{}",
                gl
                    .get_shader_info_log(&shader)
                    .unwrap_or_else(|| "Unknown error creating shader".into())
            )
        }
    }

    pub fn setup_shaders(gl: &WebGlRenderingContext) -> Result<WebGlProgram, JsValue> {
        let vertex_shader_source =
            "
        attribute vec3 coordinates;
        attribute vec4 vertexColor;
        attribute vec2 aTextureCoord;

        varying lowp vec4 vColor;
        varying highp vec2 vTextureCoord;

        void main(void) {
            gl_Position = vec4(coordinates, 1.0);
            vColor = vertexColor;
            vTextureCoord = aTextureCoord;
        }
        ";

        let fragment_shader_source =
            "
        precision mediump float;
        varying lowp vec4 vColor;

        varying highp vec2 vTextureCoord;
        uniform sampler2D uSampler;

        void main(void) {
            gl_FragColor = vColor;
            gl_FragColor = texture2D(uSampler, vTextureCoord);

        }
        "; // uniform vec4

        let vertex_shader = Game::create_shader(
            gl,
            WebGlRenderingContext::VERTEX_SHADER,
            vertex_shader_source
        );
        let fragment_shader = Game::create_shader(
            gl,
            WebGlRenderingContext::FRAGMENT_SHADER,
            fragment_shader_source
        );

        let shader_program = gl.create_program().unwrap();
        gl.attach_shader(&shader_program, &vertex_shader);
        gl.attach_shader(&shader_program, &fragment_shader);
        gl.link_program(&shader_program);

        if
            gl
                .get_program_parameter(&shader_program, WebGlRenderingContext::LINK_STATUS)
                .as_bool()
                .unwrap_or(false)
        {
            gl.use_program(Some(&shader_program));
            Ok(shader_program)
        } else {
            return Err(
                JsValue::from_str(
                    &gl
                        .get_program_info_log(&shader_program)
                        .unwrap_or_else(|| "Unknown error linking program".into())
                )
            );
        }
    }

    pub fn setup_vertices(
        gl: &WebGlRenderingContext,
        vertices: &[f32],
        shader_program: &WebGlProgram
    ) {
        let vertices_array = unsafe { js_sys::Float32Array::view(&vertices) };
        let vertex_buffer = gl.create_buffer().unwrap();

        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &vertices_array,
            WebGlRenderingContext::STATIC_DRAW
        );

        let coordinates_location = gl.get_attrib_location(&shader_program, "coordinates");

        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
        gl.vertex_attrib_pointer_with_i32(
            coordinates_location as u32,
            3,
            WebGlRenderingContext::FLOAT,
            false,
            0,
            0
        );

        gl.enable_vertex_attrib_array(coordinates_location as u32);
    }

    pub fn setup_colors(gl: &WebGlRenderingContext, colors: &[f32], shader_program: &WebGlProgram) {
        let color_array = unsafe { js_sys::Float32Array::view(&colors) };
        let color_buffer = gl.create_buffer().unwrap();

        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&color_buffer));
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &color_array,
            WebGlRenderingContext::STATIC_DRAW
        );

        let colors_location = gl.get_attrib_location(&shader_program, "vertexColor");

        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&color_buffer));
        gl.vertex_attrib_pointer_with_i32(
            colors_location as u32,
            4,
            WebGlRenderingContext::FLOAT,
            false,
            0,
            0
        );
        gl.enable_vertex_attrib_array(colors_location as u32);
    }

    fn setup_texture(&mut self, texture_coords: &[f32]) -> web_sys::WebGlTexture {
        let texture_coords_array = unsafe { js_sys::Float32Array::view(&texture_coords) };
        let texture_coords_buffer = self.gl.create_buffer().unwrap();
        self.gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&texture_coords_buffer));
        self.gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &texture_coords_array,
            WebGlRenderingContext::STATIC_DRAW
        );

        let sampler_location = self.gl
            .get_uniform_location(&self.shader_program, "uSampler")
            .unwrap();

        let texture = self.gl.create_texture().unwrap();
        self.gl.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&texture));

        let _ = self.gl.tex_image_2d_with_u32_and_u32_and_image(
            WebGlRenderingContext::TEXTURE_2D,
            0,
            WebGlRenderingContext::RGB.try_into().unwrap(),
            WebGlRenderingContext::RGB,
            WebGlRenderingContext::UNSIGNED_BYTE,
            &self.image
        );

        let texture_coord_location = self.gl.get_attrib_location(
            &self.shader_program,
            "aTextureCoord"
        );

        self.gl.uniform1i(Some(&sampler_location), 0);
        self.gl.generate_mipmap(WebGlRenderingContext::TEXTURE_2D);

        self.gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&texture_coords_buffer));
        self.gl.vertex_attrib_pointer_with_i32(
            texture_coord_location as u32,
            2,
            WebGlRenderingContext::FLOAT,
            false,
            0,
            0
        );
        self.gl.enable_vertex_attrib_array(texture_coord_location as u32);

        texture
    }

    pub fn draw_grid(&mut self) {
        let mut vertices: Vec<f32> = Vec::with_capacity(
            (self.grid_size.0 * self.grid_size.1 * 6 * 3) as usize
        );
        let mut colors: Vec<f32> = Vec::with_capacity(
            (self.grid_size.0 * self.grid_size.1 * 6 * 4) as usize
        );

        let width: f32 = i32::try_from(self.grid_size.0).unwrap() as f32;
        let height: f32 = i32::try_from(self.grid_size.1).unwrap() as f32;

        let tile_width: f32 = 1.0 / width;
        let tile_height: f32 = 1.0 / height;

        let mut texture_coords: [f32; 100 * 6 * 2] = [0.0; 100 * 6 * 2];

        // Y
        for i in 0..self.grid_size.1 {
            // X
            for j in 0..self.grid_size.0 {
                let left_relative: f32 =
                    ((j as f32) * tile_width + self.tile_spacing * tile_width) * self.scale.0;
                let right_relative: f32 =
                    (((j + 1) as f32) * tile_width - self.tile_spacing * tile_width) * self.scale.0;
                let top_relative: f32 =
                    ((i as f32) * tile_height + self.tile_spacing * tile_height) * self.scale.1;
                let bottom_relative: f32 =
                    (((i + 1) as f32) * tile_height - self.tile_spacing * tile_height) *
                    self.scale.1;

                let left: f32 = Game::convert_x_to_screen(left_relative);
                let right: f32 = Game::convert_x_to_screen(right_relative);
                let top: f32 = Game::convert_y_to_screen(top_relative);
                let bottom: f32 = Game::convert_y_to_screen(bottom_relative);

                let tile_colors = self.get_tile_colors(
                    j as i32,
                    i as i32,
                    left_relative,
                    right_relative,
                    top_relative,
                    bottom_relative
                );
                let mut iter = tile_colors.chunks_exact(4);

                let lt_color: &[f32] = iter.next().unwrap();
                let lb_color: &[f32] = iter.next().unwrap();
                let rt_color: &[f32] = iter.next().unwrap();
                let rb_color: &[f32] = iter.next().unwrap();

                // Triangle 1
                vertices.push(right); // X
                vertices.push(top); // Y
                vertices.push(0.0); // Z

                vertices.push(left); // X
                vertices.push(top); // Y
                vertices.push(0.0); // Z

                vertices.push(left); // X
                vertices.push(bottom); // Y
                vertices.push(0.0); // Z

                colors.extend_from_slice(&rt_color);
                colors.extend_from_slice(&lt_color);
                colors.extend_from_slice(&lb_color);

                texture_coords[((i * self.grid_size.0 + j) as usize) * 12 + 0] = 1.0;
                texture_coords[((i * self.grid_size.0 + j) as usize) * 12 + 1] = 0.0;

                texture_coords[((i * self.grid_size.0 + j) as usize) * 12 + 2] = 0.0;
                texture_coords[((i * self.grid_size.0 + j) as usize) * 12 + 3] = 0.0;

                texture_coords[((i * self.grid_size.0 + j) as usize) * 12 + 4] = 0.0;
                texture_coords[((i * self.grid_size.0 + j) as usize) * 12 + 5] = 1.0;

                // Triangle 2
                vertices.push(left); // X
                vertices.push(bottom); // Y
                vertices.push(0.0); // Z

                vertices.push(right); // X
                vertices.push(bottom); // Y
                vertices.push(0.0); // Z

                vertices.push(right); // X
                vertices.push(top); // Y
                vertices.push(0.0); // Z

                colors.extend_from_slice(&lb_color);
                colors.extend_from_slice(&rb_color);
                colors.extend_from_slice(&rt_color);

                texture_coords[((i * self.grid_size.0 + j) as usize) * 12 + 6] = 0.0;
                texture_coords[((i * self.grid_size.0 + j) as usize) * 12 + 7] = 1.0;

                texture_coords[((i * self.grid_size.0 + j) as usize) * 12 + 8] = 1.0;
                texture_coords[((i * self.grid_size.0 + j) as usize) * 12 + 9] = 1.0;

                texture_coords[((i * self.grid_size.0 + j) as usize) * 12 + 10] = 1.0;
                texture_coords[((i * self.grid_size.0 + j) as usize) * 12 + 11] = 0.0;
            }
        }

        let texture = self.setup_texture(&texture_coords);

        Game::setup_vertices(&self.gl, &vertices, &self.shader_program);
        Game::setup_colors(&self.gl, &colors, &self.shader_program);

        self.gl.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, (vertices.len() / 3) as i32);
    }

    pub fn on_mouse_move(&mut self, event: web_sys::MouseEvent) {
        self.mouse_pos = (
            (event.client_x() as f32) / (self.gl.drawing_buffer_width() as f32),
            (event.client_y() as f32) / (self.gl.drawing_buffer_height() as f32),
        );
    }

    fn clear(&self) {
        self.gl.clear_color(0.1f32, 0.1f32, 0.1f32, 1f32);
        self.gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
    }

    fn update_viewport(&mut self) {
        self.gl.viewport(0, 0, self.gl.drawing_buffer_width(), self.gl.drawing_buffer_height());

        let x_ratio = (self.gl.drawing_buffer_width() as f32) / (self.grid_size.0 as f32);
        let y_ratio = (self.gl.drawing_buffer_height() as f32) / (self.grid_size.1 as f32);

        if x_ratio / y_ratio > 1.0 {
            self.scale = (y_ratio / x_ratio, 1.0);
        } else {
            self.scale = (1.0, x_ratio / y_ratio);
        }
    }

    fn convert_x_to_screen(x: f32) -> f32 {
        x * 2.0 - 1.0
    }
    fn convert_y_to_screen(y: f32) -> f32 {
        y * -2.0 + 1.0
    }
    fn convert_x_from_screen(x: f32) -> f32 {
        x / 2.0 + 1.0
    }
    fn convert_y_from_screen(y: f32) -> f32 {
        y / -2.0 - 1.0
    }

    fn get_tile_colors(
        &self,
        x: i32,
        y: i32,
        left: f32,
        right: f32,
        top: f32,
        bottom: f32
    ) -> [f32; 16] {
        let lt_distance = get_distance(self.mouse_pos.0, self.mouse_pos.1, left, top);
        let lb_distance = get_distance(self.mouse_pos.0, self.mouse_pos.1, left, bottom);
        let rt_distance = get_distance(self.mouse_pos.0, self.mouse_pos.1, right, top);
        let rb_distance = get_distance(self.mouse_pos.0, self.mouse_pos.1, right, bottom);

        let lt_color: [f32; 4] = [
            (rt_distance - lt_distance) / get_difference(left, right) +
                (lb_distance - lt_distance) / get_difference(bottom, top),
            0.0,
            0.0,
            1.0,
        ];
        let lb_color: [f32; 4] = [
            (rb_distance - lb_distance) / get_difference(left, right) +
                (lt_distance - lb_distance) / get_difference(bottom, top),
            0.0,
            0.0,
            1.0,
        ];
        let rt_color: [f32; 4] = [
            (lt_distance - rt_distance) / get_difference(left, right) +
                (rb_distance - rt_distance) / get_difference(bottom, top),
            0.0,
            0.0,
            1.0,
        ];
        let rb_color: [f32; 4] = [
            (lb_distance - rb_distance) / get_difference(left, right) +
                (rt_distance - rb_distance) / get_difference(bottom, top),
            0.0,
            0.0,
            1.0,
        ];

        let mut result: [f32; 16] = [0.0; 16];

        for i in 0..4 {
            result[i + 0] = lt_color[i];
            result[i + 4] = lb_color[i];
            result[i + 8] = rt_color[i];
            result[i + 12] = rb_color[i];
        }

        // keep values between 0 and 1
        for i in 0..16 {
            if result[i] > 1.0 {
                result[i] = 1.0;
            } else if result[i] < 0.0 {
                result[i] = 0.0;
            }
        }

        result
    }
}

fn get_difference(a: f32, b: f32) -> f32 {
    (a - b).abs()
}

fn get_distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let x_diff = x2 - x1;
    let y_diff = y2 - y1;

    let squared_distance = (x_diff * x_diff + y_diff * y_diff) / 2.0;

    squared_distance.sqrt()
}

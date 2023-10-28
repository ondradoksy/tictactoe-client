use wasm_bindgen::prelude::*;
use web_sys::HtmlImageElement;
use web_sys::{ HtmlCanvasElement, WebGlRenderingContext, WebGlShader, WebGlProgram };
use std::convert::TryInto;
use std::convert::TryFrom;
use std::f32::consts::PI;
use webgl_matrix::{ Matrix, Vector, ProjectionMatrix, Mat4, Vec4, Mat3, Vec3, MulVectorMatrix };

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
    aspect_ratio: f32,
    grid_size: (i32, i32),
    image: HtmlImageElement,
    model_view_matrix: [f32; 16],
    projection_matrix: [f32; 16],
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
            aspect_ratio: 1.0,
            grid_size: (10, 10),
            image: image,
            model_view_matrix: Mat4::identity(),
            projection_matrix: Mat4::create_perspective(90.0, 1.0, 0.1, 100.0),
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

        uniform mat4 modelViewMatrix;
        uniform mat4 projectionMatrix;

        void main(void) {
            gl_Position = projectionMatrix * modelViewMatrix * vec4(coordinates, 1.0);
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
            gl_FragColor = mix(texture2D(uSampler, vTextureCoord), vColor, vColor.a);
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

    fn setup_vertices(&self, vertices: &[f32]) {
        let vertices_array = unsafe { js_sys::Float32Array::view(&vertices) };
        let vertex_buffer = self.gl.create_buffer().unwrap();

        self.gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
        self.gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &vertices_array,
            WebGlRenderingContext::STATIC_DRAW
        );

        let coordinates_location = self.gl.get_attrib_location(&self.shader_program, "coordinates");

        self.gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
        self.gl.vertex_attrib_pointer_with_i32(
            coordinates_location as u32,
            3,
            WebGlRenderingContext::FLOAT,
            false,
            0,
            0
        );

        self.gl.enable_vertex_attrib_array(coordinates_location as u32);
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

    fn setup_indices(&mut self, indices: &[u16]) {
        let indices_array = unsafe { js_sys::Uint16Array::view(&indices) };
        let index_buffer = self.gl.create_buffer().unwrap();

        self.gl.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
        self.gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
            &indices_array,
            WebGlRenderingContext::STATIC_DRAW
        );
    }

    fn setup_3d(&mut self) {
        let model_view_matrix_location = self.gl
            .get_uniform_location(&self.shader_program, "modelViewMatrix")
            .unwrap();
        let projection_matrix_location = self.gl
            .get_uniform_location(&self.shader_program, "projectionMatrix")
            .unwrap();

        self.gl.uniform_matrix4fv_with_f32_array(
            Some(&model_view_matrix_location),
            false,
            &self.model_view_matrix
        );
        self.gl.uniform_matrix4fv_with_f32_array(
            Some(&projection_matrix_location),
            false,
            &self.projection_matrix
        );
    }

    pub fn draw_grid(&mut self) {
        let mut vertices: Vec<f32> = Vec::with_capacity(
            (self.grid_size.0 * self.grid_size.1 * 4 * 3) as usize
        );
        let mut colors: Vec<f32> = Vec::with_capacity(
            (self.grid_size.0 * self.grid_size.1 * 4 * 4) as usize
        );
        let mut indices: Vec<u16> = Vec::with_capacity(
            (self.grid_size.0 * self.grid_size.1 * 6) as usize
        );

        let width: f32 = i32::try_from(self.grid_size.0).unwrap() as f32;
        let height: f32 = i32::try_from(self.grid_size.1).unwrap() as f32;

        let tile_width: f32 = 1.0 / width;
        let tile_height: f32 = 1.0 / height;

        let mut texture_coords: Vec<f32> = Vec::with_capacity(
            (self.grid_size.0 * self.grid_size.1 * 6 * 2) as usize
        );

        self.model_view_matrix = Mat4::identity();
        self.projection_matrix = Mat4::create_perspective(90.0, self.aspect_ratio, 0.1, 100.0);
        self.model_view_matrix[14] = -1.0;
        self.model_view_matrix.rotate(PI / 16.0, &[0.0, 1.0, 0.0]);

        self.setup_3d();

        Mat4::create_perspective_from_viewport(-1.0, 1.0, -1.0, 1.0, 0.1, 100.0);

        // background
        {
            let index_start = vertices.len() / 3;
            indices.push((index_start + 0) as u16);
            indices.push((index_start + 1) as u16);
            indices.push((index_start + 2) as u16);

            indices.push((index_start + 2) as u16);
            indices.push((index_start + 3) as u16);
            indices.push((index_start + 1) as u16);

            vertices.push(-1.0);
            vertices.push(1.0);
            vertices.push(0.0);

            vertices.push(-1.0);
            vertices.push(-1.0);
            vertices.push(0.0);

            vertices.push(1.0);
            vertices.push(1.0);
            vertices.push(0.0);

            vertices.push(1.0);
            vertices.push(-1.0);
            vertices.push(0.0);

            colors.extend_from_slice(&[0.0, 0.0, 0.0, 1.0]);
            colors.extend_from_slice(&[0.0, 0.0, 0.0, 1.0]);
            colors.extend_from_slice(&[0.0, 0.0, 0.0, 1.0]);
            colors.extend_from_slice(&[0.0, 0.0, 0.0, 1.0]);

            texture_coords.push(0.0);
            texture_coords.push(0.0);

            texture_coords.push(0.0);
            texture_coords.push(0.0);

            texture_coords.push(0.0);
            texture_coords.push(0.0);

            texture_coords.push(0.0);
            texture_coords.push(0.0);
        }

        // Y
        for i in 0..self.grid_size.1 {
            // X
            for j in 0..self.grid_size.0 {
                let left_relative: f32 = (j as f32) * tile_width + self.tile_spacing * tile_width;
                let right_relative: f32 =
                    ((j + 1) as f32) * tile_width - self.tile_spacing * tile_width;
                let top_relative: f32 = (i as f32) * tile_height + self.tile_spacing * tile_height;
                let bottom_relative: f32 =
                    ((i + 1) as f32) * tile_height - self.tile_spacing * tile_height;

                let left: f32 = Game::convert_x_to_screen(left_relative);
                let right: f32 = Game::convert_x_to_screen(right_relative);
                let top: f32 = Game::convert_y_to_screen(top_relative);
                let bottom: f32 = Game::convert_y_to_screen(bottom_relative);

                let index_start = vertices.len() / 3;
                // Triangle 1
                indices.push((index_start + 0) as u16);
                indices.push((index_start + 1) as u16);
                indices.push((index_start + 2) as u16);

                // Triangle 2
                indices.push((index_start + 2) as u16);
                indices.push((index_start + 3) as u16);
                indices.push((index_start + 0) as u16);

                // Vertices
                vertices.push(right); // X
                vertices.push(top); // Y
                vertices.push(((j as f32) % 2.0) * 0.05); // Z

                vertices.push(left); // X
                vertices.push(top); // Y
                vertices.push(((j as f32) % 2.0) * 0.05); // Z

                vertices.push(left); // X
                vertices.push(bottom); // Y
                vertices.push(((j as f32) % 2.0) * 0.05); // Z

                vertices.push(right); // X
                vertices.push(bottom); // Y
                vertices.push(((j as f32) % 2.0) * 0.05); // Z

                // Colors
                let vertices_index: usize = vertices.len() - 12;

                let tile_colors = self.get_tile_colors(
                    j as i32,
                    i as i32,
                    &vertices[vertices_index + 0..vertices_index + 3].try_into().unwrap(),
                    &vertices[vertices_index + 3..vertices_index + 6].try_into().unwrap(),
                    &vertices[vertices_index + 6..vertices_index + 9].try_into().unwrap(),
                    &vertices[vertices_index + 9..vertices_index + 12].try_into().unwrap()
                );

                let mut iter = tile_colors.chunks_exact(4);

                let lt_color: &[f32] = iter.next().unwrap();
                let lb_color: &[f32] = iter.next().unwrap();
                let rt_color: &[f32] = iter.next().unwrap();
                let rb_color: &[f32] = iter.next().unwrap();

                colors.extend_from_slice(&rt_color);
                colors.extend_from_slice(&lt_color);
                colors.extend_from_slice(&lb_color);
                colors.extend_from_slice(&rb_color);

                // Textures
                texture_coords.push(1.0);
                texture_coords.push(0.0);

                texture_coords.push(0.0);
                texture_coords.push(0.0);

                texture_coords.push(0.0);
                texture_coords.push(1.0);

                texture_coords.push(1.0);
                texture_coords.push(1.0);
            }
        }

        let _texture = self.setup_texture(&texture_coords);

        self.setup_indices(&indices);
        self.setup_vertices(&vertices);
        Game::setup_colors(&self.gl, &colors, &self.shader_program);

        //self.gl.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, (vertices.len() / 3) as i32);
        self.gl.draw_elements_with_i32(
            WebGlRenderingContext::TRIANGLES,
            indices.len() as i32,
            WebGlRenderingContext::UNSIGNED_SHORT,
            0
        );
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

        self.aspect_ratio = x_ratio / y_ratio;
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
        tr: &[f32; 3],
        tl: &[f32; 3],
        bl: &[f32; 3],
        br: &[f32; 3]
    ) -> [f32; 16] {
        let mut tl_pos = [tl[0], tl[1], tl[2], 1.0];
        let mut bl_pos = [bl[0], bl[1], bl[2], 1.0];
        let mut tr_pos = [tr[0], tr[1], tr[2], 1.0];
        let mut br_pos = [br[0], br[1], br[2], 1.0];

        tl_pos = tl_pos.mul_matrix(&self.model_view_matrix).mul_matrix(&self.projection_matrix);
        bl_pos = bl_pos.mul_matrix(&self.model_view_matrix).mul_matrix(&self.projection_matrix);
        tr_pos = tr_pos.mul_matrix(&self.model_view_matrix).mul_matrix(&self.projection_matrix);
        br_pos = br_pos.mul_matrix(&self.model_view_matrix).mul_matrix(&self.projection_matrix);

        tr_pos[0] = tr_pos[0] / tr_pos[3];
        tr_pos[1] = tr_pos[1] / tr_pos[3];
        tl_pos[0] = tl_pos[0] / tl_pos[3];
        tl_pos[1] = tl_pos[1] / tl_pos[3];
        bl_pos[0] = bl_pos[0] / bl_pos[3];
        bl_pos[1] = bl_pos[1] / bl_pos[3];
        br_pos[0] = br_pos[0] / br_pos[3];
        br_pos[1] = br_pos[1] / br_pos[3];

        let mut lt_color: [f32; 4] = [1.0, 0.0, 0.0, 0.5];
        let mut lb_color: [f32; 4] = [0.0, 1.0, 0.0, 0.5];
        let mut rt_color: [f32; 4] = [0.0, 0.0, 1.0, 0.5];
        let mut rb_color: [f32; 4] = [0.0, 0.0, 0.0, 0.5];

        let mut mouse_clip_space = [
            Game::convert_x_to_screen(self.mouse_pos.0),
            Game::convert_y_to_screen(self.mouse_pos.1),
            0.0,
            1.0,
        ];
        mouse_clip_space = mouse_clip_space
            .mul_matrix(&self.model_view_matrix)
            .mul_matrix(&self.projection_matrix);

        if
            point_in_polygon(
                Game::convert_x_to_screen(self.mouse_pos.0),
                Game::convert_y_to_screen(self.mouse_pos.1),
                [
                    (tl_pos[0], tl_pos[1]),
                    (bl_pos[0], bl_pos[1]),
                    (br_pos[0], br_pos[1]),
                    (tr_pos[0], tr_pos[1]),
                ]
            )
        {
            lt_color = [1.0, 1.0, 1.0, 1.0];
            lb_color = [1.0, 1.0, 1.0, 1.0];
            rt_color = [1.0, 1.0, 1.0, 1.0];
            rb_color = [1.0, 1.0, 1.0, 1.0];
        }

        if x == 9 && y == 0 {
            log!(
                "{:?}\n {:?}\n {:?}\n {:?}\n {:?} {:?}",
                tl_pos,
                bl_pos,
                br_pos,
                tr_pos,
                Game::convert_x_to_screen(self.mouse_pos.0),
                Game::convert_y_to_screen(self.mouse_pos.1)
            );
        }

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

pub fn point_in_polygon(x: f32, y: f32, points: [(f32, f32); 4]) -> bool {
    let mut intersections = 0;

    for i in 0..4 {
        let v1 = points[i];
        let j = (i + 1) % 4;
        let v2 = points[j];

        if are_intersecting(v1.0, v1.1, v2.0, v2.1, x + 10.0, y, x, y) {
            intersections += 1;
        }
    }

    intersections % 2 == 1
}

fn is_in_range(n: f32, a: f32, b: f32) -> bool {
    (a < n && n < b) || (a > n && n > b)
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

pub fn are_intersecting(
    v1x1: f32,
    v1y1: f32,
    v1x2: f32,
    v1y2: f32,
    v2x1: f32,
    v2y1: f32,
    v2x2: f32,
    v2y2: f32
) -> bool {
    let mut d1: f32;
    let mut d2: f32;
    let a1: f32;
    let a2: f32;
    let b1: f32;
    let b2: f32;
    let c1: f32;
    let c2: f32;

    let mut result: bool = true;

    // Convert vector 1 to a line (line 1) of infinite length.
    // We want the line in linear equation standard form: A*x + B*y + C = 0
    // See: http://en.wikipedia.org/wiki/Linear_equation
    a1 = v1y2 - v1y1;
    b1 = v1x1 - v1x2;
    c1 = v1x2 * v1y1 - v1x1 * v1y2;

    // Every point (x,y), that solves the equation above, is on the line,
    // every point that does not solve it, is not. The equation will have a
    // positive result if it is on one side of the line and a negative one
    // if is on the other side of it. We insert (x1,y1) and (x2,y2) of vector
    // 2 into the equation above.
    d1 = a1 * v2x1 + b1 * v2y1 + c1;
    d2 = a1 * v2x2 + b1 * v2y2 + c1;

    // If d1 and d2 both have the same sign, they are both on the same side
    // of our line 1 and in that case no intersection is possible. Careful,
    // 0 is a special case, that's why we don't test ">=" and "<=",
    // but "<" and ">".
    if d1 > 0.0 && d2 > 0.0 {
        result = false;
    }
    if d1 < 0.0 && d2 < 0.0 {
        result = false;
    }

    // The fact that vector 2 intersected the infinite line 1 above doesn't
    // mean it also intersects the vector 1. Vector 1 is only a subset of that
    // infinite line 1, so it may have intersected that line before the vector
    // started or after it ended. To know for sure, we have to repeat the
    // the same test the other way round. We start by calculating the
    // infinite line 2 in linear equation standard form.
    a2 = v2y2 - v2y1;
    b2 = v2x1 - v2x2;
    c2 = v2x2 * v2y1 - v2x1 * v2y2;

    // Calculate d1 and d2 again, this time using points of vector 1.
    d1 = a2 * v1x1 + b2 * v1y1 + c2;
    d2 = a2 * v1x2 + b2 * v1y2 + c2;

    // Again, if both have the same sign (and neither one is 0),
    // no intersection is possible.
    if d1 > 0.0 && d2 > 0.0 {
        result = false;
    }
    if d1 < 0.0 && d2 < 0.0 {
        result = false;
    }

    // If we get here, only two possibilities are left. Either the two
    // vectors intersect in exactly one point or they are collinear, which
    // means they intersect in any number of points from zero to infinite.
    if (a1 * b2 - a2 * b1).abs() <= 0.000001 {
        //result = false;
    }

    // If they are not collinear, they must intersect in exactly one point.
    result
}

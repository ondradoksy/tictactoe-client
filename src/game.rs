use wasm_bindgen::prelude::*;
use web_sys::{ HtmlCanvasElement, WebGlBuffer, MouseEvent, WheelEvent, WebSocket };
use web_sys::{ HtmlImageElement, WebGl2RenderingContext, WebGlShader, WebGlProgram };
use std::cell::RefCell;
use std::collections::VecDeque;
use std::convert::TryInto;
use std::convert::TryFrom;
use std::f32::consts::PI;
use std::rc::Rc;
use webgl_matrix::{ Matrix, ProjectionMatrix, Mat4, MulVectorMatrix };
use crate::gameobject::GameObject;
use crate::grid::Grid;
use crate::mouse::{ MouseTracker, FloatPos };
use crate::net::send;
use crate::player::Player;
use crate::playermove::PlayerMove;
use crate::texture::Texture;
use crate::utils::{ now, Size };
pub use crate::log;

pub struct Game {
    frames: i64,
    gl: WebGl2RenderingContext,
    shader_program: WebGlProgram,
    tile_scale: f32,
    aspect_ratio: f32,
    grid: Grid,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    last_time: f64,
    texture_indices: Vec<usize>,
    frame_times: VecDeque<f64>,
    model_buffer: Option<WebGlBuffer>,
    hover_tile: Option<Size>,
    mouse_tracker: MouseTracker,
    ws: WebSocket,
    players: Rc<RefCell<Vec<Player>>>,
    textures: Vec<Texture>,
}
impl Game {
    pub(crate) fn new(
        canvas_id: &str,
        grid: Grid,
        ws: &WebSocket,
        players: &Rc<RefCell<Vec<Player>>>
    ) -> Game {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id(canvas_id).unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();

        let gl: WebGl2RenderingContext = Game::init_webgl_context(&canvas);
        let shader_program: WebGlProgram = Game::setup_shaders(&gl).unwrap();

        let mut instance = Self {
            frames: 0,
            gl: gl,
            shader_program: shader_program,
            tile_scale: 0.8,
            aspect_ratio: 1.0,
            grid: grid,
            view_matrix: Mat4::identity(),
            projection_matrix: Mat4::create_perspective(PI / 2.0, 1.0, 0.1, 100.0),
            last_time: now(),
            texture_indices: Vec::new(),
            frame_times: VecDeque::new(),
            model_buffer: None,
            hover_tile: None,
            mouse_tracker: MouseTracker::new(),
            ws: ws.clone(),
            players: players.clone(),
            textures: Vec::from([
                Texture::from_url(-1, "empty.png"),
                Texture::from_url(-2, "blocked.png"),
                Texture::from_url(-3, "unknown.png"),
            ]),
        };

        instance.init();

        instance
    }

    pub fn render(&mut self) {
        let start = now();
        self.clear();
        self.update_viewport();

        self.draw_grid();

        self.frames += 1;

        let time = now() - start;
        self.frame_times.push_back(time);
        if self.frame_times.len() > 100 {
            self.frame_times.pop_front();
        }

        if self.frames % 100 == 0 {
            log!(
                "Last 100 frames: {:?}, FPS: {:?}",
                self.frames,
                (1000.0 / (now() - self.last_time)) * 100.0
            );
            self.last_time = now();
            log!(
                "Frame times: AVG={:?}ms, MAX={:?}ms, MIN={:?}ms",
                self.frame_times.iter().sum::<f64>() / 100.0,
                self.frame_times.iter().copied().fold(f64::NAN, f64::max),
                self.frame_times.iter().copied().fold(f64::NAN, f64::min)
            );
        }
    }

    pub fn init_webgl_context(canvas: &HtmlCanvasElement) -> WebGl2RenderingContext {
        let gl: WebGl2RenderingContext = canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()
            .unwrap();

        gl.viewport(0, 0, canvas.width().try_into().unwrap(), canvas.height().try_into().unwrap());

        gl.enable(WebGl2RenderingContext::DEPTH_TEST);

        gl
    }

    pub fn init(&mut self) {
        self.view_matrix[14] = -2.0; // Default zoom
        self.init_texture_indices();
    }

    fn init_texture_indices(&mut self) {
        self.texture_indices = Vec::with_capacity((self.grid.size.x * self.grid.size.y) as usize);

        // Y
        for i in 0..self.grid.size.y {
            // X
            for j in 0..self.grid.size.x {
                let index = self.grid.get_pos(&Size::new(j, i));
                if index.is_some() {
                    self.texture_indices.push((index.unwrap() + 1) as usize);
                } else {
                    self.texture_indices.push(0);
                }
            }
        }
    }

    pub fn create_shader(
        gl: &WebGl2RenderingContext,
        shader_type: u32,
        source: &str
    ) -> WebGlShader {
        let shader = gl.create_shader(shader_type).expect("Unable to create shader object");

        gl.shader_source(&shader, source);
        gl.compile_shader(&shader);

        if
            gl
                .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
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

    pub fn setup_shaders(gl: &WebGl2RenderingContext) -> Result<WebGlProgram, JsValue> {
        let vertex_shader_source =
            "#version 300 es
        in vec3 coordinates;
        in vec4 vertexColor;
        in vec2 aTextureCoord;
        
        out lowp vec4 vColor;
        out highp vec2 vTextureCoord;
        
        uniform mat4 viewMatrix;
        uniform mat4 projectionMatrix;
        
        in mat4 modelMatrix;

        void main(void) {
            gl_Position = projectionMatrix * viewMatrix * modelMatrix * vec4(coordinates, 1.0);
            vColor = vertexColor;
            vTextureCoord = aTextureCoord;
        }
        ";

        let fragment_shader_source =
            "#version 300 es
        precision mediump float;
        
        in lowp vec4 vColor;
        in highp vec2 vTextureCoord;

        out vec4 outColor;

        uniform sampler2D uSampler;

        void main(void) {
            outColor = mix(texture(uSampler, vTextureCoord), vColor, vColor.a);
        }
        "; // uniform vec4

        let vertex_shader = Game::create_shader(
            gl,
            WebGl2RenderingContext::VERTEX_SHADER,
            vertex_shader_source
        );
        let fragment_shader = Game::create_shader(
            gl,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            fragment_shader_source
        );

        let shader_program = gl.create_program().unwrap();
        gl.attach_shader(&shader_program, &vertex_shader);
        gl.attach_shader(&shader_program, &fragment_shader);
        gl.link_program(&shader_program);

        if
            gl
                .get_program_parameter(&shader_program, WebGl2RenderingContext::LINK_STATUS)
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

        self.gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
        self.gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &vertices_array,
            WebGl2RenderingContext::STATIC_DRAW
        );

        let coordinates_location = self.gl.get_attrib_location(&self.shader_program, "coordinates");

        //self.gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
        self.gl.vertex_attrib_pointer_with_i32(
            coordinates_location as u32,
            3,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0
        );

        self.gl.enable_vertex_attrib_array(coordinates_location as u32);
    }

    pub fn setup_colors(
        gl: &WebGl2RenderingContext,
        colors: &[f32],
        shader_program: &WebGlProgram
    ) {
        let color_array = unsafe { js_sys::Float32Array::view(&colors) };
        let color_buffer = gl.create_buffer().unwrap();

        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&color_buffer));
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &color_array,
            WebGl2RenderingContext::STATIC_DRAW
        );

        let colors_location = gl.get_attrib_location(&shader_program, "vertexColor");

        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&color_buffer));
        gl.vertex_attrib_pointer_with_i32(
            colors_location as u32,
            4,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0
        );
        gl.enable_vertex_attrib_array(colors_location as u32);
    }

    fn setup_texture(
        &mut self,
        texture_coords: &[f32],
        texture_image: HtmlImageElement
    ) -> web_sys::WebGlTexture {
        let texture_coords_array = unsafe { js_sys::Float32Array::view(&texture_coords) };
        let texture_coords_buffer = self.gl.create_buffer().unwrap();
        self.gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&texture_coords_buffer));
        self.gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &texture_coords_array,
            WebGl2RenderingContext::STATIC_DRAW
        );

        let sampler_location = self.gl
            .get_uniform_location(&self.shader_program, "uSampler")
            .unwrap();

        let texture = self.gl.create_texture().unwrap();
        self.gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

        let _ = self.gl.tex_image_2d_with_u32_and_u32_and_html_image_element(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            WebGl2RenderingContext::RGB.try_into().unwrap(),
            WebGl2RenderingContext::RGB,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            &texture_image
        );

        let texture_coord_location = self.gl.get_attrib_location(
            &self.shader_program,
            "aTextureCoord"
        );

        self.gl.uniform1i(Some(&sampler_location), 0);
        self.gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::NEAREST as i32
        );
        self.gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::LINEAR as i32
        );
        self.gl.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);

        self.gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&texture_coords_buffer));
        self.gl.vertex_attrib_pointer_with_i32(
            texture_coord_location as u32,
            2,
            WebGl2RenderingContext::FLOAT,
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

        self.gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
        self.gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            &indices_array,
            WebGl2RenderingContext::STATIC_DRAW
        );
    }

    fn setup_3d(&mut self) {
        let view_matrix_location = self.gl
            .get_uniform_location(&self.shader_program, "viewMatrix")
            .unwrap();
        let projection_matrix_location = self.gl
            .get_uniform_location(&self.shader_program, "projectionMatrix")
            .unwrap();

        self.gl.uniform_matrix4fv_with_f32_array(
            Some(&view_matrix_location),
            false,
            &self.view_matrix
        );
        self.gl.uniform_matrix4fv_with_f32_array(
            Some(&projection_matrix_location),
            false,
            &self.projection_matrix
        );
    }

    fn setup_models(&mut self, models: &[f32]) {
        if self.model_buffer.is_none() {
            self.model_buffer = self.gl.create_buffer();
        }
        self.update_models(models);
    }

    fn update_models(&mut self, models: &[f32]) {
        let models_array = unsafe { js_sys::Float32Array::view(&models) };

        self.gl.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&self.model_buffer.as_ref().unwrap())
        );
        self.gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &models_array,
            WebGl2RenderingContext::DYNAMIC_DRAW
        );

        let model_matrix_location = self.gl.get_attrib_location(
            &self.shader_program,
            "modelMatrix"
        );

        for i in 0..4 {
            self.gl.enable_vertex_attrib_array((model_matrix_location as u32) + i);
            self.gl.vertex_attrib_pointer_with_i32(
                (model_matrix_location as u32) + i,
                4,
                WebGl2RenderingContext::FLOAT,
                false,
                64,
                (i as i32) * 16
            );
            self.gl.vertex_attrib_divisor((model_matrix_location as u32) + i, 1);
        }
    }

    pub fn draw_grid(&mut self) {
        {
            // Setup textures
            for player in self.players.borrow_mut().iter_mut() {
                let index = self.textures
                    .iter()
                    .position(|p| p.id == player.id.try_into().unwrap());
                if index != None {
                    self.textures[index.unwrap()].image = player.get_image(&self.ws);
                    continue;
                }
                let image = player.get_image(&self.ws);
                self.textures.push(Texture::new(player.id.try_into().unwrap(), image));
            }
        }

        let width: f32 = i32::try_from(self.grid.size.x).unwrap() as f32;
        let height: f32 = i32::try_from(self.grid.size.y).unwrap() as f32;

        let tile_width: f32 = 1.0 / width;
        let tile_height: f32 = 1.0 / height;

        self.projection_matrix = Mat4::create_perspective(PI / 2.0, self.aspect_ratio, 0.1, 100.0);

        let vao = self.gl.create_vertex_array().unwrap();
        self.gl.bind_vertex_array(Some(&vao));

        self.setup_3d();

        let mut game_objects: Vec<GameObject> = Vec::new();

        // background
        {
            game_objects.push(
                GameObject::new(
                    Mat4::identity(),
                    Vec::from([0, 1, 2, 2, 3, 1]),
                    Vec::from([-1.0, 1.0, 0.0, -1.0, -1.0, 0.0, 1.0, 1.0, 0.0, 1.0, -1.0, 0.0]),
                    Vec::from([
                        0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0,
                    ]),
                    Vec::from([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
                    -1
                )
            );
        }

        let scale = if self.grid.size.x > self.grid.size.y {
            1.0 / (self.grid.size.x as f32)
        } else {
            1.0 / (self.grid.size.y as f32)
        };
        let tile_size = if tile_width > tile_height { tile_height } else { tile_width };

        self.hover_tile = None;

        // Y
        for i in 0..self.grid.size.y {
            // X
            for j in 0..self.grid.size.x {
                let origin_x = Game::convert_x_to_screen((j as f32) * tile_size + tile_size / 2.0);
                let origin_y = Game::convert_y_to_screen((i as f32) * tile_size + tile_size / 2.0);

                // Model
                let mut model_matrix = Mat4::identity();
                model_matrix[0] = scale * self.tile_scale;
                model_matrix[5] = scale * self.tile_scale;
                model_matrix[10] = scale * self.tile_scale;

                model_matrix[12] = origin_x;
                model_matrix[13] = origin_y;
                model_matrix[14] = 0.1;

                let screen_pos = get_pos_center(&self.get_tile_pos_on_screen(&model_matrix));

                let rotation_multiplier = 0.1;

                model_matrix.rotate(
                    rotation_multiplier *
                        PI *
                        (Game::convert_x_to_screen(self.mouse_tracker.get_current_pos().x) -
                            screen_pos[0]),
                    &[0.0, 1.0, 0.0]
                );
                model_matrix.rotate(
                    -rotation_multiplier *
                        PI *
                        (Game::convert_y_to_screen(self.mouse_tracker.get_current_pos().y) -
                            screen_pos[1]),
                    &[1.0, 0.0, 0.0]
                );

                let tile_colors = self.get_tile_colors(j, i, &model_matrix);

                let texture_id = self.grid.get_pos(&Size::new(j, i));

                game_objects.push(
                    GameObject::new_tile(model_matrix, tile_colors.to_vec(), if
                        texture_id.is_some()
                    {
                        texture_id.unwrap().try_into().unwrap()
                    } else {
                        -1
                    })
                );
            }
        }

        if self.frames % 100 == 0 {
            log!("Rendering: {:?} objects", game_objects.len());
        }

        let mut render_iter = 0;
        game_objects.sort_by(|a, b| a.texture_id.cmp(&b.texture_id));
        while render_iter < game_objects.len() {
            let cur = &game_objects[render_iter];
            let mut batch_size: usize = 1;
            let mut models_matrices: Vec<f32> = Vec::from(cur.model_matrix);

            for i in render_iter + 1..game_objects.len() {
                if
                    cur.texture_id != game_objects[i].texture_id ||
                    cur.colors != game_objects[i].colors
                {
                    break;
                }
                batch_size += 1;
                models_matrices.extend(game_objects[i].model_matrix.iter());
            }

            self.setup_vertices(&cur.vertices);

            Game::setup_colors(&self.gl, &cur.colors, &self.shader_program);

            self.setup_indices(&cur.indices);

            self.setup_models(&models_matrices);

            let texture_index = self.textures
                .iter()
                .position(|p| p.id == cur.texture_id)
                .unwrap_or(2);

            let _texture = self.setup_texture(
                &cur.texture_coords,
                self.textures[texture_index].image.clone()
            );

            self.gl.draw_elements_instanced_with_i32(
                WebGl2RenderingContext::TRIANGLES,
                6,
                WebGl2RenderingContext::UNSIGNED_SHORT,
                0,
                batch_size.try_into().unwrap()
            );

            render_iter += batch_size;
        }
    }

    pub fn on_mouse_move(&mut self, e: MouseEvent) {
        let last_pos = self.mouse_tracker.get_current_pos();
        self.update_mouse_pos(&e);
        if self.mouse_tracker.is_down(0) {
            let diff = last_pos - self.mouse_tracker.get_current_pos();
            self.view_matrix[12] -= diff.x;
            self.view_matrix[13] += diff.y;
        }
    }

    fn clear(&self) {
        self.gl.clear_color(0.1f32, 0.1f32, 0.1f32, 1f32);
        self.gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
    }

    fn update_viewport(&mut self) {
        self.gl.viewport(0, 0, self.gl.drawing_buffer_width(), self.gl.drawing_buffer_height());

        let x_ratio = self.gl.drawing_buffer_width() as f32;
        let y_ratio = self.gl.drawing_buffer_height() as f32;

        self.aspect_ratio = x_ratio / y_ratio;
    }

    fn convert_x_to_screen(x: f32) -> f32 {
        x * 2.0 - 1.0
    }
    fn convert_y_to_screen(y: f32) -> f32 {
        y * -2.0 + 1.0
    }

    fn get_tile_colors(&mut self, x: i32, y: i32, model_matrix: &Mat4) -> [f32; 16] {
        let mut lt_color: [f32; 4] = [1.0, 0.0, 0.0, 0.1];
        let mut lb_color: [f32; 4] = [0.0, 1.0, 0.0, 0.1];
        let mut rt_color: [f32; 4] = [0.0, 0.0, 1.0, 0.1];
        let mut rb_color: [f32; 4] = [0.0, 0.0, 0.0, 0.1];

        if self.hover_tile == None {
            let screen_pos = self.get_tile_pos_on_screen(&model_matrix);
            if
                point_in_polygon(
                    Game::convert_x_to_screen(self.mouse_tracker.get_current_pos().x),
                    Game::convert_y_to_screen(self.mouse_tracker.get_current_pos().y),
                    [
                        (screen_pos[2], screen_pos[3]),
                        (screen_pos[4], screen_pos[5]),
                        (screen_pos[6], screen_pos[7]),
                        (screen_pos[0], screen_pos[1]),
                    ]
                )
            {
                self.hover_tile = Some(Size::new(x, y));
            }
        }

        if self.hover_tile == Some(Size::new(x, y)) {
            lt_color = [1.0, 1.0, 1.0, 0.8];
            lb_color = [1.0, 1.0, 1.0, 0.8];
            rt_color = [1.0, 1.0, 1.0, 0.8];
            rb_color = [1.0, 1.0, 1.0, 0.8];
        }

        let mut result: [f32; 16] = [0.0; 16];

        for i in 0..4 {
            result[i + 0] = rt_color[i];
            result[i + 4] = lt_color[i];
            result[i + 8] = lb_color[i];
            result[i + 12] = rb_color[i];
        }

        result
    }

    fn get_tile_pos_on_screen(&self, model_matrix: &Mat4) -> [f32; 8] {
        let left: f32 = -1.0;
        let right: f32 = 1.0;
        let top: f32 = 1.0;
        let bottom: f32 = -1.0;

        let tr = [right, top, 0.0];
        let tl = [left, top, 0.0];
        let bl = [left, bottom, 0.0];
        let br = [right, bottom, 0.0];

        let mut tl_pos = [tl[0], tl[1], tl[2], 1.0];
        let mut bl_pos = [bl[0], bl[1], bl[2], 1.0];
        let mut tr_pos = [tr[0], tr[1], tr[2], 1.0];
        let mut br_pos = [br[0], br[1], br[2], 1.0];

        tl_pos = tl_pos
            .mul_matrix(&model_matrix)
            .mul_matrix(&self.view_matrix)
            .mul_matrix(&self.projection_matrix);
        bl_pos = bl_pos
            .mul_matrix(&model_matrix)
            .mul_matrix(&self.view_matrix)
            .mul_matrix(&self.projection_matrix);
        tr_pos = tr_pos
            .mul_matrix(&model_matrix)
            .mul_matrix(&self.view_matrix)
            .mul_matrix(&self.projection_matrix);
        br_pos = br_pos
            .mul_matrix(&model_matrix)
            .mul_matrix(&self.view_matrix)
            .mul_matrix(&self.projection_matrix);

        tr_pos[0] = tr_pos[0] / tr_pos[3];
        tr_pos[1] = tr_pos[1] / tr_pos[3];
        tl_pos[0] = tl_pos[0] / tl_pos[3];
        tl_pos[1] = tl_pos[1] / tl_pos[3];
        bl_pos[0] = bl_pos[0] / bl_pos[3];
        bl_pos[1] = bl_pos[1] / bl_pos[3];
        br_pos[0] = br_pos[0] / br_pos[3];
        br_pos[1] = br_pos[1] / br_pos[3];

        [tr_pos[0], tr_pos[1], tl_pos[0], tl_pos[1], bl_pos[0], bl_pos[1], br_pos[0], br_pos[1]]
    }

    pub fn on_mouse_down(&mut self, e: MouseEvent) {
        log!("mousedown {:?}", e.button());
        self.update_mouse_pos(&e);
        match e.button() {
            0 => {
                // Left
                self.mouse_tracker.set_down(0);
            }
            2 => {
                // Right
                self.mouse_tracker.set_down(2);
            }
            _ => {}
        }
        e.prevent_default();
        e.stop_propagation();
    }
    pub fn on_mouse_up(&mut self, e: MouseEvent, ws: &WebSocket) {
        log!("mouseup {:?}", e.button());
        self.update_mouse_pos(&e);
        match e.button() {
            0 => {
                // Left
                let diff =
                    self.mouse_tracker.get_current_pos() - self.mouse_tracker.get_pos(0).unwrap();

                if self.mouse_tracker.get_time_held(0).unwrap() < 1000.0 && diff.abs().max() < 0.1 {
                    // Click
                    log!("Clicked on {:?}", self.hover_tile);
                    if self.hover_tile.is_some() {
                        let pos = self.hover_tile.unwrap();
                        if self.grid.is_valid_move(&pos) {
                            send(ws, "move", self.hover_tile.unwrap().to_json().as_str());
                            log!("Sent move");
                        }
                    }
                }
                self.mouse_tracker.set_up(0);
            }
            2 => {
                // Right
                self.mouse_tracker.set_up(2);
            }
            _ => {}
        }
        e.prevent_default();
        e.stop_propagation();
    }
    pub fn on_scroll(&mut self, e: WheelEvent) {
        self.mouse_tracker.set_current_pos(
            FloatPos::new(
                (e.client_x() as f32) / (self.gl.drawing_buffer_width() as f32),
                (e.client_y() as f32) / (self.gl.drawing_buffer_height() as f32)
            )
        );

        let direction: f32 = if e.delta_y() < 0.0 { 1.0 } else { -1.0 };
        log!("scroll {:?}", direction);
        self.view_matrix[14] += direction * 0.01;
        if self.view_matrix[14] > 0.01 {
            self.view_matrix[14] = 0.01;
        }
        e.prevent_default();
        e.stop_propagation();
    }
    fn update_mouse_pos(&mut self, e: &MouseEvent) {
        self.mouse_tracker.set_current_pos(
            FloatPos::new(
                (e.client_x() as f32) / (self.gl.drawing_buffer_width() as f32),
                (e.client_y() as f32) / (self.gl.drawing_buffer_height() as f32)
            )
        );
    }
    pub fn add_move(&mut self, m: PlayerMove) {
        self.grid.add(m);
        self.init_texture_indices();
    }
}

// tl, bl, tr, br
fn get_pos_center(tile_pos: &[f32; 8]) -> [f32; 2] {
    let top_center = tile_pos[0] + (tile_pos[4] - tile_pos[0]) / 2.0;
    let bottom_center = tile_pos[2] + (tile_pos[6] - tile_pos[2]) / 2.0;
    let x = (top_center + bottom_center) / 2.0;

    let left_center = tile_pos[1] + (tile_pos[3] - tile_pos[1]) / 2.0;
    let right_center = tile_pos[5] + (tile_pos[7] - tile_pos[5]) / 2.0;
    let y = (left_center + right_center) / 2.0;

    [x, y]
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

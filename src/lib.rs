mod utils;

use std::convert::TryInto;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlRenderingContext, WebGlShader, WebGlProgram};

extern crate js_sys;
extern crate web_sys;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

static mut FRAMES: i64 = 0;

#[wasm_bindgen(start)]
pub fn init() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}

extern crate console_error_panic_hook;
use std::panic;

#[wasm_bindgen]
pub fn render() {
    unsafe { 
        FRAMES += 1; 
        if (FRAMES % 100 == 0) {
            log!("FRAME: {:?}", FRAMES);
        }
    }
}

pub fn init_webgl_context(canvas_id: &str) -> Result<WebGlRenderingContext, JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id(canvas_id).unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    let gl: WebGlRenderingContext = canvas
        .get_context("webgl")?
        .unwrap()
        .dyn_into::<WebGlRenderingContext>()
        .unwrap();

    gl.viewport(
        0,
        0,
        canvas.width().try_into().unwrap(),
        canvas.height().try_into().unwrap(),
    );

    Ok(gl)
}

pub fn create_shader(
    gl: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, JsValue> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| JsValue::from_str("Unable to create shader object"))?;

    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(JsValue::from_str(
            &gl.get_shader_info_log(&shader)
                .unwrap_or_else(|| "Unknown error creating shader".into()),
        ))
    }
}

pub fn setup_shaders(gl: &WebGlRenderingContext) -> Result<WebGlProgram, JsValue> {
    let vertex_shader_source = "
        attribute vec3 coordinates;
        attribute vec4 vertexColor;

        varying lowp vec4 vColor;

        void main(void) {
            gl_Position = vec4(coordinates, 1.0);
            vColor = vertexColor;
        }
        ";

    let fragment_shader_source = "
        precision mediump float;
        varying lowp vec4 vColor;
        void main(void) {
            gl_FragColor = vColor;
        }
        "; // uniform vec4

    let vertex_shader = create_shader(
        &gl,
        WebGlRenderingContext::VERTEX_SHADER,
        vertex_shader_source,
    )
    .unwrap();
    let fragment_shader = create_shader(
        &gl,
        WebGlRenderingContext::FRAGMENT_SHADER,
        fragment_shader_source,
    )
    .unwrap();

    let shader_program = gl.create_program().unwrap();
    gl.attach_shader(&shader_program, &vertex_shader);
    gl.attach_shader(&shader_program, &fragment_shader);
    gl.link_program(&shader_program);

    if gl
        .get_program_parameter(&shader_program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        gl.use_program(Some(&shader_program));
        Ok(shader_program)
    } else {
        return Err(JsValue::from_str(
            &gl.get_program_info_log(&shader_program)
                .unwrap_or_else(|| "Unknown error linking program".into()),
        ));
    }
}

pub fn setup_vertices(gl: &WebGlRenderingContext, vertices: &[f32], shader_program: &WebGlProgram) {
    let vertices_array = unsafe { js_sys::Float32Array::view(&vertices) };
    let vertex_buffer = gl.create_buffer().unwrap();

    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
    gl.buffer_data_with_array_buffer_view(
        WebGlRenderingContext::ARRAY_BUFFER,
        &vertices_array,
        WebGlRenderingContext::STATIC_DRAW,
    );

    let coordinates_location = gl.get_attrib_location(&shader_program, "coordinates");

    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
    gl.vertex_attrib_pointer_with_i32(
        coordinates_location as u32,
        3,
        WebGlRenderingContext::FLOAT,
        false,
        0,
        0,
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
        WebGlRenderingContext::STATIC_DRAW,
    );

    let colors_location = gl.get_attrib_location(&shader_program, "vertexColor");

    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&color_buffer));
    gl.vertex_attrib_pointer_with_i32(
        colors_location as u32,
        4,
        WebGlRenderingContext::FLOAT,
        false,
        0,
        0,
    );
    gl.enable_vertex_attrib_array(colors_location as u32);
}

#[wasm_bindgen]
pub fn init_webgl(canvas_id: &str) -> Result<WebGlRenderingContext, JsValue> {
    let gl: WebGlRenderingContext = init_webgl_context(canvas_id).unwrap();
    
    Ok(gl)
}

#[wasm_bindgen]
pub fn init_shaders(gl: WebGlRenderingContext) -> Result<WebGlProgram, JsValue> {
    let shader_program: WebGlProgram = setup_shaders(&gl).unwrap();

    Ok(shader_program)
}

#[wasm_bindgen]
pub fn draw_triangle(
    gl: WebGlRenderingContext,
    shader_program: WebGlProgram,
    selected_color: Option<Vec<f32>>,
) -> Result<WebGlRenderingContext, JsValue> {

    let mut frames_safe: i64;
    unsafe {
        frames_safe = FRAMES;
    }
    
    let vertices: [f32; 9] = [
        ((frames_safe % 400) as f32 / 100f32 - 2f32).abs() - 1.0, 0.9, 0.0, // top
        -0.9, -0.9, 0.0, // bottom left
        0.9, -0.9, 0.0, // bottom right
    ];

    setup_vertices(&gl, &vertices, &shader_program);

    let colors: [f32; 12] = [
        ((frames_safe % 240) as f32 / 120f32 - 1f32).abs(), 0.0, 0.0, 1.0, // top
        0.0, ((frames_safe % 240) as f32 / 120f32 - 1f32).abs(), 0.0, 1.0, // bottom left
        0.0, 0.0, ((frames_safe % 240) as f32 / 120f32 - 1f32).abs(), 1.0, // bottom right
    ];

    setup_colors(&gl, &colors, &shader_program);


    gl.clear_color(0f32, 0f32, 0f32, 1f32);
    gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

    gl.draw_arrays(
        WebGlRenderingContext::TRIANGLES,
        0,
        (vertices.len() / 3) as i32,
    );

    Ok(gl)
}
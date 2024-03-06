pub(crate) struct GameObject {
    pub model_matrix: [f32; 16],
    pub indices: Vec<u16>,
    pub vertices: Vec<f32>,
    pub colors: Vec<f32>,
    pub texture_coords: Vec<f32>,
    pub texture_id: i32,
}
impl GameObject {
    pub fn new(
        model_matrix: [f32; 16],
        indices: Vec<u16>,
        vertices: Vec<f32>,
        colors: Vec<f32>,
        texture_coords: Vec<f32>,
        texture_id: i32
    ) -> Self {
        Self {
            model_matrix: model_matrix,
            indices: indices,
            vertices: vertices,
            colors: colors,
            texture_coords: texture_coords,
            texture_id: texture_id,
        }
    }
    pub fn new_tile(model_matrix: [f32; 16], colors: Vec<f32>, texture_id: i32) -> Self {
        let left: f32 = -1.0;
        let right: f32 = 1.0;
        let top: f32 = 1.0;
        let bottom: f32 = -1.0;

        let padding = 0.01;

        let texture_left = 0.0 + padding;
        let texture_right = 1.0 - padding;
        let texture_top = 0.0 + padding;
        let texture_bottom = 1.0 - padding;

        Self::new(
            model_matrix,
            Vec::from([0, 1, 2, 2, 3, 0]),
            Vec::from([right, top, 0.0, left, top, 0.0, left, bottom, 0.0, right, bottom, 0.0]),
            colors,
            Vec::from([
                texture_right,
                texture_top,
                texture_left,
                texture_top,
                texture_left,
                texture_bottom,
                texture_right,
                texture_bottom,
            ]),
            texture_id
        )
    }
}

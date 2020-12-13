
#[derive(Default, Debug, Clone, Copy)]
pub struct VertexParams {
    position: [f32; 3],
    offset: [f32; 3],
    scale: f32,
    color: [f32; 4],
}

vulkano::impl_vertex!(VertexParams, position, offset, scale, color);

impl VertexParams {
    pub fn new(vertex: Vertex, mutation: FigureMutation, base_color: [f32; 4]) -> Self {
        Self {
            position: vertex.position,
            offset: mutation.position_offset,
            scale: mutation.scale,
            color: base_color,
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct FigureMutation {
    position_offset: [f32; 3],
    scale: f32,
}

impl FigureMutation {
    pub fn new(position_offset: [f32; 3], scale: f32) -> Self {
        Self {
            position_offset,
            scale,
        }
    }
    pub fn unit() -> Self {
        Self::new([0.0, 0.0, 0.0], 1.0)
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Vertex {
    position: [f32; 3],
}

impl Vertex {
    pub fn new(position: [f32; 3]) -> Self {
        Self { position }
    }
}

#[derive(Debug, Clone)]
pub struct Figure {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub mutation: FigureMutation,
    pub base_color: [f32; 4],
}

impl Figure {
    pub fn new(
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        mutation: FigureMutation,
        base_color: [f32; 4],
    ) -> Self {
        Self {
            vertices,
            indices,
            mutation,
            base_color,
        }
    }

    pub fn unit_cube(mutation: FigureMutation) -> Self {
        let unit = 0.25;

        let color = [1.0, 0.0, 0.0, 1.0];

        //fron face dots. ccw from top left
        let a = Vertex::new([unit, unit, -unit]);
        let b = Vertex::new([unit, unit, unit]);
        let c = Vertex::new([unit, -unit, unit]);
        let d = Vertex::new([unit, -unit, -unit]);

        //rear face dots. ccw from top left
        let e = Vertex::new([-unit, unit, -unit]);
        let f = Vertex::new([-unit, unit, unit]);
        let j = Vertex::new([-unit, -unit, unit]);
        let h = Vertex::new([-unit, -unit, -unit]);

        // a, b, c,
        // a, d, c,
        // e, f, j,
        // e, h, j,
        // b, c, f,
        // f, j, c,
        // a, d, e,
        // e, d, h,

        Figure::new(
            //   0, 1, 2, 3, 4, 5, 6, 7
            vec![a, b, c, d, e, f, j, h],
            vec![
                0, 1, 2, 0, 3, 2, 4, 5, 6, 4, 7, 6, 1, 2, 5, 5, 6, 2, 0, 3, 4, 4, 3, 7,
            ],
            mutation,
            color,
        )
    }
}

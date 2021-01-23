#[derive(Default, Debug, Clone, Copy)]
pub struct PerVerexParams {
    pub in_pos: [f32; 4],
    pub in_uv: [f32; 2],
    pub in_color: [f32; 3],
    pub in_normal: [f32; 3],
    pub in_tangent: [f32; 3],
}
vulkano::impl_vertex!(
    PerVerexParams,
    in_pos,
    in_uv,
    in_color,
    in_normal,
    in_tangent
);

impl PerVerexParams {
    pub fn new(
        in_pos: [f32; 4],
        in_uv: [f32; 2],
        in_color: [f32; 3],
        in_normal: [f32; 3],
        in_tangent: [f32; 3],
    ) -> Self {
        PerVerexParams {
            in_pos,
            in_uv,
            in_color,
            in_normal,
            in_tangent,
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

#[derive(Debug, Clone)]
pub enum RenderableMesh {
    Indexed(IndexedMesh),
    Regular(RegularMesh),
}

#[derive(Debug, Clone)]
pub struct MeshPoint {
    pub vert: [f32; 3],
    pub color: [f32; 3],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
}

impl MeshPoint {
    pub fn new(vert: [f32; 3], color: [f32; 3], normal: [f32; 3], tangent: [f32; 3]) -> Self {
        MeshPoint {
            vert: [vert[0], vert[1], vert[2]],
            color: [color[0], color[1], color[2]],
            normal: [normal[0], normal[1], normal[2]],
            tangent: [tangent[0], tangent[1], tangent[2]],
        }
    }

    pub fn to_vert(&self) -> PerVerexParams {
        let p = self;
        PerVerexParams::new(
            [p.vert[0], p.vert[1], p.vert[2], 1.0],
            [0.0, 0.0],
            [p.color[0], p.color[1], p.color[2]],
            [p.normal[0], p.normal[1], p.normal[2]],
            [p.tangent[0], p.tangent[1], p.tangent[2]],
        )
    }
}

#[derive(Debug, Clone)]
pub struct IndexedMesh {
    pub points: Vec<MeshPoint>,
    pub indices: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct RegularMesh {
    pub points: Vec<MeshPoint>,
}

#[derive(Debug, Clone)]
pub struct FigureSet {
    pub mesh: RenderableMesh,
    pub mutations: Vec<FigureMutation>,
    pub color_texture_path: String,
    pub normal_texture_path: String,
}

impl FigureSet {
    pub fn new(
        mesh: RenderableMesh,
        mutations: Vec<FigureMutation>,
        color_texture_path: String,
        normal_texture_path: String,
    ) -> Self {
        FigureSet {
            mesh,
            mutations,
            color_texture_path,
            normal_texture_path,
        }
    }
}

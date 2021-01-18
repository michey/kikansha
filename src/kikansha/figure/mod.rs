use nalgebra::Point3;
use nalgebra_glm::Vec3;

#[derive(Default, Debug, Clone, Copy)]
pub struct PerVerexParams {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub normal: [f32; 3],
}
vulkano::impl_vertex!(PerVerexParams, position, color, normal);

#[derive(Default, Debug, Clone, Copy)]
pub struct PerInstanceParams {
    offset: [f32; 3],
    scale: f32,
}
vulkano::impl_vertex!(PerInstanceParams, offset, scale);

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
    pub vert: Point3<f32>,
    pub color: Vec3,
    pub normal: Vec3,
}

impl MeshPoint {
    pub fn new(vert: [f32; 3], color: [f32; 3], normal: [f32; 3]) -> Self {
        MeshPoint {
            vert: Point3::new(vert[0], vert[1], vert[2]),
            color: Vec3::new(color[0],color[1],color[2]),
            normal: Vec3::new(normal[0],normal[1],normal[2])
        }
    }

    pub fn to_vert(&self) -> PerVerexParams {
        let p = self;
        PerVerexParams {
            position: [p.vert[0], p.vert[1], p.vert[2]],
            color: [p.color[0], p.color[1], p.color[2]],
            normal: [p.normal[0], p.normal[1], p.normal[2]],
        }
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
pub struct Triangle {
    pub vertices: [Point3<f32>; 3],
    pub color: [Vec3; 3],
    pub normale: Vec3,
}

impl Triangle {
    pub fn new(vertices: [Point3<f32>; 3], color: [Vec3; 3], normale: Vec3) -> Self {
        Self {
            vertices,
            color,
            normale,
        }
    }

    pub fn to_vertexes(self) -> Vec<PerVerexParams> {
        let mut result = Vec::with_capacity(3);
        for i in 0..3 {
            result.push(PerVerexParams {
                position: [
                    self.vertices[i][0],
                    self.vertices[i][1],
                    self.vertices[i][2],
                ],
                color: [self.color[i][0], self.color[i][1], self.color[i][2]],
                normal: [self.normale[0], self.normale[1], self.normale[2]],
            })
        }
        result
    }
}

pub struct TriangleBuilder {
    points: Vec<Point3<f32>>,
    color: Vec<Vec3>,
    normal: Vec3,
}

impl TriangleBuilder {
    pub fn with_norm(normal: Vec3) -> Self {
        TriangleBuilder {
            points: Vec::with_capacity(3),
            color: Vec::with_capacity(3),
            normal,
        }
    }

    pub fn add(mut self, p: Point3<f32>, c: Vec3) -> Result<TriangleBuilder, Triangle> {
        self.points.push(p);
        self.color.push(c);
        if self.points.len() == 3 && self.color.len() == 3 {
            Err(Triangle::new(
                [self.points[0], self.points[1], self.points[2]],
                [self.color[0], self.color[1], self.color[2]],
                self.normal,
            ))
        } else {
            Ok(self)
        }
    }

    pub fn get_triangle(&self) -> Result<Triangle, u16> {
        if self.points.len() == 3 && self.color.len() == 3 {
            Ok(Triangle::new(
                [self.points[0], self.points[1], self.points[2]],
                [
                    self.color[0].abs(),
                    self.color[1].abs(),
                    self.color[2].abs(),
                ],
                self.normal,
            ))
        } else {
            Err(0)
        }
    }
}

pub struct FigureBuilder {
    triangles: Vec<Triangle>,
    color: Vec3,
    current_triangle_builder: Option<TriangleBuilder>,
}

impl FigureBuilder {
    pub fn new(color: Vec3) -> Self {
        FigureBuilder {
            triangles: Vec::new(),
            color,
            current_triangle_builder: None,
        }
    }

    pub fn n(mut self, normal: Vec3) -> FigureBuilder {
        match self.current_triangle_builder {
            Some(ref b) => match b.get_triangle() {
                Ok(t) => {
                    self.triangles.push(t);
                    self.current_triangle_builder = Some(TriangleBuilder::with_norm(normal));
                    self
                }
                Err(_) => self,
            },
            None => {
                self.current_triangle_builder = Some(TriangleBuilder::with_norm(normal));
                self
            }
        }
    }

    pub fn p(mut self, p: Point3<f32>) -> FigureBuilder {
        match self.current_triangle_builder {
            Some(b) => match b.add(p, self.color) {
                Ok(builder) => {
                    self.current_triangle_builder = Some(builder);
                    self
                }
                Err(t) => {
                    self.triangles.push(t);
                    self.current_triangle_builder = None;
                    self
                }
            },
            None => self,
        }
    }

    pub fn t(self, a: Point3<f32>, b: Point3<f32>, c: Point3<f32>) -> FigureBuilder {
        self.p(a).p(b).p(c)
    }

    pub fn t_n(self, n: Vec3, a: Point3<f32>, b: Point3<f32>, c: Point3<f32>) -> FigureBuilder {
        self.n(n).p(a).p(b).p(c)
        // self.t_n_c(n, a, b, c, n)
    }

    pub fn t_n_c(
        mut self,
        n: Vec3,
        a: Point3<f32>,
        b: Point3<f32>,
        c: Point3<f32>,
        color: Vec3,
    ) -> FigureBuilder {
        let old_color = self.color;
        self.color = color;
        let mut b = self.n(n).p(a).p(b).p(c);
        b.color = old_color;
        b
    }

    pub fn build(self) -> Figure {
        println!("{}", self.triangles.len());
        Figure::new(self.triangles)
    }
}

#[derive(Debug, Clone)]
pub struct Figure {
    pub triangles: Vec<Triangle>,
}

impl Figure {
    pub fn new(triangles: Vec<Triangle>) -> Self {
        Self { triangles }
    }

    pub fn to_mesh(self) -> RenderableMesh {
        RenderableMesh::Regular(RegularMesh {
            points: self
                .triangles
                .into_iter()
                .flat_map(|triangle| {
                    triangle
                        .vertices
                        .iter()
                        .map(|vert| MeshPoint {
                            vert: Point3::new(vert[0], vert[1], vert[2]),
                            color: triangle.color[0],
                            normal: triangle.normale,
                        })
                        .collect::<Vec<MeshPoint>>()
                })
                .collect(),
        })
    }

    pub fn unit_tetrahedron() -> Self {
        let unit = 0.25;

        let color = Vec3::new(0.8, 0.0, 0.8);

        let hz_norm = Vec3::new(1.0, 1.0, 1.0);

        let a = Point3::new(-unit, unit, unit);
        let b = Point3::new(unit, -unit, unit);
        let c = Point3::new(unit, unit, -unit);
        let d = Point3::new(-unit, -unit, -unit);

        FigureBuilder::new(color)
            .t_n(hz_norm, a, b, c)
            .t_n(hz_norm, a, b, d)
            .t_n(hz_norm, a, c, d)
            .t_n(hz_norm, b, c, d)
            .build()
    }

    pub fn unit_cube() -> Self {
        let unit = 0.25;

        let color = Vec3::new(0.8, 0.3, 0.0);

        let f_side = Vec3::new(-1.0, 0.0, 0.0);
        let s_side = Vec3::new(1.0, 0.0, 0.0);

        let t_side = Vec3::new(0.0, -1.0, 0.0);
        let b_side = Vec3::new(0.0, 1.0, 0.0);
        let l_side = Vec3::new(0.0, 0.0, 1.0);
        let r_side = Vec3::new(0.0, 0.0, -1.0);

        //fron face dots. ccw from top left
        let a = Point3::new(unit, unit, -unit);
        let b = Point3::new(unit, unit, unit);
        let c = Point3::new(unit, -unit, unit);
        let d = Point3::new(unit, -unit, -unit);

        //rear face dots. ccw from top left
        let e = Point3::new(-unit, unit, -unit);
        let f = Point3::new(-unit, unit, unit);
        let j = Point3::new(-unit, -unit, unit);
        let h = Point3::new(-unit, -unit, -unit);

        FigureBuilder::new(color)
            .t_n(f_side, a, b, c)
            .t_n(f_side, a, d, c)
            .t_n(s_side, e, f, j)
            .t_n(s_side, e, h, j)
            .t_n(l_side, b, c, f)
            .t_n(l_side, f, j, c)
            .t_n(r_side, a, d, e)
            .t_n(r_side, e, h, d)
            .t_n(b_side, a, b, e)
            .t_n(b_side, e, f, b)
            .t_n(t_side, h, j, c)
            .t_n(t_side, c, d, h)
            .build()
    }
}

#[derive(Debug, Clone)]
pub struct FigureSet {
    pub mesh: RenderableMesh,
    pub mutations: Vec<FigureMutation>,
}

impl FigureSet {
    pub fn new(mesh: RenderableMesh, mutations: Vec<FigureMutation>) -> Self {
        FigureSet { mesh, mutations }
    }
}

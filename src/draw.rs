use std::{collections::HashMap, iter::once};

use glium::{backend::*, uniforms::*, *};
use vector2math::*;

use crate::{Col, Color, Rect, Vec2};

#[derive(Debug, Clone, Copy, Default)]
pub struct Vertex {
    pub pos: Vec2,
    pub color: Col,
}

implement_vertex!(Vertex, pos, color);

fn uniforms() -> UniformsStorage<'static, [[f32; 4]; 4], EmptyUniforms> {
    uniform! {
        matrix: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0]
        ]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub center: Vec2,
    pub zoom: Vec2,
    pub(crate) window_size: Vec2,
}

impl Camera {
    pub fn window_size(self) -> Vec2 {
        self.window_size
    }
    pub fn center(self, center: Vec2) -> Self {
        Camera { center, ..self }
    }
    pub fn zoom(self, zoom: Vec2) -> Self {
        Camera { zoom, ..self }
    }
    pub fn pos_to_coords(self, pos: Vec2) -> Vec2 {
        pos.sub(self.window_size.div(2.0))
            .div2([self.zoom.x(), -self.zoom.y()])
            .mul(2.0)
            .add(self.center)
    }
    pub fn coords_to_pos(self, coords: Vec2) -> Vec2 {
        coords
            .sub(self.center)
            .div(2.0)
            .mul2([self.zoom.x(), -self.zoom.y()])
            .add(self.window_size.div(2.0))
    }
    pub fn zoom_on(self, zoom: Vec2, on: Vec2) -> Self {
        let old_pos = self.pos_to_coords(on);
        let new_cam = self.zoom(zoom);
        let new_pos = new_cam.pos_to_coords(on);
        new_cam.center(self.center.add(new_pos.sub(old_pos).neg()))
    }
    pub fn map_center<F>(self, f: F) -> Self
    where
        F: FnOnce(Vec2) -> Vec2,
    {
        self.center(f(self.center))
    }
    pub fn map_zoom<F>(self, f: F) -> Self
    where
        F: FnOnce(Vec2) -> Vec2,
    {
        self.zoom(f(self.zoom))
    }
    pub fn map_zoom_on<F>(self, f: F, on: Vec2) -> Self
    where
        F: FnOnce(Vec2) -> Vec2,
    {
        self.zoom_on(f(self.zoom), on)
    }
    fn transform_rect<R>(&self, rect: R) -> R
    where
        R: Rectangle<Scalar = f32>,
    {
        R::new(
            self.transform_point(rect.top_left()),
            rect.size().div2(self.window_size).mul2(self.zoom),
        )
    }
    fn transform_point<V>(&self, p: V) -> V
    where
        V: Vector2<Scalar = f32>,
    {
        p.sub(self.center).mul2(self.zoom).div2(self.window_size)
    }
}

pub struct Drawer<'a, S, F> {
    surface: &'a mut S,
    facade: &'a F,
    program: &'a Program,
    camera: Camera,
    indices: IndicesCache,
}

impl<'a, S, F> Drawer<'a, S, F>
where
    S: Surface,
    F: Facade,
{
    pub(crate) fn new(
        surface: &'a mut S,
        facade: &'a F,
        program: &'a Program,
        camera: Camera,
    ) -> Self {
        Drawer {
            surface,
            facade,
            program,
            camera,
            indices: Default::default(),
        }
    }
    pub fn with_camera<C, G, R>(&mut self, camera: C, g: G) -> R
    where
        C: FnOnce(Camera) -> Camera,
        G: FnOnce() -> R,
    {
        let base_camera = self.camera;
        self.camera = camera(base_camera);
        let res = g();
        self.camera = base_camera;
        res
    }
    pub fn with_absolute_camera<G, R>(&mut self, g: G) -> R
    where
        G: FnOnce() -> R,
    {
        let base_camera = self.camera;
        self.camera = Camera {
            center: base_camera.window_size.div(2.0),
            zoom: [1.0, -1.0],
            window_size: base_camera.window_size,
        };
        let res = g();
        self.camera = base_camera;
        res
    }
    pub fn clear<C>(&mut self, color: C)
    where
        C: Color,
    {
        self.surface
            .clear_color(color.r(), color.g(), color.b(), color.alpha())
    }
    pub fn rectangle<C, R>(&mut self, color: C, rect: R) -> Transformable<'a, '_, S, F>
    where
        C: Color,
        R: Rectangle<Scalar = f32>,
    {
        Transformable {
            drawer: self,
            ty: DrawType::Rectangle(rect.map()),
            color: color.map(),
            drawn: false,
        }
    }
    pub fn circle<C, R>(
        &mut self,
        color: C,
        circ: R,
        resolution: u16,
    ) -> Transformable<'a, '_, S, F>
    where
        C: Color,
        R: Circle<Scalar = f32>,
    {
        Transformable {
            drawer: self,
            ty: DrawType::Ellipse {
                center: circ.center().map(),
                radii: circ.radius().square(),
                resolution,
            },
            color: color.map(),
            drawn: false,
        }
    }
    pub fn polygon<'p, C, V, P>(&mut self, color: C, vertices: P) -> Transformable<'a, '_, S, F>
    where
        C: Color,
        V: Vector2<Scalar = f32> + 'p,
        P: IntoIterator<Item = &'p V>,
    {
        Transformable {
            drawer: self,
            ty: DrawType::Polygon(vertices.into_iter().map(|v| v.map()).collect()),
            color: color.map(),
            drawn: false,
        }
    }
    pub fn line<C, V>(
        &mut self,
        color: C,
        a: V,
        b: V,
        thickness: f32,
    ) -> Transformable<'a, '_, S, F>
    where
        C: Color,
        V: Vector2<Scalar = f32>,
    {
        let perp = b
            .sub(a)
            .unit()
            .rotate_about([0.0; 2], f32::PI / 2.0)
            .mul(thickness / 2.0);
        self.polygon(color, &[a.add(perp), b.add(perp), b.sub(perp), a.sub(perp)])
    }
}

enum DrawType {
    Rectangle(Rect),
    Ellipse {
        center: Vec2,
        radii: Vec2,
        resolution: u16,
    },
    Polygon(Vec<Vec2>),
}

pub struct Transformable<'a, 'b, S, F>
where
    S: Surface,
    F: Facade,
{
    drawer: &'b mut Drawer<'a, S, F>,
    ty: DrawType,
    color: Col,
    drawn: bool,
}

impl<'a, 'b, S, F> Transformable<'a, 'b, S, F>
where
    S: Surface,
    F: Facade,
{
    pub fn draw(&mut self) {
        let vertices = match self.ty {
            DrawType::Rectangle(rect) => {
                let rect = self.drawer.camera.transform_rect(rect);
                VertexBuffer::new(
                    self.drawer.facade,
                    &[
                        Vertex {
                            pos: rect.top_left(),
                            color: self.color,
                        },
                        Vertex {
                            pos: rect.top_right(),
                            color: self.color,
                        },
                        Vertex {
                            pos: rect.bottom_right(),
                            color: self.color,
                        },
                        Vertex {
                            pos: rect.bottom_left(),
                            color: self.color,
                        },
                    ],
                )
                .unwrap()
            }
            DrawType::Ellipse {
                center,
                radii,
                resolution,
            } => VertexBuffer::new(
                self.drawer.facade,
                &once(Vertex {
                    pos: self.drawer.camera.transform_point(center),
                    color: self.color,
                })
                .chain((0..resolution).map(|i| {
                    Vertex {
                        pos: self.drawer.camera.transform_point(
                            center.add(
                                (i as f32 / resolution as f32 * f32::tau())
                                    .angle_as_vector()
                                    .mul2(radii),
                            ),
                        ),
                        color: self.color,
                    }
                }))
                .collect::<Vec<_>>(),
            )
            .unwrap(),
            DrawType::Polygon(ref vertices) => VertexBuffer::new(
                self.drawer.facade,
                &vertices
                    .iter()
                    .map(|&v| Vertex {
                        pos: self.drawer.camera.transform_point(v),
                        color: self.color,
                    })
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        };
        let indices = match self.ty {
            DrawType::Rectangle(_) => self.drawer.indices.rectangle(self.drawer.facade),
            DrawType::Ellipse { resolution, .. } => {
                self.drawer.indices.ellipse(resolution, self.drawer.facade)
            }
            DrawType::Polygon(ref vertices) => self
                .drawer
                .indices
                .polygon(vertices.len() as u16, self.drawer.facade),
        };
        self.drawer
            .surface
            .draw(
                &vertices,
                indices,
                self.drawer.program,
                &uniforms(),
                &Default::default(),
            )
            .unwrap();
        self.drawn = true;
    }
}

impl<'a, 'b, S, F> Drop for Transformable<'a, 'b, S, F>
where
    S: Surface,
    F: Facade,
{
    fn drop(&mut self) {
        if !self.drawn {
            self.draw();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum IndicesType {
    Rectangle,
    Ellipse(u16),
    Polygon(u16),
}

#[derive(Default)]
struct IndicesCache {
    map: HashMap<IndicesType, IndexBuffer<u16>>,
}

impl IndicesCache {
    fn rectangle<F>(&mut self, facade: &F) -> &IndexBuffer<u16>
    where
        F: Facade,
    {
        self.map.entry(IndicesType::Rectangle).or_insert_with(|| {
            IndexBuffer::new(
                facade,
                index::PrimitiveType::TrianglesList,
                &[0, 1, 2, 2, 3, 0],
            )
            .unwrap()
        })
    }
    fn ellipse<F>(&mut self, resolution: u16, facade: &F) -> &IndexBuffer<u16>
    where
        F: Facade,
    {
        self.map
            .entry(IndicesType::Ellipse(resolution))
            .or_insert_with(|| {
                IndexBuffer::new(
                    facade,
                    index::PrimitiveType::TrianglesList,
                    &(1..resolution)
                        .flat_map(|n| once(0).chain(once(n)).chain(once(n + 1)))
                        .chain(once(0).chain(once(resolution)).chain(once(1)))
                        .collect::<Vec<_>>(),
                )
                .unwrap()
            })
    }
    fn polygon<F>(&mut self, vertices: u16, facade: &F) -> &IndexBuffer<u16>
    where
        F: Facade,
    {
        self.map
            .entry(IndicesType::Polygon(vertices))
            .or_insert_with(|| {
                IndexBuffer::new(
                    facade,
                    index::PrimitiveType::TrianglesList,
                    &(1..(vertices - 2))
                        .flat_map(|n| once(0).chain(once(n)).chain(once(n + 1)))
                        .chain(once(0).chain(once(vertices - 2)).chain(once(vertices - 1)))
                        .collect::<Vec<_>>(),
                )
                .unwrap()
            })
    }
}

pub(crate) fn default_shaders<F>(facade: &F) -> Program
where
    F: Facade,
{
    program!(facade,
        140 => {
            vertex: "
                #version 140

                uniform mat4 matrix;

                in vec2 pos;
                in vec4 color;

                out vec4 vColor;

                void main() {
                    gl_Position = vec4(pos, 0.0, 1.0) * matrix;
                    vColor = color;
                }
            ",

            fragment: "
                #version 140
                in vec4 vColor;
                out vec4 f_color;

                void main() {
                    f_color = vColor;
                }
            "
        },

        110 => {
            vertex: "
                #version 110

                uniform mat4 matrix;

                attribute vec2 pos;
                attribute vec4 color;

                varying vec4 vColor;

                void main() {
                    gl_Position = vec4(pos, 0.0, 1.0) * matrix;
                    vColor = color;
                }
            ",

            fragment: "
                #version 110
                varying vec4 vColor;

                void main() {
                    gl_FragColor = vColor;
                }
            ",
        },

        100 => {
            vertex: "
                #version 100

                uniform lowp mat4 matrix;

                attribute lowp vec2 pos;
                attribute lowp vec4 color;

                varying lowp vec4 vColor;

                void main() {
                    gl_Position = vec4(pos, 0.0, 1.0) * matrix;
                    vColor = color;
                }
            ",

            fragment: "
                #version 100
                varying lowp vec4 vColor;

                void main() {
                    gl_FragColor = vColor;
                }
            ",
        },
    )
    .unwrap()
}
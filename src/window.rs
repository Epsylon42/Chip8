use failure::Error;

#[derive(Clone, Copy)]
struct Vertex {
    pos: [f32; 2],
}

impl From<[i32; 2]> for Vertex {
    fn from(pos: [i32; 2]) -> Self {
        Vertex {
            pos: [pos[0] as f32, pos[1] as f32],
        }
    }
}

implement_vertex!(Vertex, pos);

pub struct Window {
    pub ev: glium::glutin::EventsLoop,
    display: glium::Display,
    program: glium::Program,
    vb: glium::VertexBuffer<Vertex>,
    ib: glium::IndexBuffer<u16>,
}

impl Window {
    fn program(facade: &impl glium::backend::Facade) -> Result<glium::Program, Error> {
        glium::Program::from_source(
            facade,
            include_str!("vertex.glsl"),
            include_str!("fragment.glsl"),
            None,
        )
        .map_err(Into::into)
    }

    pub fn new() -> Result<Self, Error> {
        let ev = glium::glutin::EventsLoop::new();
        let wb = glium::glutin::WindowBuilder::new()
            .with_dimensions((800, 400).into())
            .with_title("Chip8");
        let cb = glium::glutin::ContextBuilder::new();

        let display = glium::Display::new(wb, cb, &ev)?;
        let program = Self::program(&display)?;
        let vb = glium::VertexBuffer::new(
            &display,
            &[
                [-1, 1].into(),
                [1, 1].into(),
                [1, -1].into(),
                [1, -1].into(),
                [-1, -1].into(),
                [-1, 1].into(),
            ],
        )?;

        let ib = glium::IndexBuffer::new(
            &display,
            glium::index::PrimitiveType::TrianglesList,
            &[0u16, 1, 2, 3, 4, 5],
        )?;

        Ok(Window {
            ev,
            display,
            program,
            vb,
            ib,
        })
    }

    pub fn draw(&mut self, data: Vec<u8>, width: u32, height: u32) -> Result<(), Error> {
        use glium::Surface;

        let texture = glium::texture::Texture2d::new(
            &self.display,
            glium::texture::RawImage2d {
                data: data.into(),
                width,
                height,
                format: glium::texture::ClientFormat::U8,
            },
        )?;

        let mut frame = self.display.draw();

        frame.clear_color(0.0, 0.0, 0.0, 1.0);
        frame.draw(
            &self.vb,
            &self.ib,
            &self.program,
            &uniform! {
                tex: glium::uniforms::Sampler::new(&texture)
                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
            },
            &glium::DrawParameters {
                depth: glium::Depth {
                    // test: glium::DepthTest::Ignore,
                    ..Default::default()
                },
                ..Default::default()
            },
        )?;

        frame.finish()?;

        Ok(())
    }
}

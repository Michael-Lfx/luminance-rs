//! This program shows how to tweak the render state in order to render two simple triangles with
//! different parameters.
//!
//! Press <space> to switch which triangle is rendered atop of which.
//! Press <b> to activate additive blending or disable it.
//! Press <escape> to quit or close the window.
//!
//! https://docs.rs/luminance

extern crate luminance;
extern crate luminance_glfw;

use luminance::framebuffer::Framebuffer;
use luminance::shader::program::Program;
use luminance::tess::{Mode, Tess};
use luminance::render_state::RenderState;
use luminance_glfw::event::{Action, Key, WindowEvent};
use luminance_glfw::surface::{GlfwSurface, Surface, WindowDim, WindowOpt};
use luminance::blending::{Equation, Factor};
use luminance::depth_test::DepthTest;
use luminance::context::GraphicsContext;

const VS: &'static str = include_str!("vs.glsl");
const FS: &'static str = include_str!("fs.glsl");

type Vertex = ([f32; 2], [f32; 3]);

const TRI_VERTICES: [Vertex; 6] = [
  // first triangle – a red one
  ([ 0.5, -0.5], [1., 0., 0.]),
  ([ 0.0,  0.5], [1., 0., 0.]),
  ([-0.5, -0.5], [1., 0., 0.]),
  // second triangle, a blue one
  ([-0.5,  0.5], [0., 0., 1.]),
  ([ 0.0, -0.5], [0., 0., 1.]),
  ([ 0.5,  0.5], [0., 0., 1.])
];

// Convenience type to demonstrate how the depth test influences the rendering of two triangles.
#[derive(Copy, Clone, Debug)]
enum DepthMethod { 
  Under, // draw the red triangle under the blue one
  Atop // draw the red triangle atop the blue one
}

impl DepthMethod {
  fn toggle(self) -> Self {
    match self {
      DepthMethod::Under => DepthMethod::Atop,
      DepthMethod::Atop => DepthMethod::Under,
    }
  }
}

type Blending = Option<(Equation, Factor, Factor)>;

// toggle between no blending and additive blending
fn toggle_blending(blending: Blending) -> Blending {
  match blending {
    None => Some((Equation::Additive, Factor::One, Factor::One)),
    _ => None
  }
}

fn main() {
  let mut surface = GlfwSurface::new(WindowDim::Windowed(960, 540), "Hello, world!", WindowOpt::default()).expect("GLFW surface creation");

  let (program, _) = Program::<Vertex, (), ()>::from_strings(None, VS, None, FS).expect("program creation");

  // create a red and blue triangles
  let red_triangle = Tess::new(&mut surface, Mode::Triangle, &TRI_VERTICES[0..3], None);
  let blue_triangle = Tess::new(&mut surface, Mode::Triangle, &TRI_VERTICES[3..6], None);

  let mut back_buffer = Framebuffer::back_buffer(surface.size());

  let mut blending = None;
  let mut depth_method = DepthMethod::Under;
  println!("now rendering red triangle {:?} the blue one", depth_method);

  'app: loop {
    for event in surface.poll_events() {
      match event {
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => {
          break 'app
        }

        WindowEvent::Key(Key::Space, _, Action::Release, _) => {
          depth_method = depth_method.toggle();
          println!("now rendering red triangle {:?} the blue one", depth_method);
        }

        WindowEvent::Key(Key::B, _, Action::Release, _) => {
          blending = toggle_blending(blending);
          println!("now blending with {:?}", blending);
        }

        WindowEvent::FramebufferSize(width, height) => {
          back_buffer = Framebuffer::back_buffer([width as u32, height as u32]);
        }

        _ => ()
      }
    }

    surface.pipeline_builder().pipeline(&back_buffer, [0., 0., 0., 0.], |_, shd_gate| {
      shd_gate.shade(&program, |rdr_gate, _| {
        let render_state = RenderState::default()
          // let’s disable the depth test so that every fragment (i.e. pixels) will rendered to every
          // time we have to draw a part of a triangle
          .set_depth_test(DepthTest::Disabled)
          // set the blending we decided earlier
          .set_blending(blending);

        rdr_gate.render(render_state, |tess_gate| {
          match depth_method {
            DepthMethod::Under => {
              tess_gate.render(&mut surface, (&red_triangle).into());
              tess_gate.render(&mut surface, (&blue_triangle).into());
            }

            DepthMethod::Atop => {
              tess_gate.render(&mut surface, (&blue_triangle).into());
              tess_gate.render(&mut surface, (&red_triangle).into());
            }
          }
        });
      });
    });

    surface.swap_buffers();
  }
}

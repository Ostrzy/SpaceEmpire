extern crate sdl2;

use self::sdl2::video::{Window, WindowPos, OPENGL};
use self::sdl2::render::{RenderDriverIndex, ACCELERATED, Renderer};
use self::sdl2::event::poll_event;
use self::sdl2::event::Event::{Quit, KeyDown};
use self::sdl2::keycode::KeyCode;

pub trait Game {
    fn step(&mut self) -> ();
    fn display(&self, drawer: &Renderer) -> ();
}

pub fn example(mut game : Box<Game>) {
    sdl2::init(sdl2::INIT_VIDEO);

    let window = match Window::new("rust-sdl2 demo: Video", WindowPos::PosCentered, WindowPos::PosCentered, 800, 600, OPENGL) {
        Ok(window) => window,
        Err(err) => panic!("failed to create window: {}", err)
    };

    let renderer = match Renderer::from_window(window, RenderDriverIndex::Auto, ACCELERATED) {
        Ok(renderer) => renderer,
        Err(err) => panic!("failed to create renderer: {}", err)
    };

    loop {
        match poll_event() {
            Quit(..) => break,
            KeyDown (_, _, key, _, _, _) => {
                if key == KeyCode::Escape {
                    break;
                } else if key == KeyCode::Space {
                    game.step();
                }
            }
            _ => {} 
        }
        game.display(&renderer);
    }
    sdl2::quit();
}

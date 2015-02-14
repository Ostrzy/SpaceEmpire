extern crate sdl2;

use Player;
use Starmap;
use graphics::Game;
use self::sdl2::render::RenderDrawer;
use self::sdl2::rect::Rect;
use self::sdl2::pixels::Color;

pub struct SpaceEmpire {
    starmap: Starmap,
    players: Vec<Player>
}

impl Game for SpaceEmpire {
    fn step(&mut self) {
        for player in self.players.iter_mut() {
            player.gather_resources(&self.starmap);
            println!("{:?}", player.resources);
        }
    }

    fn display(&self, drawer: &mut RenderDrawer) {
        drawer.set_draw_color(Color::RGB(255, 0, 0));
        drawer.draw_rect(&Rect::new(50, 50, 150, 175));
    }
}

impl SpaceEmpire {
    pub fn new() -> SpaceEmpire{
        SpaceEmpire {
            starmap: Starmap::generate_universe(),
            players: Player::create_players(2)
        }
    }
}

use Player;
use Starmap;
use PlayerId;
use Resources;
use graphics::Game;

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
}

impl SpaceEmpire {
    pub fn new() -> SpaceEmpire{
        SpaceEmpire {
            starmap: Starmap::generate_universe(),
            players: Player::create_players(2)
        }
    }
}

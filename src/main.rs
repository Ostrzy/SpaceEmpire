#![feature(core)]

extern crate sdl2;

use std::rc::Rc;
use std::hash::{Hash, Hasher, Writer};
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Add;
use std::num::ToPrimitive;
use self::sdl2::render::RenderDrawer;
use self::sdl2::rect::{Rect, Point};
use self::sdl2::pixels::Color;

mod graphics;
mod game;

use game::SpaceEmpire;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Resources {
    food: i32,
    technology: i32,
    gold: i32
}

impl Add for Resources {
    type Output = Resources;
    fn add(self, other:Resources) -> Resources {
        Resources {
            food:       self.food + other.food,
            technology: self.technology + other.technology,
            gold:       self.gold + other.gold
        }
    }
}

impl Resources {
    fn new() -> Resources {
        Resources{food: 0, technology: 0, gold: 0}
    }
}

#[derive(Debug, Clone)]
enum BuildingClass {
    Farm,
    Laboratory,
    GoldMine
}

#[derive(Debug, Clone)]
struct Building {
    class: BuildingClass,
    production: Resources
}

impl Building {
    fn new(class: BuildingClass) -> Building {
        let production = match class {
            BuildingClass::Farm       => Resources { food: 5, technology: 0, gold: 0 },
            BuildingClass::Laboratory => Resources { food: 0, technology: 2, gold: 0 },
            BuildingClass::GoldMine   => Resources { food: 0, technology: 0, gold: 8 }
        };
        Building { class: class, production: production }
    }

    fn produce(&self) -> Resources {
        self.production.clone()
    }
}

#[derive(Hash, Eq, PartialEq, Copy)]
enum ShipClass {
    Colony,
    Scout,
    Fighter
}

struct Ship {
    class: ShipClass,
    health: u32,
    speed: u32,
    damage: u32
}

impl Ship {
    fn new(class: ShipClass) -> Ship {
        match class {
            ShipClass::Colony  => Ship { class: class, health: 100, speed: 10, damage: 10},
            ShipClass::Scout   => Ship { class: class, health: 50,  speed: 30, damage: 5},
            ShipClass::Fighter => Ship { class: class, health: 150, speed: 10, damage: 100}
        }
    }
}

enum FleetLocation {
    Moving, // from -> to, turns/needed_turns
    Somewhere // exact location
}

struct Fleet {
    ships: HashMap<ShipClass, Vec<Ship>>,
    location: FleetLocation,
}

impl Fleet{
    fn new() -> Fleet {
        Fleet { ships: HashMap::new(), location: FleetLocation::Somewhere }
    }

    fn add(&mut self, ship: Ship) {
        match self.ships.get(&ship.class) {
          None    => { self.ships.insert(ship.class, Vec::new()); },
          Some(_) => ()
        }
        self.ships.get_mut(&ship.class).unwrap().push(ship);
    }

    fn merge(&mut self, fleet: Box<Fleet>) {
        for (ship_class, ships) in fleet.ships.into_iter() {
            for ship in ships.into_iter() {
                self.add(ship);
            }
        }
    }

    fn size(&self) -> u32 {
        let mut count = 0u32;
        for ships in self.ships.values() {
          count += ships.len() as u32;
        }
        count
    }

    fn count(&self, class: ShipClass) -> u32 {
        match self.ships.get(&class) {
            Some(ships) => ships.len() as u32,
            None        => 0u32
        }
    }

    fn move_to(
        &mut self, fleet: &mut Fleet, number: u32, class: ShipClass
    ) -> Result<(), &'static str> {

        if number > self.count(class) {
            return Err("There are no enough ships");
        }

        let ships = match self.ships.get_mut(&class) {
            Some(s) => s,
            None    => return Ok(())
        };

        for _ in (0..number) {
            fleet.add(ships.pop().unwrap());
        }
        Ok(())
    }
}

#[derive(Eq, PartialEq, Hash, Copy)]
struct PlayerId(pub u32);

struct Player {
    id: PlayerId,
    resources: Resources
}

impl PartialEq for Player {
    fn eq(&self, other : &Player) -> bool {
        self.id == other.id
    }
}

impl Player {
    fn new(id: u32) -> Player {
        Player {
            id: PlayerId(id),
            resources: Resources::new()
        }
    }

    fn gather_resources(&mut self, stars: &Starmap) -> () {
        let id = self.id;
        let owned_systems = stars.systems.values().filter(|s| s.borrow().owner == Some(id));
        let owned_buildings = owned_systems.filter_map(|s| s.borrow().building.clone());
        let owned_production = owned_buildings.map(|b| b.produce());
        self.resources = owned_production.fold(self.resources.clone(), |r, p| r + p );
    }

    fn create_players(num : u32) -> Vec<Player> {
        (0..num).map(|i| Player::new(i)).collect()
    }
}

#[derive(Eq, PartialEq, Hash, Copy)]
struct SolarSystemId(pub u32);

struct SolarSystem {
    id: SolarSystemId,
    building: Option<Building>,
    owner: Option<PlayerId>,
    fleet: Option<Fleet>,
    location: (i32, i32)
}

impl <H: Hasher + Writer> Hash<H> for SolarSystem {
    fn hash(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl SolarSystem {
    fn new(id: SolarSystemId) -> SolarSystem {
        SolarSystem { id: id, building: None, owner: None, fleet: None, location: (0, 0) }
    }

    fn set_homeworld(&mut self, player: PlayerId) {
        self.owner = Some(player);
        self.build(BuildingClass::GoldMine);
    }

    fn build(&mut self, class: BuildingClass) {
        let building = Building::new(class);
        self.building = Some(building);
    }

    fn clear(&mut self) {
        self.building = None;
        self.owner = None;
        self.fleet = None;
    }

    fn display(&self, drawer: &mut RenderDrawer) {
        drawer.set_draw_color(Color::RGB(0, 0, 255));
        let (x,y) = self.location;
        let display_x = x.to_i32().unwrap()*80;
        let display_y = y.to_i32().unwrap()*80;
        drawer.draw_rect(&Rect::new(display_x, display_y, 50, 50));
    }

    fn display_location(&self) -> (i32, i32) {
        let (x,y) = self.location;
        (x*80, y*80)
    }

    fn center(&self) -> (i32, i32) {
        let (x,y) = self.display_location();
        (x+25, y+25)
    }
}

pub struct Starmap {
    systems: HashMap<SolarSystemId, Rc<RefCell<SolarSystem>>>,
    neighbours: HashSet<SystemsConnection>
}

impl Starmap {
    fn new() -> Starmap {
        Starmap { systems: HashMap::new(), neighbours: HashSet::new() }
    }

    fn generate_universe() -> Starmap {
        // 0 - 1 - 2
        // |     / |
        // 3   4   5
        // | /     |
        // 6 - 7 - 8
        let neighbours = [
            (0,1), (1,2), (2,5),
            (5,8), (7,8), (6,7),
            (3,6), (0,3), (4,6),
            (2,4)
        ];

        let mut starmap = Starmap::new();

        for n in 0..9 {
            let system = Rc::new(RefCell::new(SolarSystem::new(SolarSystemId(n))));
            system.borrow_mut().location = ((n % 3).to_i32().unwrap(), (n / 3).to_i32().unwrap());
            starmap.systems.insert(SolarSystemId(n), system);
        }

        for neighbour in neighbours.iter() {
            let system_a = starmap.systems[SolarSystemId(neighbour.0)].clone();
            let system_b = starmap.systems[SolarSystemId(neighbour.1)].clone();
            starmap.neighbours.insert(SystemsConnection::new(system_a, system_b));
        }

        starmap
    }

    fn set_homeworlds(&mut self, players: &[PlayerId]) -> Result<(), &'static str> {
        if players.len() != 2 {
            return Err("Only two players are possible now!");
        }
        self.systems.get_mut(&SolarSystemId(0)).unwrap().borrow_mut().set_homeworld(players[0]);
        self.systems.get_mut(&SolarSystemId(8)).unwrap().borrow_mut().set_homeworld(players[1]);
        Ok(())
    }

    fn display(&self, drawer: &mut RenderDrawer) {
        for system in self.systems.values() {
            system.borrow().display(drawer);
        }
        for connection in self.neighbours.iter() {
            connection.display(drawer);
        }
    }
}

struct SystemsConnection {
    first: Rc<RefCell<SolarSystem>>,
    second: Rc<RefCell<SolarSystem>>
}

impl <H: Hasher + Writer> Hash<H> for SystemsConnection {
    fn hash(&self, state: &mut H) {
        self.first.borrow().hash(state);
        self.second.borrow().hash(state);
    }
}

impl PartialEq for SystemsConnection {
    fn eq(&self, other : &SystemsConnection) -> bool {
        self.first.borrow().id == other.first.borrow().id &&
        self.second.borrow().id == other.second.borrow().id
    }
}

impl Eq for SystemsConnection {}

impl SystemsConnection {
    fn new(system_a: Rc<RefCell<SolarSystem>>, system_b: Rc<RefCell<SolarSystem>>) -> SystemsConnection {
        SystemsConnection{first: system_a, second: system_b}
    }

    fn display(&self, drawer: &mut RenderDrawer) {
        let (x1, y1) = self.first.borrow().center();
        let (x2, y2) = self.second.borrow().center();
        drawer.draw_line(
            Point{x: x1, y: y1},
            Point{x: x2, y: y2});
    }
}

fn test_gathering_resources() {
    let mut player = Player{ id: PlayerId(1), resources: Resources::new() };
    let mut universe = Starmap::generate_universe();

    player.gather_resources(&universe);
    assert!(player.resources == Resources::new());

    assert!(universe.set_homeworlds(&[PlayerId(1), PlayerId(2)]).is_ok());

    player.gather_resources(&universe);
    assert!(player.resources == Resources { gold: 8, food: 0, technology: 0 });
}

fn test_fleet_movement() {
    // Fleets
    let mut fleet1 = Fleet::new();
    let mut fleet2 = Fleet::new();
    fleet1.add(Ship::new(ShipClass::Fighter));
    fleet1.add(Ship::new(ShipClass::Fighter));
    fleet1.add(Ship::new(ShipClass::Scout));
    fleet2.add(Ship::new(ShipClass::Fighter));
    fleet2.add(Ship::new(ShipClass::Fighter));
    fleet2.add(Ship::new(ShipClass::Colony));

    fleet1.merge(Box::new(fleet2));
    let mut fleet3 = Fleet::new();
    assert!(fleet1.move_to(&mut fleet3, 3, ShipClass::Fighter).is_ok());
}

fn print_some_buildings_really_important_piece_of_code() {
    // Buildings
    let farm = Building::new(BuildingClass::Farm);
    let lab  = Building::new(BuildingClass::Laboratory);
    let mine = Building::new(BuildingClass::GoldMine);
    println!("{:?}", farm);
    println!("{:?}", lab);
    println!("{:?}", mine);
}

fn main() {
    print_some_buildings_really_important_piece_of_code();
    test_fleet_movement();
    test_gathering_resources();

    graphics::example(Box::new(SpaceEmpire::new()));
}

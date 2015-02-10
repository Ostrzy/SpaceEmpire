use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Add;

mod graphics;

#[derive(Clone, Show, PartialEq, Eq)]
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

#[derive(Show)]
enum BuildingClass {
    Farm,
    Laboratory,
    GoldMine
}

#[derive(Show)]
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

#[derive(Show, Hash, Eq, PartialEq, Copy)]
enum ShipClass {
    Colony,
    Scout,
    Fighter
}

#[derive(Show)]
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

#[derive(Show)]
enum FleetLocation {
    Moving, // from -> to, turns/needed_turns
    Somewhere // exact location
}

struct Fleet<'a> {
    owner: &'a Player,
    ships: HashMap<ShipClass, Vec<Ship>>,
    location: FleetLocation,
}

impl<'a> Fleet<'a> {
    fn new(player: &'a Player) -> Fleet {
        Fleet { owner: player, ships: HashMap::new(), location: FleetLocation::Somewhere }
    }

    fn add(&mut self, ship: Ship) {
        match self.ships.get(&ship.class) {
          None    => { self.ships.insert(ship.class, Vec::new()); },
          Some(_) => ()
        }
        self.ships.get_mut(&ship.class).unwrap().push(ship);
    }

    fn merge<'b>(&mut self, fleet: Box<Fleet<'b>>) {
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
    fn gather_resources(&mut self, stars: &Starmap) -> () {
        let id = self.id;
        let owned_systems = stars.systems.values().filter(|s| s.owner == Some(id));
        let owned_buildings = owned_systems.filter_map(|s| s.building.as_ref());
        let owned_production = owned_buildings.map(|b| b.production.clone());
        self.resources = owned_production.fold(self.resources.clone(), |r, p| r + p );
    }
}

#[derive(Eq, PartialEq, Hash, Copy)]
struct SolarSystemId(pub u32);

struct SolarSystem<'a> {
    building: Option<Building>,
    owner: Option<PlayerId>,
    fleet: Option<Fleet<'a>>,
    location: (u32, u32)
}

struct Starmap<'a> {
    systems: HashMap<SolarSystemId, SolarSystem<'a>>,
    neighbours: HashSet<(SolarSystemId, SolarSystemId)>
}

impl<'a> SolarSystem<'a> {
    fn new() -> SolarSystem<'a> {
        SolarSystem { building: None, owner: None, fleet: None, location: (0, 0) }
    }
}

impl<'a> Starmap<'a> {
    fn new() -> Starmap<'a> {
        Starmap { systems: HashMap::new(), neighbours: HashSet::new() }
    }

    fn generate_universe() -> Starmap<'a> {
        // 0 - 1 - 2
        // |     / |
        // 3   4   5
        // | /     |
        // 6 - 7 - 8
        let neighbours = [
            (1,3), (0,2), (1,5),
            (0,6), (2,6), (2,8),
            (3,7), (6,8), (7,5)
        ];

        let mut starmap = Starmap::new();

        for n in 0..9 {
            let mut system = SolarSystem::new();
            system.location = (n % 3, n / 3);
            starmap.systems.insert(SolarSystemId(n), system);
        }

        for neighbour in neighbours.iter() {
            starmap.neighbours.insert((SolarSystemId(neighbour.0), SolarSystemId(neighbour.1)));
            starmap.neighbours.insert((SolarSystemId(neighbour.1), SolarSystemId(neighbour.0)));
        }

        starmap
    }
}

fn test_gathering_resources() {
    let mut player = Player{ id: PlayerId(1), resources: Resources::new() };
    let mut universe = Starmap::generate_universe();

    player.gather_resources(&universe);
    assert!(player.resources==Resources::new());

    universe.systems[SolarSystemId(0)].building=Some(Building::new(BuildingClass::Farm));
    universe.systems[SolarSystemId(0)].owner=Some(PlayerId(1));
    universe.systems[SolarSystemId(1)].building=Some(Building::new(BuildingClass::Farm));
    universe.systems[SolarSystemId(1)].owner=Some(PlayerId(2));
    player.gather_resources(&universe);
    assert!(player.resources==Resources { gold: 0, food: 5, technology: 0});
}

fn main() {
    // Buildings
    let farm = Building::new(BuildingClass::Farm);
    let lab  = Building::new(BuildingClass::Laboratory);
    let mine = Building::new(BuildingClass::GoldMine);
    println!("{:?}", farm);
    println!("{:?}", lab);
    println!("{:?}", mine);

    let player = Player{ id: PlayerId(1), resources: Resources::new() };

    // Fleets
    let mut fleet1 = Fleet::new(&player);
    let mut fleet2 = Fleet::new(&player);
    fleet1.add(Ship::new(ShipClass::Fighter));
    fleet1.add(Ship::new(ShipClass::Fighter));
    fleet1.add(Ship::new(ShipClass::Scout));
    fleet2.add(Ship::new(ShipClass::Fighter));
    fleet2.add(Ship::new(ShipClass::Fighter));
    fleet2.add(Ship::new(ShipClass::Colony));

    fleet1.merge(Box::new(fleet2));
    let mut fleet3 = Fleet::new(&player);
    assert!(fleet1.move_to(&mut fleet3, 3, ShipClass::Fighter).is_ok());

    test_gathering_resources();

    graphics::example()
}

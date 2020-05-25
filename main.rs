// actual size of the window
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
// size of the map
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 50;
const MAP_START_HEIGHT: i32 = 1;

const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOM: i32 = 30;

// frame limit
const LIMIT_FPS: i32 = 60;

use std::cmp;
use rand::Rng;
use tcod::colors::*;
use tcod::console::*;

const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_DARK_GROUND: Color = Color { r: 50,g: 50,b: 150 };

const PLAYER: usize = 0;
const SWORD: usize = 2;
const SHOWEL: usize = 3;
const BUCKET: usize = 4;
const ARROW: usize = 6;
const BOW: usize = 5;

struct Tcod {
    root: Root,
    con : Offscreen,
}

//Generic object for player, enemys, items etc..
#[derive(Debug)]
struct Object {
    x: i32,
    y: i32,
    char: char,
    color: Color,
    visable: bool,
    direction: (i32,i32),
    health: i32,
    images: [char;4],
}

impl Object {
    pub fn new(x: i32, y: i32, char: char, color: Color, visable: bool, direction: (i32,i32), health: i32, images: [char;4]) -> Self {
        Object { x, y, char, color, visable, direction, health, images }
    }

    /// move by the given amount, if the destination is not blocked
    pub fn move_by(&mut self, dx: i32, dy: i32, game: &Game) {  
        if !game.map[(self.x + dx) as usize][(self.y + dy) as usize].blocked && self.direction == (dx,dy) {  
            self.x += dx;  
            self.y += dy;
        }
        self.direction = (dx,dy);
    }

    /// set the color and then draw the character that represents this object at its position
    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }

    // update weapon cords
    pub fn update(&mut self, x:i32, y:i32, direction:(i32,i32)){
        self.x = x + direction.0;
        self.y = y + direction.1;
        self.direction = direction;
    }

    pub fn collision(&self, object: &Object) -> bool{
        self.x == object.x && self.y == object.y
    }

}

/// A tile of the map and its properties
#[derive(Clone, Copy, Debug)]
struct Tile {
    blocked: bool,
    block_sight: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
            blocked: false,
            block_sight: false,
        }
    }

    pub fn wall() -> Self {
        Tile {
            blocked: true,
            block_sight: true,
        }
    }
}

type Map = Vec<Vec<Tile>>;

struct Game {
    map: Map,
}

fn render_all(tcod: &mut Tcod, game: &Game, objects: &[Object]) {
    // draw all objects in the list
    for object in objects{
        if object.visable && object.health > 0{
            object.draw(&mut tcod.con);
        }
    }
    // go through all tiles, and set their background color
    for y in MAP_START_HEIGHT..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let wall = game.map[x as usize][y as usize].block_sight;
            if wall {
                tcod.con
                    .set_char_background(x, y, COLOR_DARK_WALL, BackgroundFlag::Set);
            } else {
                tcod.con
                    .set_char_background(x, y, COLOR_DARK_GROUND, BackgroundFlag::Set);
            }
        }
    }

    //draw hud stats
    tcod.con.set_default_foreground(WHITE);
    let health = objects[PLAYER].health;
    let dirt = objects[BUCKET].health - 1;
    let bow = objects[BOW].health - 1;
    let enemys = 0;
    tcod.con.print_ex(0, 0, BackgroundFlag::None, TextAlignment::Left, &format!("V: {}    M: {}    S: {}    W: {}", health,dirt,bow,enemys));

    //draw con on window
    blit(
        &tcod.con,
        (0, 0),
        (MAP_WIDTH, MAP_HEIGHT),
        &mut tcod.root,
        (0, 0),
        1.0,
        1.0,
    );

}

fn handle_keys(tcod: &mut Tcod, game: &mut Game, objects: &mut [Object] ) -> bool {

    //hide weapons
    objects[SWORD].visable = false;
    objects[SHOWEL].visable = false;
    objects[BUCKET].visable = false;
    objects[BOW].visable = false;

    use tcod::input::Key;
    use tcod::input::KeyCode::*;
    
    let key = tcod.root.wait_for_keypress(true);
    match key {
    // player movement keys
        Key { code: Up, .. } => objects[PLAYER].move_by(0, -1, game),
        Key { code: Down, .. } => objects[PLAYER].move_by(0, 1, game),
        Key { code: Left, .. } => objects[PLAYER].move_by(-1, 0, game),
        Key { code: Right, .. } => objects[PLAYER].move_by(1, 0, game),
        Key { code: Enter, alt:true, .. } => {
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
        }
        Key { code:Escape,.. } => return true,


    // weapon keys
        //Sword
        Key { code: Spacebar,.. } => {
            objects[SWORD].update(objects[PLAYER].x, objects[PLAYER].y, objects[PLAYER].direction);
            objects[SWORD].visable = true;
        }

        //Showel
        Key { code: Number1,.. } => {
            objects[SHOWEL].update(objects[PLAYER].x, objects[PLAYER].y, objects[PLAYER].direction);
            objects[SHOWEL].visable = true;
            if game.map[(objects[SHOWEL].x) as usize][(objects[SHOWEL].y) as usize].blocked {
                objects[BUCKET].health += 1;
                game.map[(objects[SHOWEL].x) as usize][(objects[SHOWEL].y) as usize] = Tile::empty();
            };
        }

        //Bucket
        Key { code: Number2,.. } => {
            objects[BUCKET].update(objects[PLAYER].x, objects[PLAYER].y, objects[PLAYER].direction);
            objects[BUCKET].visable = true;
            if objects[BUCKET].health > 1 {
                objects[BUCKET].health -= 1;
                game.map[(objects[BUCKET].x) as usize][(objects[BUCKET].y) as usize] = Tile::wall();
            }
        }

        //Bow
        Key { code: Number3,.. } => {
            objects[BOW].update(objects[PLAYER].x, objects[PLAYER].y, objects[PLAYER].direction);
            objects[BOW].visable = true;
            if objects[BOW].health > 1 {
                objects[BOW].health -= 1 ;
                objects[ARROW].visable = true;
                objects[ARROW].update(objects[PLAYER].x, objects[PLAYER].y, objects[PLAYER].direction);
            }
        }

        _ => {}
    }

    //move arrow
    if objects[ARROW].visable {
        objects[ARROW].move_by(objects[ARROW].direction.0,objects[ARROW].direction.1, game);
    }

    //pick up arrow
    if objects[PLAYER].collision(&objects[ARROW]) && objects[ARROW].visable{
        objects[ARROW].visable = false;
        objects[BOW].health += 1;
    }

    return false;
}

fn animation(objects: &mut [Object]){
    for object in objects{
        let image = object.direction;
        match image {
            (0,-1) => object.char = object.images[0],
            (0,1) => object.char = object.images[1],
            (-1,0) => object.char = object.images[2],
            (1,0) => object.char = object.images[3],
            _ => {}
        }
    }
}

// A rectangle on the map, used to characterise a room.
#[derive(Clone, Copy, Debug)]
struct Rect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }

    pub fn center(&self) -> (i32, i32){
        let center_x = (self.x1 + self.x2) / 2;
        let center_y = (self.y1 + self.y2) / 2;
        (center_x, center_y)
    }

    pub fn intersects_with(&self, other: &Rect) -> bool{
        (self.x1 <= other.x2) 
        && (self.x2 >= other.x1)
        && (self.y1 <= other.y2)
        && (self.y2 >= other.y1) 
    }
}

fn create_room(room: Rect, map: &mut Map) {
    // go through the tiles in the rectangle and make them passable
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    // horizontal tunnel. `min()` and `max()` are used in case `x1 > x2`
    for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    // vertical tunnel
    for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}


fn make_map(player: &mut Object) -> Map {
    // fill map with "blocked" tiles
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    
    let mut rooms = vec![];

    for _ in 0..MAX_ROOM{
        let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        let h = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
       
        let x = rand::thread_rng().gen_range(0, MAP_WIDTH - w);
        let y = rand::thread_rng().gen_range(0, MAP_HEIGHT - h);

        let new_room = Rect::new(x, y, w, h); 
        // run through the other rooms and see if they intersect with this one
        let failed = rooms
            .iter()
            .any(|other_room| new_room.intersects_with(other_room));

        if !failed {
            // this means there are no intersections, so this room is valid

            // "paint" it to the map's tiles
            create_room(new_room, &mut map);

            // center coordinates of the new room, will be useful later
            let (new_x, new_y) = new_room.center();

            if rooms.is_empty() {
                // this is the first room, where the player starts at
                player.x = new_x;
                player.y = new_y;
            } else {
                // all rooms after the first:
                // connect it to the previous room with a tunnel

                // center coordinates of the previous room
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();

                // toss a coin (random bool value -- either true or false)
                if rand::random() {
                    // first move horizontally, then vertically
                    create_h_tunnel(prev_x, new_x, prev_y, &mut map);
                    create_v_tunnel(prev_y, new_y, new_x, &mut map);
                } else {
                    // first move vertically, then horizontally
                    create_v_tunnel(prev_y, new_y, prev_x, &mut map);
                    create_h_tunnel(prev_x, new_x, new_y, &mut map);
                }
            }

            // finally, append the new room to the list
            rooms.push(new_room);
        }    
    }

    return map;
}

fn main() {

    //Window
    let root = Root::initializer()
        .font("sprites.png" , FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)

        .title("WindowName")
        .init();

    //Screen Console
    let mut tcod = Tcod { 
        root, 
        con: Offscreen::new(MAP_WIDTH, MAP_HEIGHT), 
    };

    tcod::system::set_fps(LIMIT_FPS);

    // create object representing the player
    let player = Object::new(0, 0, '@', WHITE, true, (0,1), 3, ['A','B','C','D']);

    // create a NPC
    let npc = Object::new(25, 20, 'S', RED, true, (0,1), 1, ['a','b','c','d']);

    // create a Weapon
    let sword = Object::new(0, 0, 'S', WHITE, false, (0,0), 1, ['E','F','G','H']);
    let shovel = Object::new(0, 0, 'S', WHITE, false, (0,0), 1, ['I','J','K','L']);
    let bucket = Object::new(0, 0, 'S', WHITE, false, (0,0), 1, ['M','M','M','M']);
    let bow = Object::new(0, 0, 'S', WHITE, false, (0,0), 2, ['N','O','P','Q']);
    let arrow = Object::new(0, 0, 'S', WHITE, false, (0,0), 1, ['R','S','T','U']);

    // the list of objects with those two
    let mut objects: Vec<Object> = vec![player, npc, sword, shovel, bucket, bow, arrow];
      


    let mut game = Game {
        // generate map (at this point it's not drawn to the screen)
        map: make_map(&mut objects[0]),
    };    

    //game loop
    while !tcod.root.window_closed() {
        tcod.con.clear();
        
        animation(&mut objects);
        render_all(&mut tcod, &game, &mut objects);
        
        tcod.root.flush();

        // handle keys and exit game if needed
        let exit = handle_keys(&mut tcod, &mut game, &mut objects);

        if exit {break}
    }
}

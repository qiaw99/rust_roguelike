// actual size of the window
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
// size of the map
//thore: map_start für die statusanzeige hinzugefügt
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 50;
const MAP_START_HEIGHT: i32 = 1;

const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOM: i32 = 30;

//jonny: Konstanten für field of view
const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic; // standard FOV Algorithmus
const FOV_LIGHT_WALLS: bool = true; // Soll die Wand gesehen werden?

//David 
const MAX_ROOM_MONSTERS: i32 = 3;

// frame limit
const LIMIT_FPS: i32 = 60;


//jonny
use tcod::map::{FovAlgorithm, Map as FovMap}; 


use std::cmp;
use rand::Rng;
use tcod::colors::*;
use tcod::console::*;


//Jonny save game
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};

use serde::{Deserialize, Serialize};

//jonny: Farben für das "Licht"
const COLOR_LIGHT_WALL: Color = Color {
    r: 130,
    g: 110,
    b: 50,
};
const COLOR_LIGHT_GROUND: Color = Color {
    r: 200,
    g: 180,
    b: 50,
};

const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_DARK_GROUND: Color = Color { r: 50,g: 50,b: 150 };

//thore: constanten für übersichtlichere listen durchgänge hinzugefügt
const PLAYER: usize = 0;
const SWORD: usize = 1;
const SHOWEL: usize = 2;
const BUCKET: usize = 3;
const ARROW: usize = 5;
const BOW: usize = 4;

struct Tcod {
    root: Root,
    con : Offscreen,
    fov: FovMap,
}



//Generic object for player, enemys, items etc..
//thore: visable, direction health, images attributes added
#[derive(Debug, Serialize, Deserialize)]
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

//thore: visable, direction health, images attributes added
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
    pub fn pos(&self) -> (i32, i32) { //jonny
        (self.x, self.y)
    }

    //thore: funktion erstellt für weapons, damit sie an der richtigen stelle erscheinen (vor den gegnern)
    pub fn update(&mut self, x:i32, y:i32, direction:(i32,i32)){
        self.x = x + direction.0;
        self.y = y + direction.1;
        self.direction = direction;
    }
    //thore: collsions function, do two objects hit?
    pub fn collision(&self, object: &Object, x:i32, y:i32) -> bool{
        self.x == object.x+x && self.y == object.y+y
    }

    //david: monster takes dmg
    pub fn takedmg(&mut self, dmg:i32){
        self.health -=dmg;
        if self.health <= 0{
            self.visable = false;
        }
    }

    //david: test attack
    pub fn attack(mut self, monsters: &mut [Object]){
        for monster in monsters{
            if self.collision(monster,0,0){
                monster.takedmg(100);
            }
        }
    }

}

/// A tile of the map and its properties
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct Tile {
    blocked: bool,
    explored: bool, //jonny
    block_sight: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
            blocked: false,
            explored: false, //jonny
            block_sight: false,
        }
    }

    pub fn wall() -> Self {
        Tile {
            blocked: true,
            explored: false, //jonny
            block_sight: true,
        }
    }
}

type Map = Vec<Vec<Tile>>;

#[derive(Serialize, Deserialize)] //Jonny
struct Game {
    map: Map,
}

fn render_all(tcod: &mut Tcod, game: &mut Game, objects: &[Object], fov_recompute: bool,torch_radius: i32, monsters: &[Object] ) {

    if fov_recompute { //jonny
        // recompute FOV if needed (the player moved or something)
        let player = &objects[PLAYER];
        tcod.fov
            .compute_fov(player.x, player.y, torch_radius, FOV_LIGHT_WALLS, FOV_ALGO);
    }

    // draw all objects in the list
    // thore: visable abfrage hinzugefügt, es muss sichtbar sein um gemalt zu werden
   // for object in objects{
        //if object.visable && object.health > 0{
           // object.draw(&mut tcod.con);
      //  }
   // }
    // draw all objects in the list
    //jonny: 
    for object in objects {
        if tcod.fov.is_in_fov(object.x, object.y) && object.visable && object.health > 0 {
             object.draw(&mut tcod.con);
        }   
    }

    
    for object in monsters {
        if tcod.fov.is_in_fov(object.x, object.y) && object.visable && object.health > 0 {
             object.draw(&mut tcod.con);
        }   
    }

    // go through all tiles, and set their background color
    //thore: die map wird erst ab map_start angefangen zu zeichnen, damit platz für das hud ist
    for y in MAP_START_HEIGHT..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let visible = tcod.fov.is_in_fov(x, y);
            let wall = game.map[x as usize][y as usize].block_sight;
            let color = match (visible, wall) {
                // outside of field of view: jonny
                (false, true) => COLOR_DARK_WALL,
                (false, false) => COLOR_DARK_GROUND,
                // inside fov:
                (true, true) => COLOR_LIGHT_WALL,
                (true, false) => COLOR_LIGHT_GROUND,
            };
            let explored = &mut game.map[x as usize][y as usize].explored;
            if visible {
                    // since it's visible, explore it
                         *explored = true;
                        }
            if *explored {
                     // show explored tiles only (any visible tile is explored already)
                        tcod.con
                         .set_char_background(x, y, color, BackgroundFlag::Set);
}
        }
    }

    //thore: draw the hud
    tcod.con.set_default_foreground(WHITE);
    let health = objects[PLAYER].health;
    let dirt = objects[BUCKET].health - 1;
    let bow = objects[BOW].health - 1;
    let enemys = monsters.len();
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

fn handle_keys(tcod: &mut Tcod, game: &mut Game, objects: &mut [Object], monsters: &mut [Object] ) -> bool {

    //thore: hide weapons after each new frame
    weapon_query(0, objects, game, monsters);

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

    //thore: key querys for weapons
        Key { code: Spacebar,.. } => weapon_query(SWORD,objects, game, monsters),
        Key { code: Number1,.. } => weapon_query(SHOWEL,objects, game, monsters),
        Key { code: Number2,.. } => weapon_query(BUCKET,objects, game, monsters),
        Key { code: Number3,.. } => weapon_query(BOW,objects, game, monsters),
        _ => {}
    }

    //thore: test for Arrow
    weapon_query(ARROW, objects, game, monsters);

    return false;
}

//thore: funktion die auf die direction eines objektes guckt und dann das richtige sprite auswählt
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

//thore: weapon verhaltens und eigenschaften funktion
fn weapon_query(weapon: usize,objects: &mut [Object], game: &mut Game, monsters: &mut [Object]){


    //thore: zeige schwert und lass es vor dem player erscheinen
    if weapon == SWORD{
        objects[SWORD].visable = true;
        objects[SWORD].update(objects[PLAYER].x, objects[PLAYER].y, objects[PLAYER].direction);
    }

    //thore: showel
    if weapon == SHOWEL{
        objects[SHOWEL].visable = true;
        objects[SHOWEL].update(objects[PLAYER].x, objects[PLAYER].y, objects[PLAYER].direction);
        if game.map[(objects[SHOWEL].x) as usize][(objects[SHOWEL].y) as usize].blocked {
            objects[BUCKET].health += 1;
            game.map[(objects[SHOWEL].x) as usize][(objects[SHOWEL].y) as usize] = Tile::empty();
        };
    }

    //thore: bucket
    if weapon == BUCKET{
        objects[BUCKET].visable = true;
        objects[BUCKET].update(objects[PLAYER].x, objects[PLAYER].y, objects[PLAYER].direction);
        if objects[BUCKET].health > 1 {
            objects[BUCKET].health -= 1;
            game.map[(objects[BUCKET].x) as usize][(objects[BUCKET].y) as usize] = Tile::wall();
        }
    }

    //thore: bow
    if weapon == BOW{
        objects[BOW].visable = true;
        objects[BOW].update(objects[PLAYER].x, objects[PLAYER].y, objects[PLAYER].direction);
        if objects[BOW].health > 1 {
            objects[BOW].health -= 1 ;
            objects[ARROW].visable = true;
            objects[ARROW].update(objects[PLAYER].x, objects[PLAYER].y, objects[PLAYER].direction);
        }
    }

    //thore: hide all weapons after each farme
    if weapon == 0{
        objects[SWORD].visable = false;
        objects[SHOWEL].visable = false;
        objects[BUCKET].visable = false;
        objects[BOW].visable = false;
    }
 
    if weapon == ARROW{
        //thore: move arrow if visable
        if objects[ARROW].visable {
            objects[ARROW].move_by(objects[ARROW].direction.0,objects[ARROW].direction.1, game);
        }

        //thore: pick up arrow if player stands on it
        if objects[PLAYER].collision(&objects[ARROW],0,0) && objects[ARROW].visable{
            objects[ARROW].visable = false;
            objects[BOW].health += 1;
        }
    }

    //david fightsystem
    if weapon == ARROW || weapon == SWORD {
        for monster in monsters{
            if objects[SWORD].collision(monster,0,0) && objects[SWORD].visable{
                monster.takedmg(100);
            }
            if objects[ARROW].collision(monster,0,0) || objects[ARROW].collision(monster,objects[ARROW].direction.0,objects[ARROW].direction.1) && objects[ARROW].visable{
                monster.takedmg(100);
            }
        }
    }

}

// Qianli: A rectangle on the map, used to characterise a room.
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

// Qianli: go through the tiles in the rectangle and make them passable
fn create_room(room: Rect, map: &mut Map) {
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

// Qianli: horizontal tunnel. 
fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

// Qianli: vertical tunnel
fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

// Qianli: create rooms and tunnels randomly.
fn make_map(objects: &mut Vec<Object>, mut monsters: &mut Vec<Object>) -> Map {
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
                objects[0].x = new_x;
                objects[0].y = new_y;

                // Qianli: put the player and npc in the same room while initializing the game(Location randomly)   
                objects[1].x = rand::thread_rng().gen_range(new_room.x1 + 1, new_room.x2);
                objects[1].y = rand::thread_rng().gen_range(new_room.y1 + 1, new_room.y2);
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
            placing_monster(new_room, &mut map, &mut monsters);
        }    
    }

    return map;
}

// Qianli: test, whether play has touched the npc
//David: changed the function, tests for health
fn can_survive(objects: &mut Vec<Object>) -> bool{
    let player = &objects[0];

    return player.health<=0;
}

//David: added monster list
// Qianli: let npc follow player and sleep for a while
#[allow(dead_code)]
fn ai_follow_player(objects: &mut [Object], game: &mut Game, monsters: &mut Vec<Object>){

    //david remove enemies if health 0
    let mut i:usize = 0;
    while i<monsters.len(){
        if monsters[i].health<=0{
            //david unblock removed monster position
            game.map[monsters[i].x as usize][monsters[i].y as usize].blocked = false;
            monsters.remove(i);
        }
        i+=1;
    }


    for monster in monsters{

        //david unblock current monster position
        game.map[monster.x as usize][monster.y as usize].blocked = false;
       

        let dis_x = &objects[PLAYER].x - &monster.x;
        let dis_y = &objects[PLAYER].y - &monster.y;
        //println!("dis_x: {:?}", dis_x);
        //println!("dis_y: {:?}", dis_y);
        //println!("object_x: {:?}",objects[PLAYER].x);

        let random = rand::thread_rng().gen_range(0, 2);
        if random == 1 && dis_y != 0
        {   
            //david test for collision, change color but do not move into player
            if objects[PLAYER].collision(monster, 0, &dis_y / &dis_y.abs()){
                monster.color = RED;
                objects[PLAYER].health-=1;
            }
            //else move
            else{
                monster.move_by(0, &dis_y / &dis_y.abs(), game);
                monster.color = WHITE;
            }
        
        }
        else if dis_x != 0
        {
            //david test for collision, change color but do not move into player
            if objects[PLAYER].collision(monster, &dis_x / &dis_x.abs(), 0){
                monster.color = RED;
                objects[PLAYER].health-=1;
            }
            //else move
            else{
                monster.move_by(&dis_x / &dis_x.abs(), 0, game);
                monster.color = WHITE;
            }
        
        }

        //david block new monster position
        game.map[monster.x as usize][monster.y as usize].blocked = true;

    }
}
//David
fn placing_monster(room: Rect, map: &Map, monsters: &mut Vec<Object>) { 
    let num_monsters = rand::thread_rng().gen_range(0, MAX_ROOM_MONSTERS + 1);
        for i in 0..num_monsters {
            //chose random spot for this monster
            let x = rand::thread_rng().gen_range(room.x1 + 1, room.x2);
            let y = rand::thread_rng().gen_range(room.y1 + 1, room.y2);
            // Here we choose a 50% chance of spawning either an Orc or Troll
            // rand::random::<f32> will create a random number between 0.0 and 1.0 which is 100%
            let orc = if rand::random::<f32>() < 0.5 {
                let orc = Object::new(x, y, 'W', WHITE, true, (0,1), 100,['W','w','W','W']);
                monsters.append(&mut vec![orc]);

            } else {
                let troll = Object::new(x, y, 'W', WHITE, true, (0,1), 100, ['W', 'W', 'W', 'W']);
                monsters.append(&mut vec![troll]);
            };

        }
    }



fn new_game(tcod: &mut Tcod) -> (Game, Vec<Object>,Vec<Object>) {
    //thore: added atributes visable, direction health, images to all objects
    // create object representing the player
    let player = Object::new(0, 0, '@', WHITE, true, (0,1), 10, ['A','B','C','D']);

    //thore: create all Weapons
    let sword = Object::new(0, 0, 'S', WHITE, false, (0,0), 1, ['E','F','G','H']);
    let shovel = Object::new(0, 0, 'S', WHITE, false, (0,0), 1, ['I','J','K','L']);
    let bucket = Object::new(0, 0, 'S', WHITE, false, (0,0), 1, ['M','M','M','M']);
    let bow = Object::new(0, 0, 'S', WHITE, false, (0,0), 2, ['N','O','P','Q']);
    let arrow = Object::new(0, 0, 'S', WHITE, false, (0,0), 1, ['R','S','T','U']);

    // the list of objects with those two
    //thore: added weapon objects to list
    let mut objects: Vec<Object> = vec![player, sword, shovel, bucket, bow, arrow];  

    //david
    let mut monsters: Vec<Object> = vec![];

    let  game = Game {
        // generate map (at this point it's not drawn to the screen)
        map: make_map(&mut objects, &mut monsters),
    };    

    

    initialise_fov(tcod, &game.map);

    (game, objects,monsters)
}
fn initialise_fov(tcod: &mut Tcod, map: &Map) {
  //jonny: fov
  for y in  MAP_START_HEIGHT..MAP_HEIGHT {
    for x in 0..MAP_WIDTH {
        tcod.fov.set(
            x,
            y,
            !map[x as usize][y as usize].block_sight,
            !map[x as usize][y as usize].blocked,
        );
    }
}
// unexplored areas start black (which is the default background color)
tcod.con.clear();
}

fn play_game(tcod: &mut Tcod, game: &mut Game, objects: &mut Vec<Object>,  torch_radius: i32, monsters: &mut Vec<Object> ) {
   
    let mut previous_player_position = (-1, -1); //jonny: FOV neu berechnen
    //game loop
    while !tcod.root.window_closed() {
        tcod.con.clear();

        //thore: check for animation
        animation(objects);
        let fov_recompute = previous_player_position != (objects[PLAYER].pos());//jonny
        render_all( tcod,  game,  objects, fov_recompute,torch_radius, monsters );
        
        tcod.root.flush();
        previous_player_position = objects[PLAYER].pos(); //jonny
        // handle keys and exit game if needed
        let exit = handle_keys(tcod, game, objects, monsters);
        
        ai_follow_player( objects, game, monsters);

        if exit {
            save_game(game, objects,torch_radius, monsters).unwrap();
            break;
        }

        if monsters.len()<= 0 {
            msgbox("\nHerzlichen Glueckwunsch du Lappen, du hast gewonnen\n", 24, &mut tcod.root);
            break;
        }
        
        // Qianli: check for the break condition
        if can_survive( objects) {
            msgbox("\nDu bist gestorben.\n\nDruecke eine beliebige Taste um ins Hauptmenue zu gelangen.\n", 24, &mut tcod.root);
            break}
    }
}

fn menu<T: AsRef<str>>(header: &str, options: &[T], width: i32, root: &mut Root) -> Option<usize> {
    assert!(
        options.len() <= 26, //Buchstaben von A-Z
        "Cannot have a menu with more than 26 options."
    );

    // calculate total height for the header (after auto-wrap) and one line per option
    let header_height = if header.is_empty() {
        0
    } else {
        root.get_height_rect(0, 0, width, SCREEN_HEIGHT, header)
    };
    let height = options.len() as i32 + header_height;

    // create an off-screen console that represents the menu's window
    let mut window = Offscreen::new(width, height);

    // print the header, with auto-wrap
    window.set_default_foreground(WHITE);
    window.print_rect_ex(
        0,
        0,
        width,
        height,
        BackgroundFlag::None,
        TextAlignment::Left,
        header,
    );

    // print all the options
    for (index, option_text) in options.iter().enumerate() {
        let menu_letter = (b'a' + index as u8) as char;
        let text = format!("({}) {}", menu_letter, option_text.as_ref());
        window.print_ex(
            0,
            header_height + index as i32,
            BackgroundFlag::None,
            TextAlignment::Left,
            text,
        );
    }

    // blit the contents of "window" to the root console
    let x = SCREEN_WIDTH / 2 - width / 2;
    let y = SCREEN_HEIGHT / 2 - height / 2;
    blit(&window, (0, 0), (width, height), root, (x, y), 1.0, 0.7);

    // present the root console to the player and wait for a key-press
    root.flush();
    let key = root.wait_for_keypress(true);

    // convert the ASCII code to an index; if it corresponds to an option, return it
    if key.printable.is_alphabetic() {
        let index = key.printable.to_ascii_lowercase() as usize - 'a' as usize;
        if index < options.len() {
            Some(index)
        } else {
            None
        }
    } else {
        None
    }
}


fn main_menu(tcod: &mut Tcod) {
    let img = tcod::image::Image::from_file("bg.png") 
        .ok()
        .expect("Hintergrund nicht gefunden");  

        

    tcod.root.set_default_foreground(LIGHT_YELLOW);
    tcod.root.print_ex(
        SCREEN_WIDTH / 2,
        SCREEN_HEIGHT / 2 - 4,
        BackgroundFlag::None,
        TextAlignment::Center,
        "Test2",
    );
    tcod.root.print_ex(
        SCREEN_WIDTH / 2,
        SCREEN_HEIGHT - 2,
        BackgroundFlag::None,
        TextAlignment::Center,
        "Test1",
    );
    

    while !tcod.root.window_closed() {  
        // show the background image, at twice the regular console resolution
        tcod::image::blit_2x(&img, (0, 0), (-1, -1), &mut tcod.root, (0, 0));

        // show options and wait for the player's choice
        let choices = &["Neues Spiel", "Spiel fortsetzen", "Hardcore mode","Verlassen"];
        let choice = menu("", choices, 24, &mut tcod.root);
        let  torch_radius: i32 = 10; //FOV Radius

        match choice {  
            Some(0) => {
                // new game

                let (mut game, mut objects, mut monsters) = new_game(tcod);
                play_game(tcod, &mut game, &mut objects, torch_radius, &mut monsters);
            }

            
            Some(1) => {
                // load game
                match load_game() {
                    Ok((mut game, mut objects, torch_radius, mut monsters)) => {
                        initialise_fov(tcod, &game.map);
                        play_game(tcod, &mut game, &mut objects, torch_radius, &mut monsters);
                    }
                    Err(_e) => {
                        msgbox("\nNo saved game to load.\n", 24, &mut tcod.root);
                        continue;
                    }
                }
            }
            

            Some(2) => {
                // new game
                let  torch_radius: i32 = 2;
                let (mut game, mut objects, mut monsters) = new_game(tcod);
                play_game(tcod, &mut game, &mut objects,torch_radius, &mut monsters);
            }




            Some(3) => {
                // quit
                break;
            }

            _ => {}  
        }
    }
}

fn msgbox(text: &str, width: i32, root: &mut Root) {
    let options: &[&str] = &[];
    menu(text, options, width, root);
}

fn save_game(game: &Game, objects: &[Object], torch_radius: i32, monsters: &mut [Object]) -> Result<(), Box<dyn Error>> {  
    let save_data = serde_json::to_string(&(game, objects, torch_radius, monsters))?;  
    let mut file = File::create("savegame")?;  
    file.write_all(save_data.as_bytes())?;  
    Ok(())  
}

fn load_game() -> Result<(Game, Vec<Object>, i32, Vec<Object>), Box<dyn Error>> {
    let mut json_save_state = String::new();
    let mut file = File::open("savegame")?;
    file.read_to_string(&mut json_save_state)?;
    let result = serde_json::from_str::<(Game, Vec<Object>, i32, Vec<Object>)>(&json_save_state)?;
    Ok(result)
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
        fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT), //Jonny FOV 
    };

    tcod::system::set_fps(LIMIT_FPS);

    main_menu(&mut tcod);
}

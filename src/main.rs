extern crate ggez;
extern crate snowflake;
extern crate rand;
use snowflake::*;
use ggez::*;
use ggez::event::*;
use ggez::graphics::{DrawMode, Point, Rect, Color};
use std::time::Duration;
use std::collections::{HashMap, HashSet};
use rand::Rng;
use rand::distributions::{IndependentSample, Range};

pub const BLACK: Color = Color { r:0.0, b:0.0, g:0.0, a:1.0 };
pub const WHITE: Color = Color { r:1.0, b:1.0, g:1.0, a:1.0 };
pub const LIMEGREEN: Color = Color { r:0.196, b:0.804, g:0.196, a:1.0 };
pub const RED: Color = Color { r:1.0, b:0.0, g:0.0, a:1.0 };
pub const BLUE: Color = Color { r:0.0, b:0.0, g:1.0, a:1.0 };

type ID = ProcessUniqueId;
type Coord = i32;

enum Position {
    Left,
    Right,
    Top,
    Bottom
}

enum GameState {
    Menu,
    Playing,
    Pause,
    GameOver
}

struct Rusto {
    id: ID,
    x: i32,
    y: i32,
    color: Color
}

impl Rusto {
    fn new(x:Coord, y:Coord) -> Rusto {
        let colors = Rusto::get_color_vector();
        let mut rng = rand::thread_rng();
        let color_range_value = Range::new(0,3).ind_sample(&mut rng); 
        let color = colors[color_range_value as usize];
        Rusto {
            id: ProcessUniqueId::new(),
            x: x,
            y: y,
            color: color
        }
    }

    fn get_color_vector() -> Vec<Color> {
        vec![LIMEGREEN, RED, BLUE]
    }
}

struct Assets {
    font: graphics::Font,
}

impl Assets {
    fn new(ctx: &mut Context) -> GameResult<Assets> {
        let font = graphics::Font::new(ctx, "/FiraSans-Regular.ttf", 18)?;
        Ok(Assets {
            font: font,
        })
    }
}

struct MainState {
    assets: Assets,
    pos_x: f32,
    rustos: HashMap<ID,Rusto>,
    blob: Vec<ID>,
    drop_timer: f64,
    current_score: u32,
    state: GameState,
    ROW_START: Coord,
    ROW_END: Coord
}

enum BlobDirections {
    Right,
    Left,
    Clockwise,
    CounterClockwise,
}

enum DropState {
    Drop,
    NotDrop
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let mut s = MainState {
            assets: Assets::new(ctx)?,
            pos_x: 0.0,
            rustos: HashMap::new(),
            blob: Vec::new(),
            drop_timer: 0.0,
            current_score: 0,
            state: GameState::Playing,
            ROW_START: 1,
            ROW_END: 6
        };
        s.blob = MainState::new_blob(&mut s);
        Ok(s)
    }

    fn new_blob(s: &mut MainState) -> Vec<ID> {
        let mut new_blob: Vec<ID> = Vec::new();
        let new_rusto = Rusto::new(3,1);
        let new_rusto_id = new_rusto.id;
        s.rustos.insert(new_rusto.id, new_rusto);
        new_blob.push(new_rusto_id);
        let new_rusto = Rusto::new(2,1);
        let new_rusto_id = new_rusto.id;
        s.rustos.insert(new_rusto_id, new_rusto);
        new_blob.push(new_rusto_id);
        new_blob
    }
    
    fn rusto_world_to_screen_cords(&self, rusto: &Rusto) -> (f32,f32) {
        let rusto_size = 50.0;
        let x_coff = 50.0;
        let y_coff = 50.0;
        (x_coff + rusto.x as f32 * rusto_size, y_coff + rusto.y as f32 * rusto_size)
    }

    fn move_blob(&mut self, direction: BlobDirections) {
        match direction {
            BlobDirections::Left => {
                for id in self.blob.iter() {
                    let rusto = self.rustos.get(&id).unwrap();
                    if rusto.x <= self.ROW_START {
                        return;
                    }
                }
                for id in self.blob.iter() {
                    let rusto = self.rustos.get_mut(&id).unwrap();
                    rusto.x -= 1;
                }
            },
            BlobDirections::Right => {
                for id in self.blob.iter() {
                    let rusto = self.rustos.get(&id).unwrap();
                    if rusto.x > 6 {
                        return;
                    }
                }
                for id in self.blob.iter() {
                    let rusto = self.rustos.get_mut(&id).unwrap();
                    rusto.x += 1;
                }
            }
            _ => {}
        }
    }

    fn check_side_rusto_position(&self) -> Position {
        let main_rusto = self.rustos.get(&self.blob[0]).unwrap();
        let side_rusto = self.rustos.get(&self.blob[1]).unwrap();
        if side_rusto.x < main_rusto.x {
            Position::Left
        } else if side_rusto.x > main_rusto.x {
            Position::Right
        } else if side_rusto.y < main_rusto.y {
            Position::Top
        } else {
            Position::Bottom
        }
    }

    fn rotate_side_rusto_pos_in_blob(&mut self, new_pos: Position) {
        let (main_x, main_y) = {
            let main_rusto = self.rustos.get(&self.blob[0]).unwrap();
            (main_rusto.x, main_rusto.y)
        };
        let mut side_rusto = self.rustos.get_mut(&self.blob[1]).unwrap();
        match new_pos {
            Position::Left => {
                if main_x > self.ROW_START {
                    side_rusto.x = main_x - 1;
                    side_rusto.y = main_y;
                }
            }
            Position::Right =>  {
                if main_x < self.ROW_END {
                    side_rusto.x = main_x + 1;
                    side_rusto.y = main_y;
                }
            }
            Position::Top =>  {
                if main_y > -1 {
                    side_rusto.x = main_x;
                    side_rusto.y = main_y - 1;
                }
            }
            Position::Bottom =>  {
                if main_y < 10 {
                    side_rusto.x = main_x;
                    side_rusto.y = main_y + 1;
                }
            }
        }
    }

    fn rotate_blob_clockwise(&mut self) {
        let side_rusto_pos = self.check_side_rusto_position();
        match side_rusto_pos {
            Position::Left => { self.rotate_side_rusto_pos_in_blob(Position::Top) }
            Position::Right => { self.rotate_side_rusto_pos_in_blob(Position::Bottom) }
            Position::Top => { self.rotate_side_rusto_pos_in_blob(Position::Right) }
            Position::Bottom => { self.rotate_side_rusto_pos_in_blob(Position::Left) }
        }
    }
    
    fn draw_rustos(&self, ctx: &mut Context) -> GameResult<()>{
        for (_, rusto) in self.rustos.iter() {
            let (x,y) = self.rusto_world_to_screen_cords(&rusto);
            graphics::set_color(ctx, rusto.color)?;
            graphics::rectangle(
                ctx,
                DrawMode::Fill,
                Rect { x: x, y: y, w:50.0, h:50.0}
            )?;
        }
        graphics::set_color(ctx, WHITE)?;
        Ok(())
    }

    fn draw_playarea(&self, ctx: &mut Context) -> GameResult<()> {
        graphics::rectangle(
            ctx,
            DrawMode::Line,
            Rect { x: 200.0, y: 300.0, w:300.0, h:550.0}
        )?;
        Ok(())
    }

    fn draw_ui(&self, ctx: &mut Context) -> GameResult<()> {
        let score_disp = graphics::Text::new(ctx, "score", &self.assets.font)?;
        let current_score = graphics::Text::new(ctx, &self.current_score.to_string(), &self.assets.font)?;
        let score_dest = graphics::Point::new(
            (score_disp.width() / 2) as f32 + 400.0,
            (score_disp.height() / 2) as f32 + 10.0,
        );
        let current_score_dest = graphics::Point::new(
            (current_score.width() / 2) as f32 + 400.0,
            (current_score.height() / 2) as f32 + 30.0,
        );
        graphics::draw(ctx, &score_disp, score_dest, 0.0)?;
        graphics::draw(ctx, &current_score, current_score_dest, 0.0)?;
        Ok(())
    }

    fn draw_gameover(&self, ctx: &mut Context) -> GameResult<()> {
        let game_over_text = graphics::Text::new(ctx, "GAME OVER", &self.assets.font)?;
        let game_over_dest = graphics::Point::new(
            (game_over_text.width() / 2) as f32 + 400.0,
            (game_over_text.height() / 2) as f32 + 60.0,
        );
        graphics::draw(ctx, &game_over_text, game_over_dest, 0.0)?;
        Ok(())
    }

    fn check_blob_boundaries(&mut self, add_x: i32, add_y: i32) -> DropState {
        let blob_ids: HashMap<ID,u32> = self.blob.iter()
            .map(|id| (*id, 1 as u32))
            .collect();
        for id in self.blob.iter() {
            let rusto: &Rusto = self.rustos.get(&id).unwrap(); 
            //bottom of screen
            if rusto.y + add_y == 10 {
                return DropState::Drop
            }
            for (hash_id, hash_rusto) in self.rustos.iter() {
                if !blob_ids.contains_key(&hash_id) && rusto.x + add_x == hash_rusto.x && rusto.y + add_y == hash_rusto.y {
                    return DropState::Drop
                }
            }
        }
        DropState::NotDrop
    }

    fn make_adjacency_list_for_rusto(&self, rusto: &Rusto) -> Vec<ID> {
        self.rustos.iter()            
            .filter(|&(_,r)| rusto.color == r.color )
            .filter(|&(_,r)| rusto.id != r.id)
            .filter(|&(_,r)| {
                (r.y == rusto.y && r.x + 1 == rusto.x)
                    || (r.y == rusto.y && r.x - 1 == rusto.x )
                    || (r.x == rusto.x && r.y + 1 == rusto.y )
                    || (r.x == rusto.x && r.y - 1 == rusto.y ) })
            .map(|(id,_)| *id)
            .collect()
    }

    fn return_all_groups(&self, id: ID, adj_list: &HashMap<ID, Vec<ID>>, visited_nodes: &mut HashSet<ID>) -> Vec<ID> {
        if visited_nodes.contains(&id) {
            return Vec::new();
        }
        visited_nodes.insert(id);
        adj_list.get(&id).unwrap().iter()
            .flat_map(|id| {
                let mut group = self.return_all_groups(*id, adj_list, visited_nodes);
                group.push(*id);
                group
            })
            .collect()
    }
    
    fn get_all_groups(&mut self) -> Vec<Vec<ID>> {
        let mut group_list: Vec<Vec<ID>> = Vec::new();
        let adjacency_list: HashMap<ID, Vec<ID>> = self.rustos.iter()
            .map(|(id,rusto)| (*id, self.make_adjacency_list_for_rusto(rusto)))
            .collect();
        let mut visited_nodes: HashSet<ID> = HashSet::new();
        for (id,_) in self.rustos.iter() {
            if !visited_nodes.contains(id) {
                let mut group = self.return_all_groups(*id, &adjacency_list, &mut visited_nodes);
                group.push(*id);
                group.sort();
                group.dedup();
                group_list.push(group);
            }
        }
        println!("Group list is {:?}", group_list);
        group_list
    }

    fn drop_all_rustos(&mut self) {
        let mut rustos_by_x: HashMap<Coord, Vec<ID>> = HashMap::new();
        for (id, rusto) in self.rustos.iter() {
            let rusto_vec = rustos_by_x.entry(rusto.x).or_insert(Vec::new());
            rusto_vec.push(*id);
        }

        for (x, vec_id) in rustos_by_x.iter() {
            let mut move_coff = 0;
            let y_hash: HashMap<Coord, ID> = vec_id.iter()
                .map(|id| (self.rustos.get(id).unwrap().y, *id))
                .collect();
            for y in (1..11).rev() {
                if y_hash.contains_key(&y) {
                    let id = y_hash.get(&y).unwrap();
                    let mut rusto = self.rustos.get_mut(id).unwrap();
                    rusto.y += move_coff;
                } else {
                    move_coff += 1;
                }
            }
        }
    }

    fn remove_rusto(&mut self, id: ID) {
        self.rustos.remove(&id);
    }

    fn regroup_rustos(&mut self) {
        let mut regrouped = false;
        let groups = self.get_all_groups();
        for group in groups.iter() {
            if group.len() > 3 {
                regrouped = true;
                self.current_score += 100;
                for id in group.iter() {
                    self.remove_rusto(*id);
                }
            }
        }
        self.drop_all_rustos();
        if regrouped {
            self.regroup_rustos();
        }
    }
    
    fn drop_blob(&mut self) {
        self.blob = Vec::new();
        self.regroup_rustos();
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context, dt: Duration) -> GameResult<()> {
        match self.state {
            GameState::Playing => {
                self.drop_timer += timer::duration_to_f64(dt);
                if self.drop_timer > 0.5 {
                    if self.blob.len() > 0 {
                        let drop_state: DropState = self.check_blob_boundaries(0, 1);
                        match drop_state {
                            DropState::Drop => { self.drop_blob(); }
                            DropState::NotDrop => {
                                for id in self.blob.iter() {
                                    let mut rusto: &mut Rusto = self.rustos.get_mut(&id).unwrap(); 
                                    if rusto.y < 10 {
                                        rusto.y = rusto.y + 1;
                                    }
                                }
                            }
                        }
                    } else {
                        self.blob = MainState::new_blob(self);
                    }
                    self.drop_timer = 0.0;
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        self.draw_playarea(ctx)?;
        self.draw_rustos(ctx)?;
        self.draw_ui(ctx)?;
        match self.state {
            GameState::GameOver => {
            }
            _ => {}
        }
        graphics::present(ctx);
        Ok(())
    }
    fn key_down_event(&mut self, keycode: Keycode, _keymod: Mod, _repeat: bool) {
        match self.state {
            GameState::Playing => {
                match keycode {
                    Keycode::Up => {
                        if self.blob.len() > 0 {
                            self.rotate_blob_clockwise();
                        }
                    }
                    Keycode::Left => {
                        self.move_blob(BlobDirections::Left);
                    }
                    Keycode::Right => {
                        self.move_blob(BlobDirections::Right);
                    }
                    Keycode::Space => {
                        
                    }
                    _ => (), // Do nothing
                }
            }
            GameState::GameOver => {}
            _ => {}
        }
    }
}

pub fn main() {
    let mut c = conf::Conf::new();
    c.window_title = "Rusto Rusto".to_string();
    let ctx = &mut Context::load_from_conf("rustorusto", "neurogoo", c).unwrap();
    let state = &mut MainState::new(ctx).unwrap();
    event::run(ctx, state).unwrap();
}

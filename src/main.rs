extern crate ggez;
extern crate snowflake;
use snowflake::*;
use ggez::*;
use ggez::event::*;
use ggez::graphics::{DrawMode, Point, Rect, Color};
use std::time::Duration;
use std::collections::{HashMap};

pub const BLACK: Color = Color { r:0.0, b:0.0, g:0.0, a:1.0 };
pub const WHITE: Color = Color { r:1.0, b:1.0, g:1.0, a:1.0 };
pub const LIMEGREEN: Color = Color { r:0.196, b:0.804, g:0.196, a:1.0 };
pub const RED: Color = Color { r:1.0, b:0.0, g:0.0, a:1.0 };
pub const BLUE: Color = Color { r:0.0, b:0.0, g:1.0, a:1.0 };

type ID = ProcessUniqueId;

struct Rusto {
    id: ID,
    x: i32,
    y: i32,
    color: Color
}

impl Rusto {
    fn new(x:i32, y:i32) -> Rusto {
        Rusto {
            id: ProcessUniqueId::new(),
            x: x,
            y: y,
            color: RED
        }
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
        };
        s.blob = MainState::new_blob(&mut s);
        Ok(s)
    }

    fn new_blob(s: &mut MainState) -> Vec<ID> {
        let mut new_blob: Vec<ID> = Vec::new();
        let new_rusto = Rusto::new(3,3);
        let new_rusto_id = new_rusto.id;
        s.rustos.insert(new_rusto.id, new_rusto);
        new_blob.push(new_rusto_id);
        let new_rusto = Rusto::new(2,3);
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
                    if rusto.x <= 1 {
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

    fn check_blob_boundaries(&mut self, add_x: i32, add_y: i32) -> DropState {
        for id in self.blob.iter() {
            let rusto: &Rusto = self.rustos.get(&id).unwrap(); 
            //bottom of screen
            if rusto.y + add_y == 10 {
                return DropState::Drop
            }
            for (hash_id, hash_rusto) in self.rustos.iter() {
                if id != hash_id && rusto.x + add_x == hash_rusto.x && rusto.y + add_y == hash_rusto.y {
                    return DropState::Drop
                }
            }
        }
        DropState::NotDrop
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context, dt: Duration) -> GameResult<()> {
        self.drop_timer += timer::duration_to_f64(dt);
        if self.drop_timer > 0.5 {
            if self.blob.len() > 0 {
                let drop_state: DropState = self.check_blob_boundaries(0, 1);
                match drop_state {
                    DropState::Drop => { self.blob = Vec::new() }
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
        Ok(())
    }
    
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        self.draw_playarea(ctx)?;
        self.draw_rustos(ctx)?;
        self.draw_ui(ctx)?;
        graphics::present(ctx);
        Ok(())
    }
    fn key_down_event(&mut self, keycode: Keycode, _keymod: Mod, _repeat: bool) {
        match keycode {
            Keycode::Up => {
                
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
}

pub fn main() {
    let mut c = conf::Conf::new();
    c.window_title = "Rusto Rusto".to_string();
    let ctx = &mut Context::load_from_conf("rustorusto", "neurogoo", c).unwrap();
    let state = &mut MainState::new(ctx).unwrap();
    event::run(ctx, state).unwrap();
}

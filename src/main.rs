extern crate ggez;
use ggez::*;
use ggez::graphics::{DrawMode, Point, Rect};
use std::time::Duration;
use std::collections::{HashMap};

type ID = u32;

struct Rusto {
    id: ID,
    x: u32,
    y: u32
}

struct MainState {
    pos_x: f32,
    rustos: HashMap<ID,Rusto>,
    blob: Vec<ID>,
    drop_timer: f64
}

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {
        let mut s = MainState {
            pos_x: 0.0,
            rustos: HashMap::new(),
            blob: Vec::new(),
            drop_timer: 0.0
        };
        s.rustos.insert(1, Rusto {
            id: 1,
            x: 3,
            y: 3,
        });
        s.blob.push(1);
        Ok(s)
    }
    fn rusto_world_to_screen_cords(&self, rusto: &Rusto) -> (f32,f32) {
        let rusto_size = 50.0;
        let x_coff = 50.0;
        let y_coff = 50.0;
        (x_coff + rusto.x as f32 * rusto_size, y_coff + rusto.y as f32 * rusto_size)
    }

    fn draw_rustos(&self, ctx: &mut Context) -> GameResult<()>{
        for (_, rusto) in self.rustos.iter() {
            let (x,y) = self.rusto_world_to_screen_cords(&rusto);
            graphics::rectangle(
                ctx,
                DrawMode::Fill,
                Rect { x: x, y: y, w:50.0, h:50.0}
            )?;
        }
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
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context, dt: Duration) -> GameResult<()> {
        self.drop_timer += timer::duration_to_f64(dt);
        if self.drop_timer > 0.5 {
            for id in self.blob.iter() {
                let mut rusto: &mut Rusto = self.rustos.get_mut(&id).unwrap();           
                if rusto.y < 10 {
                    rusto.y = rusto.y + 1;
                }
            }
            self.drop_timer = 0.0;
        }
        Ok(())
    }
    
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        self.draw_playarea(ctx)?;
        self.draw_rustos(ctx)?;
        graphics::present(ctx);
        Ok(())
    }
}

pub fn main() {
    let mut c = conf::Conf::new();
    c.window_title = "Rusto Rusto".to_string();
    let ctx = &mut Context::load_from_conf("rustorusto", "neurogoo", c).unwrap();
    let state = &mut MainState::new(ctx).unwrap();
    event::run(ctx, state).unwrap();
}

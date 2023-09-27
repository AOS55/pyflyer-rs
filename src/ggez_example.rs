use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

use ggez::{Context, ContextBuilder, GameResult, conf};
use ggez::graphics::{self, Color, Canvas, Image};
use ggez::GameError;
use ggez::event;

pub struct MyGameError(GameError);


impl From<MyGameError> for PyErr {
    fn from(error: MyGameError) -> Self {
        PyValueError::new_err(error.0.to_string())
    }
}

impl From<GameError> for MyGameError {
    fn from(game_error: GameError) -> Self {
        Self(game_error)
    }
}


#[pyfunction(name="ContextBuilder")]
pub fn py_context_builder() -> Result<(), MyGameError> {
    let mut cb = ContextBuilder::new("flyer-env", "ggez")
        .window_mode(conf::WindowMode::default().dimensions(400.0, 400.0));

    // Build the context ctx and event loop
    let (mut ctx, event_loop) = cb.build()?;
    
    struct State {}

    impl State {
        fn new(ctx: &mut Context) -> GameResult<State> {

            let s = State {};
            Ok(s)
        }
    }

    impl event::EventHandler<ggez::GameError> for State {

        fn update(&mut self, _ctx: &mut Context) -> GameResult {
            Ok(())
        }
        
        fn draw(&mut self, _ctx: &mut Context) -> GameResult {
            
            let mut canvas = graphics::Canvas::from_frame(_ctx, Color::BLACK);
            
            let frame = _ctx.gfx.frame();
            println!("frame: {:?}", frame.to_pixels(_ctx));


            // match screenshot(_ctx) {

            //     Ok(image) => image.encode(_ctx, graphics::ImageFormat::Png, path::Path::new("1.png")),
            //     Err(_) => () // Do some error handling
                
            // }

            canvas.finish(_ctx) 

        }
    }

    let state = State::new(&mut ctx)?;

    event::run(ctx, event_loop, state)
}
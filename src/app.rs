use crate::tui;

use color_eyre::owo_colors::OwoColorize;
use color_eyre::{
    eyre::WrapErr,
    Result,
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use num::ToPrimitive;

use ratatui::widgets::canvas::Canvas;
use ratatui::{
    prelude::*,
    widgets::{block::*, canvas, *},
    widgets::Paragraph,
    style::Color,
};

use std::path::Path;

use std::time::Duration;
use std::vec;

use rand::prelude::*;

use crate::read_write::*;

#[derive(Debug, Default)]
pub struct App {
    pub score: u64,
    pub highscore: u64,
    exit: bool,
    y: Vec<f64>,
    x: Vec<f64>,
    head: Vec<f64>,
    length: usize,
    fruits: Vec<f64>,
    on_puase: bool,
    dead: bool,
    auto: bool,
    direction: Vec<Vec<f64>>,
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {

        if self.dead {
            let title = Title::from(" Dinosaur Game ".bold());
            let instructions = Title::from(Line::from(vec![
                " Restart ".into(),
                "<Enter> ".bold(),
                " Quit ".into(),
                "<Q> ".bold(),
            ]));
        
            
            let block = Block::default()
                    .title(title.alignment(Alignment::Center)
                        .position(Position::Top))
                    .title(instructions
                        .alignment(Alignment::Center)
                        .position(Position::Bottom))
                    .borders(Borders::ALL);
            
            let best_counter_text = Text::from(vec![Line::from(vec![
                "Highscore ".into(),
                self.highscore.to_string().into(),
            ])]);

            Paragraph::new(best_counter_text)
            .block(block.clone())
            .left_aligned()
            .render(area, buf);
            
            let info_text = Text::from(vec![Line::from(vec![
                "You died with score ".into(),
                self.score.to_string().into(),
            ])]);

            Paragraph::new(info_text)
            .block(block)
            .centered()
            .bold()
            .red()
            .render(area, buf);
        }        
        else {
            let title = Title::from(" Dinosaur Game ".bold());
            let instructions = Title::from(Line::from(vec![
                " Up ".into(),
                "<Up> ".bold(),
                " Right ".into(),
                "<Right> ".bold(),
                " Down ".into(),
                " Left ".into(),
                "<Left>".bold(),
                "<Down> ".bold(),
                " Pause ".into(),
                "<Esc> ".bold(),
                " Quit ".into(),
                "<Q> ".bold(),
                " Auto ".into(),
                "<Tab> ".bold(),
            ]));
            
            let color: Color;
            let player_color: Color;

            color = Color::Black;
            player_color = Color::White;
            
            
            let block = Block::default()
                        .title(title.alignment(Alignment::Center)
                            .position(Position::Top))
                        .title(instructions
                            .alignment(Alignment::Center)
                            .position(Position::Bottom))
                        .borders(Borders::ALL);

            let counter_text = Text::from(vec![Line::from(vec![
                "Score ".into(),
                self.score.to_string().into(),
            ])]);

            let best_counter_text = Text::from(vec![Line::from(vec![
                "Highscore ".into(),
                self.highscore.to_string().into(),
            ])]);

            Paragraph::new(counter_text)
            .block(block.clone())
            .right_aligned()
            .render(area, buf);

            Paragraph::new(best_counter_text)
            .block(block.clone())
            .left_aligned()
            .render(area, buf);
                
            if self.on_puase {
                let pause_text = Text::from("Paused");

                Paragraph::new(pause_text)
                .centered()
                .block(block.clone())
                .render(area, buf);
            }

            //TODO render snake and food
            let player = Canvas::default()
                .block(block)
                .x_bounds([-90.0, 90.0])
                .y_bounds([-45.0, 45.0])
                .background_color(color)
                .paint(|ctx|{
                    ctx.draw(&canvas::Rectangle {
                        x: self.head[0],
                        y: self.head[1],
                        width: 2.0,
                        height: 2.0,
                        color: player_color,
                    });
                    for i in 0..self.length {
                        ctx.draw(&canvas::Rectangle {
                            x: self.x[i],
                            y: self.y[i],
                            width: 1.0,
                            height: 1.0,
                            color: player_color,
                        });
                    }
                    ctx.layer();
                    if self.fruits.len() > 0 {
                        ctx.draw(&canvas::Rectangle {
                            x: self.fruits[0],
                            y: self.fruits[1],
                            width: 1.0,
                            height: 1.0,
                            color: Color::Red,
                        })
                    }
                });
            player.render(area, buf);  
        }   
    }   
}

impl App {
    // runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui) -> Result<()> {
        loop {
            terminal.draw(|frame| self.render_frame(frame))?;
            let time = 100000;
            if event::poll(Duration::from_micros(time))? {
                self.handle_events().wrap_err("handle events failed")?;
            }
            if self.exit {
                break;
            }
            if self.on_puase || self.dead {
                continue;
            }
            if self.auto {
                autorun(self)?;
            }
            self.collision_check()?;
            self.death_check()?;
            self.highscore();
            self.update_position()?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    fn highscore(&mut self) {
        if self.score > self.highscore {
            self.highscore = self.score;
        }
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event).wrap_err_with(|| {
                    format!("handling key event failed: \n{key_event:#?}")
                })
            }
           _ => Ok(())
        }
    }

    fn death_check(&mut self) -> Result<()> {
        if self.head[0] > 90.0 || self.head[0] < -90.0 || self.head[1] > 45.0 || self.head[1] < -45.0 {
            self.dead = true;
            return Ok(());
        }
        let x_check: Vec<bool> = self.x.clone().into_iter().map(|x| {
            x > self.head[0] - 1.0 && x < self.head[0] + 1.0
        }).collect();

        let y_check: Vec<bool> = self.y.clone().into_iter().map(|x| {
            x > self.head[1] - 1.0 && x < self.head[1] + 1.0
        }).collect();
        for i in 0..self.x.len() {
            if y_check[i] && x_check[i] {
                self.dead = true;
                return Ok(());
            }
        }
        Ok(())
    }

    fn collision_check(&mut self) -> Result<()> {
        //TODO: implement
        if (self.head[0] > self.fruits[0] - 1.0 && self.head[0] < self.fruits[0] + 1.0) && (self.head[1] > self.fruits[1] - 1.0 && self.head[1] < self.fruits[1] + 1.0) {
            self.score += 100;
            self.update_enemies()?;
            self.append_segment()?;
            self.length += 1;
        }
        Ok(())
    }

    fn append_segment(&mut self) -> Result<()> {
        if self.length == 0 {
            self.x.push(self.head[0] - self.direction[0][0]);
            self.y.push(self.head[1] - self.direction[0][1]);
            self.direction.push(self.direction[self.length].clone());
        }
        else {
            self.x.push(self.x[self.length - 1] - self.direction[self.length][0]);
            self.y.push(self.y[self.length - 1] - self.direction[self.length][1]);
            self.direction.push(self.direction[self.length].clone());
        }
        Ok(())
    }

    fn update_position(&mut self) -> Result<()> {
        //TODO: implement
        self.head[0] += self.direction[0][0];
        self.head[1] += self.direction[0][1];
        self.update_segments()?;
        Ok(())
    }

    fn update_segments(&mut self) -> Result<()> {
        if self.length == 0 {
            return Ok(());
        }
        let last_dir = self.direction.clone();
        for i in 0..self.length{
            self.x[i] += self.direction[i + 1][0];
            self.y[i] += self.direction[i + 1][1];
            self.direction[i + 1] = last_dir[i].clone();
        }
        Ok(())
    }

    fn update_enemies(&mut self) -> Result<()> {
        //TODO: implement
        let mut rng = thread_rng();

        self.fruits[0] = rng.gen_range(-90.0..90.0);
        self.fruits[1] = rng.gen_range(-45.0..45.0);

        Ok(())
    }

    pub fn new() -> App {
        App {
            score: 0,
            highscore: 0,
            exit: false, 
            y: vec![],
            x: vec![],
            length: 0,
            on_puase: false,
            dead: false,
            auto: false,
            fruits: vec![0.0, 0.0],
            head: vec![1.0, 0.0],
            direction: vec![vec![1.0, 0.0]],
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Down => self.down()?,
            KeyCode::Up => self.up()?,
            KeyCode::Right => self.right()?,
            KeyCode::Left => self.left()?,
            KeyCode::Esc => self.pause()?,
            KeyCode::Enter => self.restart()?,
            KeyCode::Tab => self.auto()?,
            _ => {}
        }
        Ok(())
    }

    fn right(&mut self) -> Result<()> {
        if self.direction[0][0] == -1.0 {
            return Ok(());
        }
        self.direction[0][0] = 1.0;
        self.direction[0][1] = 0.0;
        Ok(())
    }

    fn up(&mut self) -> Result<()> {
        if self.direction[0][1] == -1.0 {
            return Ok(());
        }
        self.direction[0][0] = 0.0;
        self.direction[0][1] = 1.0;
        Ok(())
    }

    fn down(&mut self) -> Result<()> {
        if self.direction[0][1] == 1.0 {
            return Ok(());
        }
        self.direction[0][0] = 0.0;
        self.direction[0][1] = -1.0;
        Ok(())
    }

    fn left(&mut self) -> Result<()> {
        if self.direction[0][0] == 1.0 {
            return Ok(());
        }
        self.direction[0][0] = -1.0;
        self.direction[0][1] = 0.0;
        Ok(())
    }

    fn auto(&mut self) -> Result<()> {
        if self.auto {
            self.auto = false;
        }
        else {
            self.auto = true;
        }
        Ok(())
    }

    fn restart(&mut self) -> Result<()> {

        if self.dead {
            let path = Path::new("Highscore.bin");
            save(path, self.highscore)?;
            
            let num = read(path)?;

            self.dead = false;
            self.on_puase = false;
            self.y = vec![];
            self.x = vec![];
            self.fruits = vec![0.0, 0.0];
            self.score = 0;
            self.highscore = num;
            self.auto = false;
            self.length = 0;
            self.head = vec![0.0, 0.0];
            self.direction = vec![vec![1.0, 0.0]];
        }

        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn pause(&mut self) -> Result<()> {
        if self.on_puase {
            self.on_puase = false;
        }
        else {
            self.on_puase = true;
        }
        Ok(())
    }

}

fn autorun(app: &mut App) -> Result<()> {
    //TODO: implemmet
    Ok(())
}


/*
//TODO: refactor app class and extract player and enemies into seperate classes
#[derive(Debug, Default)]
pub struct Player {
    x: u8,
    y: u8,
    in_air: bool
}

impl Widget for &Player {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {
                let block = Block::bordered();
    }
}

impl Player {

    fn handle_events(&mut self) -> Result<()> {
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        Ok(())
    }

    fn jump(&mut self) -> Result<()> {
        Ok(())
    }

    fn duck(&mut self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Cactus {
    x: u8
}

impl Widget for &Cactus {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {
        
    }
}

#[derive(Debug, Default)]
pub struct Bird {
    x: u8,
    y: u8
}

impl Widget for &Bird {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {
        
    }
}

*/
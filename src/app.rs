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
    length: i64,
    enemies: Vec<Vec<f64>>,
    on_puase: bool,
    dead: bool,
    auto: bool,
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
                .style(Style::default().bg(player_color))
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
                        x: self.x[0],
                        y: self.y[0],
                        width: 10.0,
                        height: 1.0,
                        color: player_color,
                    });
                    ctx.layer();
                    ctx.draw(&canvas::Line {
                        x1: -90.0,
                        y1: -20.0,
                        x2: 90.0,
                        y2: -20.0,
                        color: Color::Green,
                    });
                    ctx.layer();
                    if self.enemies.len() > 0 {
                        for enemy in self.enemies.iter(){
                            ctx.draw(&canvas::Rectangle {
                                x: enemy[0],
                                y: enemy[2],
                                width: 2.0,
                                height: enemy[1],
                                color: Color::Red,
                            })
                        }
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
            let time = 5000;
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
            self.update_position()?;
            self.update_enemies()?;
            if self.collision_check() {
                self.dead = true;
            }
            self.score += 1;
            self.highscore();
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

    fn collision_check(&mut self) -> bool {
        //TODO: implement
        for enemy in self.enemies.iter() {
            if enemy[0] < 5.0 && enemy[0] > -5.0 {
                return true;
                }
            }
        false
    }

    fn update_position(&mut self) -> Result<()> {
        //TODO: implement
        Ok(())
    }

    fn update_enemies(&mut self) -> Result<()> {
        //TODO: implement
        let mut rng = thread_rng();
        let mut last_in_range: bool = false;
        let mut last_is_flying: bool = false;
        let mut last_one = 0.0;
        if self.enemies.len() > 0 {
            last_one = self.enemies[self.enemies.len() - 1][0];
            if last_one < 50.0 || last_one > 84.0 {
                last_in_range = true;
                if self.enemies[self.enemies.len() - 1][2] > -20.0 {
                    last_is_flying = true;
                }
            }
        }
        else {
            last_in_range = true;
        }
        
        if rng.gen_range(0.0..1.0) < 0.008 && last_in_range {
            let mut height = rng.gen_range(5.0..8.0);
            let flying = rng.gen_range(0.0..1.0);
            let mut y = -20.0;
            if (flying > 0.75 && flying < 0.82) && !(last_one > 84.0 && !last_is_flying){
                y = rng.gen_range(-12.0..-8.0);
                height = 1.0;
            }
            else if flying > 0.82 && !(last_one > 84.0 && !last_is_flying){
                y = rng.gen_range(0.0..5.0);
                height = 1.0;
            }
            self.enemies.push(vec![88.0, height, y]);
        }
        let mut count = 0;
        for  enemy in self.enemies.iter_mut() {
            if enemy[0] > - 90.0 {
                enemy[0] -= 1.0;
            }
            else {
                count += 1;
            }
        }
        for _ in 0..count {
            self.enemies.remove(0);
        }

        Ok(())
    }

    pub fn new() -> App {
        App {
            score: 0,
            highscore: 0,
            exit: false, 
            y: vec![-20.0],
            x: vec![2.0],
            length: 0,
            on_puase: false,
            dead: false,
            auto: false,
            enemies: vec![vec![9.0]]

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
        Ok(())
    }

    fn up(&mut self) -> Result<()> {
        Ok(())
    }

    fn down(&mut self) -> Result<()> {
        Ok(())
    }

    fn left(&mut self) -> Result<()> {
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
            self.y = vec![-20.0];
            self.x = vec![0.0];
            self.enemies = vec![];
            self.score = 0;
            self.highscore = num;
            self.auto = false;
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
    //TODO: implemment
    let mut enemies_in_front = vec![];

    if app.enemies.len() > 0 {
        for enemy in app.enemies.iter() {
            if enemy[0] > 5.0 {
                enemies_in_front.push(enemy);
            }
        }
        
        if enemies_in_front.len() > 0 {
            let closest_enemy: &Vec<f64> = enemies_in_front[0];
        }
    }

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
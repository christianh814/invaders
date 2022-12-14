use invaders::frame::{self, new_frame, Drawable};
use invaders::invaders::Invaders;
use invaders::player::Player;
use invaders::render::render;
use rusty_audio::Audio;
use std::error::Error;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::{io, thread};
//use std::sync::mpsc::{self, Receiver, Sender};
use crossterm::cursor::{Hide, Show};
use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::{terminal, ExecutableCommand};

fn main() -> Result<(), Box<dyn Error>> {
    let mut audio = Audio::new();
    audio.add("explode", "audio/explode.wav");
    audio.add("lose", "audio/lose.wav");
    audio.add("move", "audio/move.wav");
    audio.add("pew", "audio/pew.wav");
    audio.add("startup", "audio/startup.wav");
    audio.add("win", "audio/win.wav");
    audio.play("startup");

    // Terminal Setup
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    // Render Loop in a separate thread
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        let mut stdout = io::stdout();
        render(&mut stdout, &last_frame, &last_frame, true);
        loop {
            let curr_frame = match render_rx.recv() {
                Ok(x) => x,
                Err(_) => break,
            };
            render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });

    // Game loop
    let mut player = Player::new();
    let mut instant = Instant::now();
    let mut invaders = Invaders::new();

    'gameloop: loop {
        // per frame init
        let delta = instant.elapsed();
        instant = Instant::now();
        let mut curr_fame = new_frame();
        // Input handling
        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Left => player.move_left(),
                    KeyCode::Right => player.move_right(),
                    KeyCode::Char(' ') | KeyCode::Enter => {
                        if player.shoot() {
                            audio.play("pew");
                        }
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        audio.play("lose");
                        break 'gameloop;
                    }
                    _ => {}
                }
            }
        }

        //Updates
        player.update(delta);
        if invaders.update(delta) {
            audio.play("move");
        }
        if player.detect_hits(&mut invaders) {
            audio.play("explode");
        }

        // Draw and render
        let drawables: Vec<&dyn Drawable> = vec![&player, &invaders];
        for drawable in drawables {
            drawable.draw(&mut curr_fame);
        }
        let _ = render_tx.send(curr_fame);
        thread::sleep(Duration::from_millis(1));

        //Win/Lose conditions
        if invaders.all_killed() {
            audio.play("win");
            break 'gameloop
        }
        if invaders.reached_bottom() {
            audio.play("lose");
            break 'gameloop
        }
    }

    // Cleanup
    drop(render_tx);
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}

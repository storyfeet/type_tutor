use clap_conf::prelude::*;
use rand::prelude::*;
use std::io::Write;
use termion::color::{Bg, Red, Reset};
use termion::cursor::Goto;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

pub struct Word {
    s: String,
    x: u16,
    y: u16,
    speed: u16,
    dead: bool,
}

fn drop_last_char(s: &mut String) {
    let l = s.len();
    for x in 1..6 {
        if l < x {
            return;
        }
        if let Some(_) = s.as_str().get(l - x..) {
            s.remove(l - x);
            return;
        }
    }
}

fn main() {
    let clap = clap_app!(Type_Tutor =>
        (about:"A simple Typing Tutor")
        (version:crate_version!())
        (author:"Matthew Stoodley")
        (@arg file: -f --file +takes_value "The file containing the words")
        (@arg config: -c +takes_value "The locaion of the config file")
    )
    .get_matches();

    let cfg = with_toml_env(&clap, vec!["{HOME}/.config/type_tutor/init.toml"]);

    let word_list = match cfg.grab().arg("file").conf("file").done() {
        Some(fname) => {
            let fdata = std::fs::read_to_string(fname).expect("Could not read file");
            fdata
                .split("\n")
                .map(|sp| sp.trim().to_string())
                .collect::<Vec<String>>()
        }
        None => vec![
            "please", "use", "-f", "to ", "select", "a", "word", "list", "file",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect(),
    };

    let (ch_s, ch_r) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        for k in stdin.keys() {
            ch_s.send(k).expect("Could not send on channel");
        }
    });
    let mut score = 0;
    let mut lives = 6;

    let mut screen = std::io::stdout()
        .into_raw_mode()
        .expect("could not get raw mode");

    let mut words: Vec<Word> = Vec::new();
    let mut typing = String::new();
    let mut rng = rand::thread_rng();

    loop {
        while let Ok(Ok(k)) = ch_r.try_recv() {
            match k {
                Key::Esc => {
                    return;
                }
                Key::Char(' ') | Key::Char('\n') => {
                    words = words
                        .into_iter()
                        .filter(|w| {
                            if w.s == typing && !w.dead {
                                score += 1
                            }
                            w.s != typing || w.dead
                        })
                        .collect();
                    typing.clear();
                }
                Key::Char(c) => typing.push(c),
                Key::Backspace => {
                    drop_last_char(&mut typing);
                }
                _ => {}
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
        //maybe new word
        if rng.gen_range(0, 300 + score) < 20 + score {
            let y = rng.gen_range(2, 20);
            let s = word_list
                .choose(&mut rng)
                .expect("Wordlist empty")
                .to_string();

            if let Some(w) = words.iter_mut().find(|w| w.y == y) {
                w.speed += 1;
            } else if let Some(w) = words.iter_mut().find(|w| w.s == s) {
                w.speed += 1;
            } else {
                let nw = Word {
                    s,
                    x: 40,
                    y,
                    dead: false,
                    speed: 0,
                };
                words.push(nw);
            }
        }

        //move words and kill words

        for w in &mut words {
            if w.x > 0 && rng.gen_range(0, 700 + score) < 20 + score {
                w.x -= 1;
            }
            if w.x == 0 && !w.dead {
                lives -= 1;
                w.dead = true;
            }
        }

        if lives == 0 {
            break;
        }
        //print everything
        write!(
            screen,
            "{}{}Welcome to Type Tutor:    Score = {} Lives = {} \n\r>",
            termion::clear::All,
            Goto(1, 1),
            score,
            lives,
        )
        .expect("Could not clear screen");

        for w in &mut words {
            if w.dead {
                write!(screen, "{}{}{}{}", Goto(1, w.y), Bg(Red), w.s, Bg(Reset)).ok();
            } else {
                write!(screen, "{}{}", Goto(w.x, w.y), w.s).ok();
            }
        }
        write!(screen, "{}{}", Goto(0, 21), typing).ok();
        screen.flush().ok();
    }

    println!("Game over: your score = {}", score);
}

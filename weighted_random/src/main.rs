use std::{fs, io::stdout};
use crossterm::{
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    //style::Print,
    execute,
};

use weighted_random::*;

fn main() {
    println!("Loading data");
    let file = fs::read_to_string("data.json");
    let mut data = match file {
        Ok(file) => serde_json::from_str(&file).unwrap(),
        Err(_) => DataStorage::new(None),
    };
    let mut current_streak = 0;
    let mut points_achieved = 0;
    let mut questions_answered = 0;

    let mut stdout = stdout();
    enable_raw_mode().unwrap();
    let mut exiting = false;
    while !exiting {
        execute!(stdout, Clear(ClearType::All)).unwrap();
        println!("Points achieved: {}, Questions answered: {}, remaining items: {}, reamining weight: {}\r\n",
            points_achieved, questions_answered, data.get_remaining_items(), data.get_remaining_weight());
        println!("Press ctrl + 's' to save and exit, ctrl + 'a' to add new items, ctrl + 'r' to reset unused items or space to get a random item\r\n");
        
        let event = read().unwrap();
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('a'),
                modifiers: KeyModifiers::CONTROL, .. }) => {
                    disable_raw_mode().unwrap();
                    println!("Enter items separated by new lines, press enter twice to finish\r\n");
                    let mut items = Vec::new();
                    loop {
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input).unwrap();
                        if input.trim().is_empty() {
                            break;
                        }
                        items.push(input.trim().to_string());
                    }
                    data.insert_range(2, items);
                    enable_raw_mode().unwrap();
                },
            Event::Key(KeyEvent {
                code: KeyCode::Char('r'),
                modifiers: KeyModifiers::CONTROL, .. }) => {
                    println!("Type 'reset' to confirm\r\n");
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap();
                    if input.trim() == "reset" {
                        data.reset_unused_items();
                    }
                },
            Event::Key(KeyEvent {
                code: KeyCode::Char(' '), .. }) => { loop {
                    execute!(stdout, Clear(ClearType::All)).unwrap();
                    println!("Points achieved: {}, Questions answered: {}, remaining items: {}, reamining weight: {}\r\n",
                        points_achieved, questions_answered, data.get_remaining_items(), data.get_remaining_weight());
                    let (layer, index, item) = data.get_random();
                    println!("Tell me everything you know about {}\r\n", item);
                    println!("Press ' ' or enter if you know something about it or 'w' if you don't\r\n");
                    println!("Press 'q' or escape to go to the menu or ctrl + 's' to save and exit\r\n");
                    questions_answered += 1;
                    let inner_event = read().unwrap();
                    match inner_event {
                        Event::Key(KeyEvent {code: KeyCode::Char(' '), .. }) |
                        Event::Key(KeyEvent {code: KeyCode::Enter, .. }) => {
                            data.move_down(layer, index);
                            current_streak += 1;
                            points_achieved += fibonacci(current_streak+1);
                        },
                        Event::Key(KeyEvent {code: KeyCode::Char('w'), .. }) => {
                            data.move_up(layer, index);
                            current_streak = 0;
                        },
                        Event::Key(KeyEvent {code: KeyCode::Char('q'), .. }) |
                        Event::Key(KeyEvent {code: KeyCode::Esc, .. }) => {
                            questions_answered -= 1;
                            break;
                        },
                        Event::Key(KeyEvent {
                            code: KeyCode::Char('s'),
                            modifiers: KeyModifiers::CONTROL, .. }) => {
                                println!("Exiting\r\n");
                                exiting = true;
                                break;
                        },
                        _ => {},
                    }}
                },
            Event::Key(KeyEvent {
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::CONTROL, .. }) => {
                    println!("Exiting\r\n");
                    exiting = true;
                },
            _ => {}
        }
    }
    disable_raw_mode().unwrap();
    let file = serde_json::to_string(&data).unwrap();
    fs::write("data.json", file).unwrap();
}
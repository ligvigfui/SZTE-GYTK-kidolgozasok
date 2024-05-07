use std::{
    fs, io::{stdin, stdout}, time::{Duration, SystemTime}
};
use crossterm::{
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    //style::Print,
    execute,
};

use weighted_random::*;

struct File {
    file_name: String,
    message: String,
    data: DataStorage<String>,
}

fn load_file () -> File {
    println!("What file do you want to load? (type the number)\r");
    let files_in_folder = fs::read_dir("./datasets").unwrap();
    let mut files = Vec::new();
    for file in files_in_folder {
        let file = file.unwrap();
        let file_name = file.file_name();
        let file_name = file_name.to_str().unwrap().to_string();
        files.push(file_name);
    }
    println!("0: new file");
    for (i, file) in files.iter().enumerate() {
        println!("{}: {}", i + 1, file);
    }
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    let index = input.trim().parse::<usize>().unwrap_or_default();
    let mut file_name = Option::None;
    let mut data;
    let mut message = "".to_string();
    if index == 0 {
        data = DataStorage::new(None);
        while file_name == Option::None {
            println!("Name your file");
            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();
            match fs::write(format!("./datasets/{}", input), serde_json::to_string(&data).unwrap()) {
                Ok(_) => file_name = Option::Some(format!("{}.json", input.trim())),
                Err(e) => println!("Operation failed: {e}"),
            }
        }
    } else {
        file_name = Option::Some(files[index - 1].clone());
        let file = fs::read_to_string(format!("./datasets/{}", file_name.as_ref().unwrap()));
        data = match file {
            Ok(file) => match serde_json::from_str(&file) {
                Ok(file) => file,
                Err(e) => {
                    message = format!("{e}\r\nwe created a new file for you instead");
                    file_name = Some(format!("{:?}.json", SystemTime::now()));
                    DataStorage::new(None)
                },
            },
            Err(e) => {
                message = format!("{e}\r\nwe created a new file for you instead");
                file_name = Some(format!("{:?}.json", SystemTime::now()));
                DataStorage::new(None)
            },
        };
    }
    data.update_weight_sum();
    File {
        file_name: file_name.unwrap(),
        message,
        data
    }
}

fn main() {
    let mut file = load_file();
    let mut current_streak = 0;
    let mut points = 0;
    let mut answered = 0;

    let mut stdout = stdout();
    enable_raw_mode().unwrap();
    let mut exiting = false;
    execute!(stdout, Clear(ClearType::All)).unwrap();
    println!("{}\r", file.message);
    while !exiting {
        println!("Points: {}, Answered: {}, remaining: {}, remaining weight: {}\r",
            points, answered, &file.data.get_remaining_items(), &file.data.get_remaining_weight());
        println!("Press ctrl + 's' to save and exit, ctrl + 'a' to add new items, ctrl + 'r' to reset unused items or space to get a random item\r");
        
        let event = read().unwrap();
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('a'),
                modifiers: KeyModifiers::CONTROL, .. }) => {
                    disable_raw_mode().unwrap();
                    println!("Enter items separated by new lines, press enter twice to finish\r");
                    let mut items = Vec::new();
                    loop {
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input).unwrap();
                        if input.trim().is_empty() {
                            break;
                        }
                        items.push(input.trim().to_string());
                    }
                    file.data.insert_range(1, items);
                    save(&file);
                    enable_raw_mode().unwrap();
                },
            Event::Key(KeyEvent {
                code: KeyCode::Char('r'),
                modifiers: KeyModifiers::CONTROL, .. }) => {
                    println!("Type 'reset' to confirm\r");
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap();
                    if input.trim() == "reset" {
                        file.data.reset_unused_items();
                    }
                    save(&file);
                },
            Event::Key(KeyEvent {
                code: KeyCode::Char(' '), .. }) => { 
                    if file.data.get_remaining_items() == 0 {
                        println!("No more items to show\r");
                        read().unwrap();
                        continue;
                    }
                    loop {
                    execute!(stdout, Clear(ClearType::All)).unwrap();
                    println!("Points: {}, Answered: {}, remaining: {}, remaining weight: {}\r",
                        points, answered, &file.data.get_remaining_items(), &file.data.get_remaining_weight());
                    let (layer, index, item) = file.data.get_random();
                    println!("Tell me everything you know about {}\r", item);
                    println!("Press ' ' or enter if you know something about it or 'w' if you don't\r");
                    println!("Press 'q' or escape to go to the menu or ctrl + 's' to save and exit\r");
                    std::thread::sleep(Duration::from_millis(100));
                    answered += 1;
                    let inner_event = read().unwrap();
                    match inner_event {
                        Event::Key(KeyEvent {code: KeyCode::Char(' '), .. }) |
                        Event::Key(KeyEvent {code: KeyCode::Enter, .. }) => {
                            file.data.move_down(layer, index);
                            current_streak += 1;
                            points += fibonacci(current_streak+1) as usize;
                        },
                        Event::Key(KeyEvent {code: KeyCode::Char('w'), .. }) => {
                            file.data.move_up(layer, index);
                            current_streak = 0;
                        },
                        Event::Key(KeyEvent {code: KeyCode::Char('q'), .. }) |
                        Event::Key(KeyEvent {code: KeyCode::Esc, .. }) => {
                            answered -= 1;
                            break;
                        },
                        Event::Key(KeyEvent {
                            code: KeyCode::Char('s'),
                            modifiers: KeyModifiers::CONTROL, .. }) => {
                                println!("Exiting\r\n");
                                exiting = true;
                                break;
                        },
                        Event::Key(KeyEvent {
                            code: KeyCode::Char('r'),
                            modifiers: KeyModifiers::CONTROL, .. }) => {
                                points = 0;
                                current_streak = 0;
                                answered = 0;
                            },
                        _ => current_streak = 0,
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
        execute!(stdout, Clear(ClearType::All)).unwrap();
    }
    disable_raw_mode().unwrap();
    save(&file);
}

fn save(file: &File) {
    let file_data = serde_json::to_string(&file.data).unwrap();
    fs::write(format!("./datasets/{}", file.file_name), file_data).unwrap();
    println!("Saved {} file\r", file.file_name);
}
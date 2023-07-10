use crate::{println, print, api::console, io::keyboard::ScancodeStream};
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1, KeyCode};
use futures_util::{stream::Stream, StreamExt};

use alloc::string::String;

static mut SCANCODES: ScancodeStream = ScancodeStream {};
//let mut scancodes = ScancodeStream::new();
     //let mut keyboard: Keyboard<layouts::Us104Key, ScancodeSet1> =
//         Keyboard::new(HandleControl::Ignore);
pub async fn main() {
    console::clear();
    println!("WattleOS shell");
    unsafe {
        SCANCODES = ScancodeStream::new();
    }
    //let mut kb: Keyboard<layouts::Us104Key, ScancodeSet1> = Keyboard::new(HandleControl::Ignore);
    //println!("queue: {}", 'a');//unsafe{keyboard::QUEUE[0] as char});
    //crate::println!("awaiting keypresses...");
    //println!("queue: {}", input().await);
    //println!("queue: {}", '2');
    //let queue = messagequeue::get_message_queue();
    
    
    loop{
        update().await;
    };
}

pub async fn input() -> String {
    let mut keyboard: Keyboard<layouts::Us104Key, ScancodeSet1> = Keyboard::new(HandleControl::Ignore);
    let mut out: String = String::from("");

    unsafe {
        while let Some(scancode) = SCANCODES.next().await {
            if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
                if let Some(key) = keyboard.process_keyevent(key_event) {
                    match key {
                        DecodedKey::Unicode(character) => match character {

                            '\u{0008}' => {
                                if out.len() > 0 {
                                    console::back_space();
                                    out.pop();
                                }
                            },
                            '\n' => {
                                crate::print!("\n");
                                return out;
                            },
                            c => {
                                crate::print!("{}", character);
                                out.push(character);
                            }
                        }
                        DecodedKey::RawKey(key) => match key {
                            KeyCode::ArrowRight => {
                                //FBWRITER.try_get().unwrap().lock().move_cursor_right();
                            }
                            _ => crate::print!("RAW KEY: {:?}", key)
                        }
                    }
                }
            }
        }
    }

    out
}

pub async fn update() {
    print!("\n$ ");
    let command: String = input().await;

    if command == String::from("") {

    } else if command == String::from("help") {
        println!("commands:\n   clear");
    } else if command == String::from("clear") {
        console::clear();
    } else {
        println!("\"{}\" not found", command);
    }

    
    //print!("a");
}
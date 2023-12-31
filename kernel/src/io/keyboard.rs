use conquer_once::spin::OnceCell;
use futures_util::task::AtomicWaker;
use core::{pin::Pin, task::{Context, Poll}};
use crossbeam_queue::{ArrayQueue, PopError};
use futures_util::{stream::Stream, StreamExt};
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1, KeyCode};

const SCANCODE_QUEUE_SIZE: usize = 128;

static WAKER: AtomicWaker = AtomicWaker::new();
pub static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

pub struct ScancodeStream;

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(SCANCODE_QUEUE_SIZE))
            .expect("ScancodeStream::new should only be called once!");
        ScancodeStream {}
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, context: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("Scancode queue not initialized!");
        if let Ok(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }
        WAKER.register(&context.waker());
        match queue.pop() {
            Ok(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            Err(PopError) => Poll::Pending,
        }
    }
}

pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            crate::println!("Scancode queue full, dropping keyboard input!");
        } else {
            WAKER.wake();
        }
    } else {
        crate::println!("Scancode queue not initialized!");
    }
}

//pub async fn print_keypresses() {}

pub async fn print_keypresses() {
    //crate::println!("awaiting keypresses...");
    let mut scancodes = ScancodeStream::new();
    let mut keyboard: Keyboard<layouts::Us104Key, ScancodeSet1> = Keyboard::new(HandleControl::Ignore);

    //let queue = messagequeue::get_message_queue();

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => match character {

                        '\u{0008}' => {
                            x86_64::instructions::interrupts::without_interrupts(|| {
                                //crate::framebuffer::FBWRITER.try_get().unwrap().lock().back_space();
                            })
                        },
                        c => {
                            
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


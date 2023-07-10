#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

use bootloader_api::BootInfo;
use conquer_once::spin::OnceCell;

mod logger;
use crate::logger::init_logger;

mod io;
mod allocator;
mod memory;
mod task;
mod cpu;
mod api;
mod programs;

mod size;
use crate::size::*;

mod shell;

use io::{x2apic, acpi, keyboard, serial, vga};
use api::console;

extern crate alloc;

use alloc::vec::Vec;
use alloc::vec;

pub static TIMER_FN: OnceCell<fn()> = OnceCell::uninit();

const CONFIG: bootloader_api::BootloaderConfig = {
    let mut config = bootloader_api::BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(bootloader_api::config::Mapping::Dynamic);
    config.kernel_stack_size = Size::MiB(1).bytes() as u64; // 100 KiB
    //config.frame_buffer.width = core::prelude::v1::Some(800);
    //config.frame_buffer.width = core::prelude::v1::Some(600);
    config
};

bootloader_api::entry_point!(kernel_main, config = &CONFIG);

const VERSION: &str = "0.1.0";

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("{info}");
    hlt_loop();
}

fn init(boot_info: &'static BootInfo) {
    memory::init(boot_info);
    allocator::init_heap();
    let apic = acpi::init(boot_info);
    x2apic::init(&apic);
    cpu::init();
    vga::init(boot_info);
    console::init(console::palette::Flat);
    init_logger();
}

fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    init(boot_info);
    x86_64::instructions::interrupts::enable();

    vga::clear(console::palette().black);
    println!("WattleOS v{}", VERSION);

    //let shell = shell::Shell::init();
    
    /*TIMER_FN.init_once(|| {
        let func: fn() = function;
        func
    });*/

    /*
    println!("\x1b2;1mBlue        \x1b1;1m\x1b2;0mBlue\x1b0m");
    println!("\x1b2;2mGreen       \x1b1;2m\x1b2;0mGreen\x1b0m");
    println!("\x1b2;3mCyan        \x1b1;3m\x1b2;0mCyan\x1b0m");
    println!("\x1b2;4mRed         \x1b1;4m\x1b2;0mRed\x1b0m");
    println!("\x1b2;5mMagenta     \x1b1;5m\x1b2;0mMagenta\x1b0m");
    println!("\x1b2;6mBrown       \x1b1;6m\x1b2;0mBrown\x1b0m");
    println!("\x1b2;7mLight Gray  \x1b1;7m\x1b2;0mLight Gray\x1b0m");
    println!("\x1b2;8mDark Gray   \x1b1;8m\x1b2;0mDark Gray\x1b0m");
    println!("\x1b2;9mLight Blue  \x1b1;9m\x1b2;0mLight Blue\x1b0m");
    println!("\x1b2;amLight Green \x1b1;am\x1b2;0mLight Green\x1b0m");
    println!("\x1b2;bmLight Cyan  \x1b1;bm\x1b2;0mLight Cyan\x1b0m");
    println!("\x1b2;cmLight Red   \x1b1;cm\x1b2;0mLight Red\x1b0m");
    println!("\x1b2;dmPink        \x1b1;dm\x1b2;0mPink\x1b0m");
    println!("\x1b2;emYellow      \x1b1;em\x1b2;0mYellow\x1b0m");
    println!("\x1b2;fmWhite       \x1b1;fm\x1b2;0mWhite\x1b0m");
    */

    //vga::char_bitmap(0, 0, 2, 0xFF_FF_FF_FF, 0x00_00_00_FF, 'A');
    //vga::rect(0, 0, 100, 100, 0x27_AE_60_80);
    //shell_executor.spawn(task::Task::new(keyboard::print_keypresses()));
    let mut shell_executor = task::executor::Executor::new();
    shell_executor.spawn(task::Task::new(programs::shell::main()));
    shell_executor.run();

    //let mut shell_executor = task::executor::Executor::new();
    //shell_executor.spawn(task::Task::new(keyboard::print_keypresses()));
    //shell_executor.run();
    //screen::rectangle(0, 0, 100, 100, 0xFF_FF_FF_FF);

    hlt_loop();
}

async fn async_number() -> u32 {
    42
}

async fn console_update() {
    //screen::console::render();
    //screen::flip();
}
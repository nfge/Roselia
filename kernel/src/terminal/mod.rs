use crate::{
    kprint,
    ACPI_TABLE, MODULES, TERMINAL,
    cpu::{self},
    func::{get_time, poweroff, reset},
    gop::{color::Color, fonts::VGA_FONT, graphics::Graphics},
    keyboard::KeyBoard,
    log,
    memory::{get_heap_free, get_heap_used},
    ramfs::{check_directory, create_file, is_valid, mkdir, read_file, write_file},
    terminal::{command::Command, token::Token},
    timer::sleep,
};
use acpi::get_table;
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::fmt::{Arguments, Write};
use kernel_api::{
    acpi_tables::mcfg::Mcfg,
    keyboard::{
        keycode::{KeyCode, key_event_to_char},
        keyevent::KeyEvent,
    },
    module::{ACCEPT_ARGS, ModuleArgs},
};
use utils::serial_println;

mod command;
pub mod export;
mod lexer;
mod parser;
mod token;

pub struct Terminal {
    graphics: Graphics,
    keyboard: KeyBoard,
    x: usize,
    y: usize,
    scale: usize,
    color: Color,
    width: usize,
    height: usize,
    cols: usize,
    rows: usize,
    char_buffer: Vec<Vec<char>>,
    buf_x: usize,
    buf_y: usize,
    running: bool,
    pwd: String,
}

impl Terminal {
    pub fn new(graphics: Graphics, x: usize, y: usize, scale: usize, color: Color) -> Self {
        let (width, height) = graphics.mode_info.resolution();
        let cols = width / (8 * scale);
        let rows = height / (16 * scale);
        let mut pwd = String::new();
        pwd.push('/');
        Self {
            graphics: graphics,
            keyboard: KeyBoard::new(),
            x: x,
            y: y,
            scale: scale,
            color: color,
            width: width,
            height: height,
            cols: cols,
            rows: rows,
            char_buffer: vec![vec![' '; cols]; rows],
            buf_x: 0,
            buf_y: 0,
            running: false,
            pwd: pwd,
        }
    }
    pub fn write_char(&mut self, char: char) {
        match char {
            '\n' => self.new_line(),
            '\r' => {
                self.x = 0;
            }
            _ => {
                if self.buf_x >= self.cols {
                    self.new_line();
                }
                self.graphics
                    .draw_char(char, VGA_FONT, self.x, self.y, self.scale, self.color);
                self.x += 8 * self.scale;
                self.push(char);
            }
        }
    }
    pub fn write_string(&mut self, text: &str) {
        for c in text.chars() {
            self.write_char(c);
        }
    }
    pub fn print_char(&mut self, char: char) {
        self.write_char(char);
        self.graphics.present();
    }
    pub fn print_string(&mut self, text: &str) {
        self.write_string(text);
        self.graphics.present();
    }
    pub fn print_string_ln(&mut self, text: &str) {
        self.print_string(text);
        self.new_line();
    }
    pub fn print_fmt(&mut self, args: Arguments) {
        self.write_fmt(args).unwrap();
        self.graphics.present();
    }
    fn push(&mut self, c: char) {
        if self.buf_y >= self.rows || self.buf_x >= self.cols {
            return;
        }

        self.char_buffer[self.buf_y][self.buf_x] = c;
        self.buf_x += 1;
    }
    pub fn flush_screen(&mut self) {
        self.graphics.flush();
        self.graphics.present();
        self.char_buffer = vec![vec![' '; self.cols]; self.rows];
        self.buf_x = 0;
        self.buf_y = 0;
        self.x = 0;
        self.y = 0;
    }
    fn new_line(&mut self) {
        self.x = 0;
        self.buf_x = 0;

        if self.buf_y + 1 >= self.rows {
            self.scroll_up();
        } else {
            self.buf_y += 1;
            self.y += 16 * self.scale;
        }
    }
    fn scroll_up(&mut self) {
        self.char_buffer.remove(0);
        self.char_buffer.push(vec![' '; self.cols]);
        self.redraw();
    }
    fn redraw(&mut self) {
        self.graphics.flush();
        self.graphics.present();

        for row in 0..self.rows {
            for col in 0..self.cols {
                let c = self.char_buffer[row][col];
                if c != ' ' {
                    self.graphics.draw_char(
                        c,
                        VGA_FONT,
                        col * 8 * self.scale,
                        row * 16 * self.scale,
                        self.scale,
                        self.color,
                    );
                }
            }
        }
        self.buf_y = self.rows - 1;
        self.y = (self.rows - 1) * 16 * self.scale;
    }
    pub fn backspace(&mut self) {
        if self.buf_x == 0 && self.buf_y == 0 {
            return;
        }

        let char_width = 8 * self.scale;
        let char_height = 16 * self.scale;

        if self.buf_x == 0 {
            self.buf_y -= 1;
            self.buf_x = self.cols - 1;
            self.y -= char_height;
            self.x = char_width * (self.cols - 1);
        } else {
            self.buf_x -= 1;
            self.x -= char_width;
        }

        self.char_buffer[self.buf_y][self.buf_x] = ' ';

        for y in 0..char_height {
            for x in 0..char_width {
                self.graphics
                    .draw_pixel(self.x + x, self.y + y, Color::Black as u32);
            }
        }
        self.graphics.present();
    }
    #[allow(dead_code)]
    pub fn run(&mut self) {
        self.print_string_ln("Press Enter to start terminal");
        loop {
            match self.keyboard.get_key() {
                Some(event) => {
                    if KeyCode::Enter == event.code {
                        break;
                    }
                }
                _ => {
                    x86_64::instructions::hlt();
                }
            }
        }
        self.flush_screen();
        self.running = true;
        let cpwd = self.pwd.clone();
        kprint!("{}>", cpwd.as_str());
        while self.running {
            if let Some(key) = self.keyboard.get_key() {
                self.handle_keyboard(key);
            } else {
                x86_64::instructions::hlt();
            }
        }
    }
    fn handle_command(&mut self) {
        let mut line: String = String::new();
        for i in 0..self.buf_x {
            line.push(self.char_buffer[self.buf_y][i]);
        }
        let command_line = line
            .split_once('>')
            .map(|(_, s)| s.trim_start())
            .unwrap_or(&line);
        let tokens = lexer::Lexer::tokenize(&command_line);
        let command =
            parser::Parser::parse(tokens).unwrap_or(Command::new(" ".to_string(), Vec::new()));
        self.new_line();
        if command.name == " " {
            return;
        };
        match command.name.as_str() {
            "help" => self.print_string(
                "Commands: help, info, reset, poweroff, flush, time, date, mem, heap\n",
            ),
            "info" => match read_file("/kernel/info") {
                Ok(data) => {
                    let s = core::str::from_utf8(&data).unwrap();
                    kprint!("{s}");
                }
                Err(e) => {
                    kprint!("{:#?}\n", e);
                }
            },
            "reset" => {
                self.print_string_ln("Reseting...");
                sleep(1000);
                unsafe { reset() };
            }
            "poweroff" => unsafe { poweroff() },
            "flush" => self.flush_screen(),
            "cpu" => {
                let cpu = cpu::cpuinfo::get_cpu();
                let cpu_therm = cpu::cpuinfo::get_cpu_therm();
                let cpu_freq = cpu::cpuinfo::get_frequency();
                kprint!("Vendor: {}\n", cpu.0.unwrap().as_str());
                kprint!("Model: {}\n", cpu.1.unwrap().as_str());
                if cpu::cpuinfo::get_cpu().0.unwrap().as_str() == cpu::cpuinfo::INTEL {
                    let _ = self.print_fmt(format_args!("Temp: {}\n", cpu_therm.unwrap_or(0)));
                    kprint!(
                        "Freq:\n Bus: {}\n Base: {}\n Max: {}\n",
                        cpu_freq.0, cpu_freq.1, cpu_freq.2
                    );
                }
            }
            "echo" => {
                let text = match command.args.first() {
                    Some(text) => text,
                    None => {
                        self.print_string_ln("Usage: echo [str]");
                        return;
                    }
                };
                self.print_string_ln(text);
            }
            "cat" => match command.args.first() {
                Some(arg) => {
                    let target = if arg.starts_with('/') {
                        arg.clone()
                    } else {
                        let mut base = self.pwd.clone();
                        if !base.ends_with('/') {
                            base.push('/');
                        }
                        base + arg
                    };
                    match is_valid(&target) {
                        Ok(_) => match read_file(&target) {
                            Ok(data) => match core::str::from_utf8(&data) {
                                Ok(text) => {
                                    kprint!("{}", text);
                                }
                                Err(_) => {
                                    kprint!("Not a valid utf-8 file\n");
                                }
                            },
                            Err(e) => {
                                kprint!("{:#?}\n", e);
                            }
                        },
                        Err(e) => {
                            kprint!("{:#?}\n", e);
                        }
                    }
                }
                None => {
                    kprint!("missing file operand\n");
                }
            },
            "ls" => match command.args.first() {
                Some(path) => match check_directory(path.as_str()) {
                    Ok(nodes) => {
                        for node in nodes {
                            kprint!("{}\n", node.name.as_str());
                        }
                    }
                    Err(e) => {
                        kprint!("{:#?}\n", e);
                    }
                },
                None => {
                    let cpwd = self.pwd.clone();
                    match check_directory(&cpwd.as_str()) {
                        Ok(nodes) => {
                            for node in nodes {
                                kprint!("{}\n", node.name.as_str());
                            }
                        }
                        Err(e) => {
                            kprint!("{:#?}\n", e);
                        }
                    }
                }
            },
            "cd" => match command.args.first() {
                Some(s) => match is_valid(s.as_str()) {
                    Ok(_) => {
                        self.pwd = s.to_string();
                    }
                    Err(e) => {
                        kprint!("{:#?}\n", e);
                    }
                },
                None => {}
            },
            "time" => {
                let t = get_time();
                match t {
                    Ok(time) => {
                        kprint!("{}:{}:{}\n", time.hour, time.minute, time.second);
                    }
                    Err(_) => {
                        self.print_string_ln("Error during reading rtc");
                    }
                }
            }
            "date" => {
                let t = get_time();
                match t {
                    Ok(time) => {
                        kprint!("{}.{}.{}\n", time.day, time.month, time.year);
                    }
                    Err(_) => {
                        self.print_string_ln("Error during reading rtc");
                    }
                }
            }
            "scale" => {
                let scale = match command.args.first() {
                    Some(text) => text,
                    None => {
                        self.print_string_ln("Usage: scale [value]");
                        return;
                    }
                };
                if scale.parse::<usize>().unwrap() <= 0 {
                    self.print_string_ln("Scale must not be less than or equal to 0");
                    return;
                }
                if scale.parse::<usize>().unwrap() >= 12 {
                    self.print_string_ln("It is not recommended to set scale more than 12");
                    return;
                }
                self.scale = scale.parse::<usize>().unwrap();
                self.cols = self.width / (8 * self.scale);
                self.rows = self.height / (16 * self.scale);
                self.flush_screen();
            }
            "color" => {
                let color = match command.args.first() {
                    Some(text) => text.as_str(),
                    None => {
                        self.print_string_ln("Usage: color [value]");
                        return;
                    }
                };
                match color {
                    "white" => self.color = Color::White,
                    "red" => self.color = Color::Red,
                    "green" => self.color = Color::Green,
                    "blue" => self.color = Color::Blue,
                    "black" => self.color = Color::Black,
                    _ => self.print_string_ln("Color not found"),
                }
            }
            "sleep" => {
                let time = match command.args.first() {
                    Some(text) => text,
                    None => {
                        self.print_string_ln("Usage: sleep [value in ms]");
                        return;
                    }
                };
                sleep(time.parse::<u64>().unwrap())
            }
            "uptime" => {
                let ticks_per_sec =
                    crate::timer::TICKS_PER_SEC.load(core::sync::atomic::Ordering::Relaxed);
                let ticks = crate::timer::TICKS.load(core::sync::atomic::Ordering::Relaxed);
                let seconds = ticks / ticks_per_sec;
                kprint!("{:?}s\n", seconds);
            }
            "heap" => match command.args.first() {
                Some(text) => match text.as_str() {
                    "free" => {
                        kprint!("Free memory: {}KB\n", get_heap_free() / 1024);
                    }
                    "used" => {
                        kprint!("Used memory: {}KB\n", get_heap_used() / 1024);
                    }
                    _ => self.print_string_ln("Usage: heap [free || used]"),
                },
                None => self.print_string_ln("Usage: heap [free || used]"),
            },
            "mem" => match read_file("/sys/memory") {
                Ok(data) => {
                    let s = core::str::from_utf8(&data).unwrap();
                    kprint!("{}", s);
                }
                Err(e) => {
                    kprint!("{:#?}\n", e);
                }
            },
            "resolution" => {
                let width = self.width;
                let height = self.height;
                let cols = self.cols;
                let rows = self.rows;
                kprint!(
                    "Width: {}. Height: {} ({}x{} chars)",
                    width, height, cols, rows
                );
                self.new_line();
            }
            "pci" => {
                let mcfg_ptr = unsafe { get_table::<Mcfg>(ACPI_TABLE.unwrap(), b"MCFG").unwrap() };
                let mcfg = unsafe { &*mcfg_ptr };
                let count = unsafe { mcfg.entry_count() };
                for i in 0..count {
                    let entry = unsafe { &mcfg.entry(i) };
                    match command.args.first() {
                        Some(s) => {
                            if s.is_empty() {
                                return;
                            }
                            match s.as_str() {
                                "legacy" => {
                                    let devices = unsafe { pci::enumerate::enumerate_legacy() };
                                    for device in devices {
                                        let (vendor_name, device_name) = pci::check(
                                            device.header.vendor_id,
                                            device.header.device_id,
                                        );
                                        kprint!(
                                            "{}:{}.{}\n",
                                            device.bus, device.device, device.function
                                        );
                                        kprint!(
                                            "{:04x} {}\n{:04x} {}\n\n",
                                            device.header.vendor_id as u16,
                                            vendor_name.unwrap_or("Not found in pci.ids"),
                                            device.header.device_id as u16,
                                            device_name.unwrap_or("Not found in pci.ids")
                                        );
                                    }
                                }
                                "mcfg" => {
                                    let devices = unsafe { pci::enumerate::enumerate_mcfg(entry) };
                                    for device in devices {
                                        let (vendor_name, device_name) = pci::check(
                                            device.header.vendor_id,
                                            device.header.device_id,
                                        );
                                        kprint!(
                                            "{}:{}.{}\n",
                                            device.bus, device.device, device.function
                                        );
                                        kprint!(
                                            "{:04x} {}\n{:04x} {}\n\n",
                                            device.header.vendor_id as u16,
                                            vendor_name.unwrap_or("Not found in pci.ids"),
                                            device.header.device_id as u16,
                                            device_name.unwrap_or("Not found in pci.ids")
                                        );
                                    }
                                }
                                "id" => match command.args.get(1) {
                                    Some(s) => {
                                        let s = s as &str;
                                        if let Some((vendor, device)) = s.split_once(":") {
                                            let vendor_id =
                                                u16::from_str_radix(vendor, 16).unwrap();
                                            let device_id =
                                                u16::from_str_radix(device, 16).unwrap();
                                            let device = match pci::find_by_id(
                                                entry, vendor_id, device_id,
                                            ) {
                                                Some(data) => data,
                                                None => return self.print_string_ln("Not found"),
                                            };
                                            let (vendor_name, device_name) =
                                                pci::check(vendor_id, device_id);
                                            kprint!(
                                                "{}:{}.{}\n",
                                                device.bus, device.device, device.function
                                            );
                                            kprint!(
                                                "{:04x} {}\n{:04x} {}\n\n",
                                                device.header.vendor_id as u16,
                                                vendor_name.unwrap_or("Not found in pci.ids"),
                                                device.header.device_id as u16,
                                                device_name.unwrap_or("Not found in pci.ids")
                                            );
                                        } else {
                                            self.print_string_ln("Using: pci id vendor:device");
                                        }
                                    }
                                    None => self.print_string_ln("Using: pci id vendor:device"),
                                },
                                _ => self.print_string_ln("Using: pci [legacy || mcfg || id]"),
                            }
                        }
                        _ => self.print_string_ln("Using: pci [legacy || mcfg || id]"),
                    }
                }
            }
            "readlog" => {
                let data = read_file("/kernel/log").unwrap();
                let text = core::str::from_utf8(&data).unwrap();
                kprint!("{}", text);
            }
            "modinfo" => match command.args.first() {
                Some(s) => unsafe {
                    if let Some(modules) = &mut *core::ptr::addr_of_mut!(MODULES) {
                        for module in modules {
                            let name = core::str::from_utf8(
                                &module.info.name[..module
                                    .info
                                    .name
                                    .iter()
                                    .position(|&c| c == 0)
                                    .unwrap_or(module.info.name.len())],
                            )
                            .unwrap();
                            if name == s.as_str() {
                                kprint!(
                                    "Name: {}\nModule version: {}\nMagic: {}\nFlags: {}\n",
                                    name,
                                    module.info.module_version,
                                    module.info.magic,
                                    module.info.flags
                                );
                                return;
                            }
                        }
                        self.print_string_ln("Module not found");
                    }
                },
                None => self.print_string_ln("Usage modinfo [module name]"),
            }
            _ => {
                let name = command.name.as_str();
                if let Some(modules) = unsafe { &*core::ptr::addr_of_mut!(MODULES) } {
                    let mut found = false;
                    for module in modules {
                        let end = module
                            .info
                            .name
                            .iter()
                            .position(|&c| c == 0)
                            .unwrap_or(module.info.name.len());

                        if core::str::from_utf8(&module.info.name[..end]).unwrap() == name {
                            if module.info.flags & ACCEPT_ARGS != 0 {
                                let init: extern "C" fn(*const ModuleArgs) -> i32 =
                                    unsafe { core::mem::transmute(module.entry_fn) };
                                let raw_argv: Vec<Vec<u8>> = command
                                    .args
                                    .iter()
                                    .map(|s| {
                                        let mut bytes = s.as_bytes().to_vec();
                                        bytes.push(0);
                                        bytes
                                    })
                                    .collect();
                                let argv: Vec<*const u8> =
                                    raw_argv.iter().map(|s| s.as_ptr()).collect();
                                let args = ModuleArgs {
                                    argc: argv.len() as u64,
                                    argv: argv.as_ptr(),
                                };
                                let result = init(&args as *const ModuleArgs);
                                found = true;
                                if result == 0 {
                                    break;
                                } else {
                                    kprint!("Module exited with error code {}\n", result);
                                    break;
                                }
                            } else {
                                let init: extern "C" fn() -> i32 =
                                    unsafe { core::mem::transmute(module.entry_fn) };
                                let result = init();
                                found = true;
                                if result == 0 {
                                    break;
                                } else {
                                    kprint!("Module exited with error code {}\n", result);
                                    break;
                                }
                            }
                        }
                    }

                    if !found {
                        self.print_string_ln("Command not found");
                        return;
                    }
                }
            }
        }
    }
    fn handle_keyboard(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Enter => {
                if !event.shift {
                    self.handle_command();
                    if self.running {
                        let cpwd = self.pwd.clone();
                        kprint!("{}>", cpwd.as_str());
                    } else {
                        return;
                    }
                } else {
                    self.new_line();
                }
            }
            KeyCode::Escape => {}
            KeyCode::Backspace => {
                self.backspace();
            }
            KeyCode::ArrowUp => {}
            KeyCode::ArrowDown => {}
            KeyCode::ArrowLeft => {}
            KeyCode::ArrowRight => {}
            _ => {
                self.print_char(key_event_to_char(event).unwrap());
            }
        }
    }
    pub fn set_cursor(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
        self.buf_x = x;
        self.buf_y = y;
    }
    pub fn set_cursor_cell(&mut self, cell_x: Option<usize>, cell_y: Option<usize>) {
        if let Some(cell_x) = cell_x {
            self.x = cell_x * 8;
            self.buf_x = cell_x * 8;
        }
        if let Some(cell_y) = cell_y {
            self.y = cell_y * 16;
            self.buf_y = cell_y * 16;
        }
    }
    // pub fn clear_line(&mut self) {
    //     let char_width = 8;
    //     let char_height = 16;
    //     for px in 0..(self.x * char_width) {
    //         for py in 0..char_height {
    //             self.graphics
    //                 .draw_pixel(px, self.y + py, Color::Black as u32);
    //         }
    //     }
    //     self.x = 0;
    //     self.buf_x = 0;
    // }
}

impl core::fmt::Write for Terminal {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {{
        if unsafe { !$crate::TERMINAL.is_null() } {
            let term = unsafe {$crate::TERMINAL};
            unsafe { let _ = (*term).print_fmt(format_args!($($arg)*)); };
        }
    }};
}

#[macro_export]
macro_rules! kprintln {
    ($($arg:tt)*) => {{
        if unsafe { !$crate::TERMINAL.is_null() } {
            let term = unsafe {$crate::TERMINAL};
            unsafe {
                let _ = (*term).print_fmt(format_args!($($arg)*));
                let _ = (*term).print_fmt(format_args!("\n"));
            };
        }
    }};
}

use crate::{
    ACPI_TABLE, GET_VAR_FN, RAMFS, RESET_FN, SET_VAR_FN, TERMINAL, TIME_FN, cpu,
    func::{get_time, reset, s5_soft_off},
    gop::{
        color::Color,
        fonts::{VGA_FONT, font8x16::FONT8X16},
        graphics::Graphics,
    },
    keyboard::KeyBoard,
    log,
    memory::{get_free, get_used},
    ramfs::{create_file, mkdir, read_file, write_file},
    terminal::{command::Command, token::Token},
    timer::sleep,
};
use acpi_tables::{get_table, mcfg::Mcfg};
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::fmt::Write;

mod command;
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
}

impl Terminal {
    pub fn new(graphics: Graphics, x: usize, y: usize, scale: usize, color: Color) -> Self {
        let (width, height) = graphics.mode_info.resolution();
        let cols = width / (8 * scale);
        let rows = height / (16 * scale);
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
        }
    }
    pub fn print_char(&mut self, char: char) {
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
    pub fn print_string(&mut self, text: &str) {
        for c in text.chars() {
            self.print_char(c);
        }
    }
    pub fn print_string_ln(&mut self, text: &str) {
        self.print_string(text);
        self.new_line();
    }
    fn push(&mut self, c: char) {
        if self.buf_y >= self.rows || self.buf_x >= self.cols {
            return;
        }

        self.char_buffer[self.buf_y][self.buf_x] = c;
        self.buf_x += 1;
    }
    // fn push_command(&mut self, c: char){
    //     if self.cmd_buf_len < self.cmd_buffer.len() {
    //         self.cmd_buffer[self.cmd_buf_len] = c;
    //         self.cmd_buf_len += 1;
    //     }
    // }
    pub fn flush_screen(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.graphics.draw_pixel(x, y, Color::Black as u32);
            }
        }
        self.char_buffer = vec![vec![' '; self.cols]; self.rows];
        self.buf_x = 0;
        self.buf_y = 0;
        self.x = 0;
        self.y = 0;
    }
    // fn flashback(&mut self) {
    //     for y in 0..self.graphics.mode_info.resolution().1 {
    //         for x in 0..self.graphics.mode_info.resolution().0 {
    //             self.graphics.draw_pixel(x, y, Color::White as u32);
    //         }
    //     }
    //     sleep(700);
    //     self.flush_screen();
    // }
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
        for y in 0..self.height {
            for x in 0..self.width {
                self.graphics.draw_pixel(x, y, Color::Black as u32);
            }
        }

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
    }
    #[allow(dead_code)]
    pub fn run(&mut self) {
        self.print_string_ln("Press Enter to start terminal");
        loop {
            match self.keyboard.get_key() {
                Some('\n') => break,
                _ => {
                    x86_64::instructions::hlt();
                }
            }
        }
        self.flush_screen();
        self.running = true;
        self.print_char('>');
        while self.running {
            if self.keyboard.key_state.get_ctrl() && self.keyboard.key_state.get_shift() {
                match self.keyboard.get_key() {
                    Some('C') => {
                        self.new_line();
                        self.print_string_ln("Interrupt detected. Reseting...");
                        sleep(1000);
                        unsafe { reset() };
                    }
                    _ => {}
                }
            }
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
        if command.name == " " {return};
        match command.name.as_str() {
            "help" => self
                .print_string("Commands: help, info, reset, poweroff, flush, time, date, heap\n"),
            "info" => {
                let _ = write!(
                    self,
                    "Roselia Kernel {} ({})",
                    env!("CARGO_PKG_VERSION"),
                    env!("GIT_COMMIT")
                );
                self.new_line();
            }
            "reset" => {
                self.print_string_ln("Reseting...");
                sleep(1000);
                unsafe { reset() };
            }
            "poweroff" => unsafe { s5_soft_off() },
            "flush" => self.flush_screen(),
            "cpu" => {
                let cpu = cpu::cpuinfo::get_cpu();
                let cpu_therm = cpu::cpuinfo::get_cpu_therm();
                let _ = write!(self, "Vendor: {}\n", cpu.0.unwrap().as_str());
                let _ = write!(self, "Model: {}\n", cpu.1.unwrap().as_str());
                let _ = write!(self, "Temp: {}\n", cpu_therm.unwrap_or(0));
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
            "time" => {
                let t = get_time();
                match t {
                    Ok(time) => {
                        let _ = write!(self, "{}:{}:{}\n", time.hour, time.minute, time.second);
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
                        let _ = write!(self, "{}-{}-{}\n", time.year, time.month, time.day);
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
                let _ = write!(self, "{:?}s\n", seconds);
            }
            "panic" => panic!(),
            "heap" => match command.args.first() {
                Some(text) => match text.as_str() {
                    "free" => {
                        let _ = write!(self, "Free memory: {}KB\n", get_free() / 1024);
                    }
                    "used" => {
                        let _ = write!(self, "Used memory: {}KB\n", get_used() / 1024);
                    }
                    _ => self.print_string_ln("Usage: heap [free || used]"),
                },
                None => self.print_string_ln("Usage: heap [free || used]"),
            },
            "resolution" => {
                let width = self.width;
                let height = self.height;
                let cols = self.cols;
                let rows = self.rows;
                let _ = write!(
                    self,
                    "Width: {}. Height: {} ({}x{} chars)",
                    width, height, cols, rows
                );
                self.new_line();
            }
            "game" => {
                let mut x = self.x;
                let key_x: usize = 96;

                loop {
                    self.set_cursor(key_x, self.y);
                    self.print_string("K");
                    self.set_cursor(x, self.y);
                    match self.keyboard.get_key() {
                        Some('w') => {
                            self.clear_line();
                            x += 8;
                            self.set_cursor(x, self.y);
                            self.print_string("P");
                        }
                        Some('s') => {
                            self.clear_line();
                            x -= 8;
                            self.set_cursor(x, self.y);
                            self.print_string("P");
                        }
                        Some('z') => break,
                        _ => {
                            x86_64::instructions::hlt();
                        }
                    }
                    if x == key_x {
                        self.new_line();
                        self.print_string_ln("You win");
                        break;
                    }
                }
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
                                    let devices = unsafe { pci::enumerate_legacy() };
                                    for device in devices {
                                        let (vendor_name, device_name) = pci::check(
                                            device.header.vendor_id,
                                            device.header.device_id,
                                        );
                                        let _ = write!(
                                            self,
                                            "Bus: {}, device: {}, function: {}\n",
                                            device.bus, device.device, device.function
                                        );
                                        let _ = write!(
                                            self,
                                            "{:04x} {}\n{:04x} {}\n\n",
                                            device.header.vendor_id as u16,
                                            vendor_name.unwrap_or("Not found in pci.ids"),
                                            device.header.device_id as u16,
                                            device_name.unwrap_or("Not found in pci.ids")
                                        );
                                    }
                                }
                                "mcfg" => {
                                    let devices = unsafe { pci::enumerate_mcfg(entry) };
                                    for device in devices {
                                        let (vendor_name, device_name) = pci::check(
                                            device.header.vendor_id,
                                            device.header.device_id,
                                        );
                                        let _ = write!(
                                            self,
                                            "Bus: {}, device: {}, function: {}\n",
                                            device.bus, device.device, device.function
                                        );
                                        let _ = write!(
                                            self,
                                            "{:04x} {}\n{:04x} {}\n\n",
                                            device.header.vendor_id as u16,
                                            vendor_name.unwrap_or("Not found in pci.ids"),
                                            device.header.device_id as u16,
                                            device_name.unwrap_or("Not found in pci.ids")
                                        );
                                    }
                                }
                                _ => self.print_string_ln("Using: pci [legacy || mcfg]"),
                            }
                        }
                        _ => self.print_string_ln("Using: pci [legacy || mcfg]"),
                    }
                }
            }
            _ => self.print_string("Command not found\n"),
        }
    }
    fn handle_keyboard(&mut self, char: char) {
        match char {
            '\n' => {
                if !self.keyboard.key_state.get_shift() {
                    self.handle_command();
                    if self.running {
                        self.print_char('>');
                    } else {
                        return;
                    }
                } else {
                    self.new_line();
                }
            }
            '\x08' => {
                self.backspace();
            }
            _ => {
                self.print_char(char);
            }
        }
    }
    pub fn set_cursor(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
        self.buf_x = x;
        self.buf_y = y;
    }
    pub fn clear_line(&mut self) {
        let char_width = 8;
        let char_height = 16;
        for px in 0..(self.x * char_width) {
            for py in 0..char_height {
                self.graphics
                    .draw_pixel(px, self.y + py, Color::Black as u32);
            }
        }
        self.x = 0;
        self.buf_x = 0;
    }
}

impl core::fmt::Write for Terminal {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.print_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        if unsafe { !$crate::TERMINAL.is_null() } {
            let term = unsafe {$crate::TERMINAL};
            unsafe { let _ = write!((*term),  $($arg)*); };
        }
    }};
}

#[macro_export]
macro_rules! kprintln {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        if unsafe { !$crate::TERMINAL.is_null() } {
            let term = unsafe {$crate::TERMINAL};
            unsafe {
                let _ = write!((*term),  $($arg)*);
                let _ = write!((*term), "\n");
            };
        }
    }};
}

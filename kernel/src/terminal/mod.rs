use crate::{
    RESET_FN, TIME_FN, cpu,
    gop::{color::Color, fonts::font8x16::FONT8X16, graphics::Graphics},
    keyboard::KeyBoard,
    timer::sleep,
};
use heapless::String;
use uefi::Status;

pub struct Terminal {
    graphics: Graphics,
    keyboard: KeyBoard,
    x: usize,
    y: usize,
    scale: usize,
    color: Color,
    char_buffer: [[char; 64]; 64],
    buf_x: usize,
    buf_y: usize,
    running: bool,
}

impl Terminal {
    pub fn new(graphics: Graphics, x: usize, y: usize, scale: usize, color: Color) -> Self {
        Self {
            graphics: graphics,
            keyboard: KeyBoard::new(),
            x: x,
            y: y,
            scale: scale,
            color: color,
            char_buffer: [[' '; 64]; 64],
            buf_x: 0,
            buf_y: 0,
            running: false,
        }
    }
    pub fn print_char(&mut self, char: char) {
        match char {
            '\n' => self.new_line(),
            '>' => {
                self.graphics
                    .draw_char(char, FONT8X16, self.x, self.y, self.scale, self.color);
                self.x += 8 * self.scale;
            }
            _ => {
                self.graphics
                    .draw_char(char, FONT8X16, self.x, self.y, self.scale, self.color);
                self.x += 8 * self.scale;
                self.push(char);
            }
        }
    }
    pub fn print_string(&mut self, text: &str) {
        for c in text.chars() {
            self.graphics
                .draw_char(c, FONT8X16, self.x, self.y, self.scale, self.color);
            self.x += 8 * self.scale;
            self.push(c);
        }
        if text.contains("\n") {
            self.new_line();
        }
    }
    pub fn print_string_ln(&mut self, text: &str) {
        self.print_string(text);
        self.new_line();
    }
    fn push(&mut self, c: char) {
        if self.buf_y >= 64 {
            return;
        }

        self.char_buffer[self.buf_y][self.buf_x] = c;
        self.buf_x += 1;

        if self.buf_x >= 64 {
            self.buf_x = 0;
            self.buf_y += 1;
        }
    }
    // fn push_command(&mut self, c: char){
    //     if self.cmd_buf_len < self.cmd_buffer.len() {
    //         self.cmd_buffer[self.cmd_buf_len] = c;
    //         self.cmd_buf_len += 1;
    //     }
    // }
    pub fn flush_screen(&mut self) {
        for y in 0..self.graphics.mode_info.resolution().1 {
            for x in 0..self.graphics.mode_info.resolution().0 {
                self.graphics.draw_pixel(x, y, Color::Black as u32);
            }
        }
        self.char_buffer = [[' '; 64]; 64];
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
        self.y += 16 * self.scale;
        self.x = 0;

        self.buf_x = 0;
        self.buf_y += 1;
    }
    pub fn backspace(&mut self) {
        if self.buf_x == 0 && self.buf_y == 0 {
            return;
        }

        let char_width = 8 * self.scale;
        let char_height = 16 * self.scale;

        if self.buf_x == 0 {
            self.buf_y -= 1;
            self.buf_x = 63;
            self.y -= char_height;
            self.x = char_width * 63;
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
        self.running = true;
        self.print_char('>');
        while self.running {
            if let Some(key) = self.keyboard.get_key() {
                self.handle_keyboard(key);
            }
        }
    }
    fn handle_command(&mut self) {
        let mut line: String<64> = String::new();
        for i in 0..self.buf_x {
            line.push(self.char_buffer[self.buf_y][i]).ok();
        }
        let mut args = line.as_str().split_whitespace();
        self.new_line();

        match args.next() {
            Some(v) => match v {
                "help" => self.print_string("Commands: help, info, reset, flush,time\n"),
                "info" => self.print_string("Kernel 0.2. Made by nfge\n"),
                "reset" => {
                    let typeofreset = match args.next() {
                        Some(v) => v,
                        None => {
                            self.print_string_ln("Usage: reset [shutdown || cold || warm]");
                            return;
                        }
                    };
                    self.print_string("Shutdown...\n");
                    sleep(900);
                    match typeofreset {
                        "cold" => {
                            unsafe {
                                RESET_FN.unwrap()(
                                    uefi::runtime::ResetType::COLD,
                                    Status::SUCCESS,
                                    None,
                                )
                            };
                        }
                        "warm" => {
                            unsafe {
                                RESET_FN.unwrap()(
                                    uefi::runtime::ResetType::WARM,
                                    Status::SUCCESS,
                                    None,
                                )
                            };
                        }
                        "shutdown" => {
                            unsafe {
                                RESET_FN.unwrap()(
                                    uefi::runtime::ResetType::SHUTDOWN,
                                    Status::SUCCESS,
                                    None,
                                )
                            };
                        }
                        _ => {
                            self.print_string_ln("Error");
                        }
                    }
                }
                "flush" => self.flush_screen(),
                "cpu" => {
                    let cpu = cpu::cpuinfo::get_cpu();
                    self.print_string_ln(cpu.0.unwrap().as_str());
                    self.print_string_ln(cpu.1.unwrap().as_str());
                }
                "print" => {
                    let text = match args.next() {
                        Some(v) => v,
                        None => {
                            self.print_string_ln("Usage: print [str]");
                            return;
                        }
                    };
                    self.print_string_ln(text);
                }
                "time" => {
                    let mut buf = itoa::Buffer::new();
                    let t = unsafe { TIME_FN.unwrap()() };
                    let time = t.unwrap().0;
                    self.print_string("Year: ");
                    self.print_string_ln(buf.format(time.year()));
                    self.print_string("Month: ");
                    self.print_string_ln(buf.format(time.month()));
                    self.print_string("Day: ");
                    self.print_string_ln(buf.format(time.day()));
                    self.print_string("Hour: ");
                    self.print_string_ln(buf.format(time.hour()));
                    self.print_string("Seconds: ");
                    self.print_string_ln(buf.format(time.second()));
                }
                "scale" => {
                    let scale = match args.next() {
                        Some(v) => v,
                        None => {
                            self.print_string_ln("Usage: scale [value]");
                            return;
                        }
                    };
                    if scale.parse::<usize>().unwrap() <= 0 {
                        self.print_string_ln("Scale must not be less than or equal to 0");
                        return;
                    }
                    self.scale = scale.parse::<usize>().unwrap();
                }
                "color" => {
                    let color = match args.next() {
                        Some(v) => v,
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
                    let time = match args.next() {
                        Some(v) => v,
                        None => {
                            self.print_string_ln("Usage: sleep [value in ms]");
                            return;
                        }
                    };
                    sleep(time.parse::<u64>().unwrap())
                }
                "prompt" => {
                    self.print_string_ln("Mango Fonk");
                    for _ in 0..10 {
                        self.print_string("Mango ");
                    }
                }
                _ => self.print_string("Command not found\n"),
            },
            None => {
                return;
            }
        }
    }
    fn handle_keyboard(&mut self, char: char) {
        match char {
            '\n' => {
                if !self.keyboard.key_state.get_shift() {
                    self.handle_command();
                    self.print_char('>');
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
}

impl core::fmt::Write for Terminal {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.print_string(s);
        Ok(())
    }
}

use super::key_table::Key;

pub(super) static SCANCODE_TABLE: &[Key] = &[
    Key { scancode: 0x1E, letter: 'a'}, // standard
    Key { scancode: 0x30, letter: 'b'},
    Key { scancode: 0x2E, letter: 'c'},
    Key { scancode: 0x20, letter: 'd'},
    Key { scancode: 0x12, letter: 'e'},
    Key { scancode: 0x21, letter: 'f'},
    Key { scancode: 0x22, letter: 'g'},
    Key { scancode: 0x23, letter: 'h'},
    Key { scancode: 0x17, letter: 'i'},
    Key { scancode: 0x24, letter: 'j'},
    Key { scancode: 0x25, letter: 'k'},
    Key { scancode: 0x26, letter: 'l'},
    Key { scancode: 0x32, letter: 'm'},
    Key { scancode: 0x31, letter: 'n'},
    Key { scancode: 0x18, letter: 'o'},
    Key { scancode: 0x19, letter: 'p'},
    Key { scancode: 0x10, letter: 'q'},
    Key { scancode: 0x13, letter: 'r'},
    Key { scancode: 0x1F, letter: 's'},
    Key { scancode: 0x14, letter: 't'},
    Key { scancode: 0x16, letter: 'u'},
    Key { scancode: 0x2F, letter: 'v'},
    Key { scancode: 0x11, letter: 'w'},
    Key { scancode: 0x2D, letter: 'x'},
    Key { scancode: 0x15, letter: 'y'},
    Key { scancode: 0x2C, letter: 'z'},
    Key { scancode: 0x39, letter: ' '}, // special
    Key { scancode: 0x1C, letter: '\n'}, 
    Key { scancode: 0x0E, letter: '\x08'},
    // Key { scancode: 0x2A, letter: '\0'},
    Key { scancode: 0x02, letter: '1'}, // numbers
    Key { scancode: 0x03, letter: '2'},
    Key { scancode: 0x04, letter: '3'},
    Key { scancode: 0x05, letter: '4'},
    Key { scancode: 0x06, letter: '5'},
    Key { scancode: 0x07, letter: '6'},
    Key { scancode: 0x08, letter: '7'},
    Key { scancode: 0x09, letter: '8'},
    Key { scancode: 0x0A, letter: '9'},
    Key { scancode: 0x0B, letter: '0'},
    Key { scancode: 0x4B, letter: '\0'}, // arrows
    Key { scancode: 0x4D, letter: '\0'},
    Key { scancode: 0x48, letter: '\0'},
    Key { scancode: 0x50, letter: '\0'},
    Key { scancode: 0x0C, letter: '-'}, // symbols
    Key { scancode: 0x0D, letter: '='},
    Key { scancode: 0x33, letter: ','},
    Key { scancode: 0x34, letter: '.'},
    Key { scancode: 0x28, letter: '\''}
];
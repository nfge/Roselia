pub fn check_special(c: char) -> Option<char> {
    match c {
        '\'' => Some('"'),
        ';' => Some(':'),
        '1' => Some('!'),
        '2' => Some('@'),
        '3' => Some('#'),
        '4' => Some('$'),
        '5' => Some('%'),
        '6' => Some('^'),
        '7' => Some('&'),
        '8' => Some('*'),
        '9' => Some('('),
        '0' => Some(')'),
        '-' => Some('_'),
        '=' => Some('+'),
        '[' => Some('{'),
        ']' => Some('}'),
        ',' => Some('<'),
        '.' => Some('>'),
        _ => return None
    }
}
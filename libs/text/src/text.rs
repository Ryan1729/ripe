pub mod byte_slice {
    pub fn reflow(bytes: &[u8], width: usize) -> Vec<u8> {
        if width == 0 || bytes.is_empty() {
            return Vec::new();
        }

        let mut output = Vec::with_capacity(bytes.len() + bytes.len() / width);

        let mut x = 0;
        for word in split_whitespace(bytes) {
            x += word.len();

            if x == width && x == word.len() {
                output.extend(word.iter());
                continue;
            }

            if x >= width {
                output.push(b'\n');

                x = word.len();
            } else if x > word.len() {
                output.push(b' ');

                x += 1;
            }
            output.extend(word.iter());
        }

        output
    }

    pub fn split_whitespace(bytes: &[u8]) -> impl Iterator<Item = &[u8]> {
        bytes
            .split(|b| b.is_ascii_whitespace())
            .filter(|word| !word.is_empty())
    }

    pub fn lines(bytes: &[u8]) -> impl Iterator<Item = &[u8]> {
        bytes.split(|&b| b == b'\n')
    }
}

pub mod string {
    pub fn reflow(bytes: &str, width: usize) -> String {
        if width == 0 || bytes.is_empty() {
            return String::new();
        }

        let mut output = String::with_capacity(bytes.len() + bytes.len() / width);

        let mut x = 0;
        for word in split_whitespace(bytes) {
            x += word.len();

            if x == width && x == word.len() {
                output.push_str(word);
                continue;
            }

            if x >= width {
                output.push('\n');

                x = word.len();
            } else if x > word.len() {
                output.push(' ');

                x += 1;
            }
            output.push_str(word);
        }

        output
    }

    pub fn split_whitespace(bytes: &str) -> impl Iterator<Item = &str> {
        bytes
            .split_ascii_whitespace()
            .filter(|word| !word.is_empty())
    }

    pub fn lines(bytes: &str) -> impl Iterator<Item = &str> {
        bytes.lines()
    }
}
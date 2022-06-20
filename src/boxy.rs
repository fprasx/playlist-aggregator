// Because you can't type these easily
pub const UL: &str = "┌";
pub const UR: &str = "┐";
pub const DL: &str = "└";
pub const DR: &str = "┘";
pub const H: &str = "─";
pub const V: &str = "│";

pub struct BoxConfig {
    line: usize,
    indent: String,
    pad_ud: usize,
    pad_lr: usize,
    ripples: usize,
    box_height: usize,
    height: usize,
    text: String,
    len: usize,
}

// TODO: handle multiline
impl BoxConfig {
    pub fn new(indent: usize, ripples: usize, pad_ud: usize, pad_lr: usize, text: String) -> Self {
        let len = text.len();
        Self {
            line: 0,
            indent: " ".repeat(indent),
            pad_ud,
            pad_lr,
            ripples,
            text,
            len,
            box_height: 2 + 1 + 2 * pad_ud,
            height: 2 + 1 + 2 * pad_ud + ripples,
        }
    }

    fn line(&mut self, text: &str) {
        let tail = if self.line < self.box_height {
            self.line
        } else if self.line > self.height - self.box_height {
            self.height - self.line
        } else {
            self.box_height - 1
        };
        println!("{}{text}{}", self.indent, " │".repeat(tail));
        self.line += 1;
    }

    fn ripples(&mut self) {
        for i in (1..self.ripples + 1).rev() {
            self.line(&format!(
                "{}{}{UR}",
                " ".repeat(2 * i),
                H.repeat(self.len + self.pad_lr * 2 + 2 - 1),
            ))
        }
    }

    fn box_text(&mut self) {
        self.line(&format!(
            "{}{}{}",
            UL,
            H.repeat(self.len + 2 * self.pad_lr),
            UR
        ));
        for _ in 0..self.pad_ud {
            self.line(&format!("{V}{}{V}", " ".repeat(self.len + 2 * self.pad_lr)))
        }
        self.line(&format!(
            "{V}{}{}{}{V}",
            " ".repeat(self.pad_lr),
            self.text,
            " ".repeat(self.pad_lr)
        ));
        for _ in 0..self.pad_ud {
            self.line(&format!("{V}{}{V}", " ".repeat(self.len + 2 * self.pad_lr)))
        }
        self.line(&format!(
            "{}{}{}",
            DL,
            H.repeat(self.len + 2 * self.pad_lr),
            DR
        ));
    }

    pub fn draw(&mut self) {
        self.ripples();
        self.box_text();
        // Reset the line we're on to 0 so we can start again
        self.line = 0;
    }
}
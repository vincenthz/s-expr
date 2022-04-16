use super::data::GroupKind;

/// Simple printer
#[derive(Clone)]
pub struct Printer {
    buf: String,
    prev: PrinterState,
}

#[derive(Clone, PartialEq, Eq)]
pub enum PrinterState {
    Group,
    Text,
}

impl Default for Printer {
    fn default() -> Self {
        Self {
            buf: String::new(),
            prev: PrinterState::Group,
        }
    }
}

impl Printer {
    /// Create a new group
    pub fn open(&mut self, grp: GroupKind) {
        if self.prev == PrinterState::Text {
            self.buf.push(' ');
        }
        let c = match grp {
            GroupKind::Paren => '(',
            GroupKind::Bracket => '[',
            GroupKind::Brace => '{',
        };
        self.prev = PrinterState::Group;
        self.buf.push(c);
    }

    /// Close a group
    pub fn close(&mut self, grp: GroupKind) {
        let c = match grp {
            GroupKind::Paren => ')',
            GroupKind::Bracket => ']',
            GroupKind::Brace => '}',
        };
        self.prev = PrinterState::Group;
        self.buf.push(c);
    }

    /// Add text
    pub fn text(&mut self, s: &str) {
        if self.prev == PrinterState::Text {
            self.buf.push(' ');
        }
        self.prev = PrinterState::Text;
        self.buf.push_str(s)
    }

    pub fn to_string(self) -> String {
        self.buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t1() {
        let mut p = Printer::default();
        p.open(GroupKind::Paren);
        p.close(GroupKind::Paren);
        let out = p.to_string();
        assert_eq!(out, "()");
    }

    #[test]
    fn t2() {
        let mut p = Printer::default();
        p.open(GroupKind::Paren);
        p.text("let");
        p.text("x");
        p.text("=");
        p.text("1");
        p.close(GroupKind::Paren);
        let out = p.to_string();
        assert_eq!(out, "(let x = 1)");
    }

    #[test]
    fn t3() {
        let mut p = Printer::default();
        p.open(GroupKind::Paren);
        p.text("let");
        p.text("x");
        p.text("=");
        p.open(GroupKind::Paren);
        p.text("+");
        p.text("1");
        p.text("0xabc");
        p.close(GroupKind::Paren);
        p.close(GroupKind::Paren);
        let out = p.to_string();
        assert_eq!(out, "(let x = (+ 1 0xabc))");
    }
}

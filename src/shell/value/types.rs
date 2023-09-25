use std::fmt;

bitflags::bitflags! {
    #[rustfmt::skip]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Type: u16 {
        const NULL =        0b000000000001;
        const INT =         0b000000000010;
        const FLOAT =       0b000000000100;
        const BOOL =        0b000000001000;
        const STRING =      0b000000010000;
        const LIST =        0b000000100000;
        const MAP =         0b000001000000;
        const TABLE =       0b000010000000;
        const RANGE =       0b000100000000;
        const REGEX =       0b001000000000;
        const BINARY =      0b010000000000;
        const CLOSURE =     0b100000000000;

        const ANY = Self::NULL.bits() | Self::INT.bits() | Self::FLOAT.bits() | Self::BOOL.bits() | Self::STRING.bits() | Self::LIST.bits() | Self::MAP.bits() | Self::TABLE.bits() | Self::RANGE.bits() | Self::RANGE.bits() | Self::BINARY.bits();
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut is_first = false;

        if self.intersects(Self::NULL) {
            write!(f, "`null`")?;
            is_first = true;
        }

        if self.intersects(Self::INT) {
            if is_first {
                write!(f, " or ")?;
            }
            is_first = true;
            write!(f, "`int`")?;
        }

        if self.intersects(Self::FLOAT) {
            if is_first {
                write!(f, " or ")?;
            }
            is_first = true;
            write!(f, "`float`")?;
        }

        if self.intersects(Self::STRING) {
            if is_first {
                write!(f, " or ")?;
            }
            is_first = true;
            write!(f, "`string`")?;
        }

        if self.intersects(Self::LIST) {
            if is_first {
                write!(f, " or ")?;
            }
            is_first = true;
            write!(f, "`list`")?;
        }

        if self.intersects(Self::MAP) {
            if is_first {
                write!(f, " or ")?;
            }
            is_first = true;
            write!(f, "`map`")?;
        }

        if self.intersects(Self::TABLE) {
            if is_first {
                write!(f, " or ")?;
            }
            is_first = true;
            write!(f, "`table`")?;
        }

        if self.intersects(Self::RANGE) {
            if is_first {
                write!(f, " or ")?;
            }
            write!(f, "`range`")?;
        }

        if self.intersects(Self::REGEX) {
            if is_first {
                write!(f, " or ")?;
            }
            write!(f, "`regex`")?;
        }

        if self.intersects(Self::BINARY) {
            if is_first {
                write!(f, " or ")?;
            }
            write!(f, "`binary`")?;
        }

        if self.intersects(Self::CLOSURE) {
            if is_first {
                write!(f, " or ")?;
            }
            write!(f, "`closure`")?;
        }

        Ok(())
    }
}

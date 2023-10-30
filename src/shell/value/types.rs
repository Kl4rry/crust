use std::fmt;

bitflags::bitflags! {
    #[rustfmt::skip]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Type: u16 {
        const NULL =        1 << 0;
        const INT =         1 << 1;
        const FLOAT =       1 << 2;
        const BOOL =        1 << 3;
        const STRING =      1 << 4;
        const LIST =        1 << 5;
        const MAP =         1 << 6;
        const TABLE =       1 << 7;
        const RANGE =       1 << 8;
        const REGEX =       1 << 9;
        const BINARY =      1 << 10;
        const CLOSURE =     1 << 11;

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

use std::fmt;

use bitflags::bitflags;

bitflags! {
    #[rustfmt::skip]
    pub struct Type: u16 {
        const NULL =        0b0000000001;
        const INT =         0b0000000010;
        const FLOAT =       0b0000000100;
        const BOOL =        0b0000001000;
        const STRING =      0b0000010000;
        const LIST =        0b0000100000;
        const MAP =         0b0001000000;
        const TABLE =       0b0010000000;
        const RANGE =       0b0100000000;
        const REGEX =       0b1000000000;
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut is_first = false;

        if self.intersects(Self::NULL) {
            write!(f, "'null'")?;
            is_first = true;
        }

        if self.intersects(Self::INT) {
            if is_first {
                write!(f, " or ")?;
            }
            is_first = true;
            write!(f, "'int'")?;
        }

        if self.intersects(Self::FLOAT) {
            if is_first {
                write!(f, " or ")?;
            }
            is_first = true;
            write!(f, "'float'")?;
        }

        if self.intersects(Self::STRING) {
            if is_first {
                write!(f, " or ")?;
            }
            is_first = true;
            write!(f, "'string'")?;
        }

        if self.intersects(Self::LIST) {
            if is_first {
                write!(f, " or ")?;
            }
            is_first = true;
            write!(f, "'list'")?;
        }

        if self.intersects(Self::MAP) {
            if is_first {
                write!(f, " or ")?;
            }
            is_first = true;
            write!(f, "'map'")?;
        }

        if self.intersects(Self::TABLE) {
            if is_first {
                write!(f, " or ")?;
            }
            is_first = true;
            write!(f, "'table'")?;
        }

        if self.intersects(Self::RANGE) {
            if is_first {
                write!(f, " or ")?;
            }
            write!(f, "'range'")?;
        }

        Ok(())
    }
}

use std::fmt;
use std::ops::Deref;

#[derive(Copy, Clone, Debug)]
pub struct Register(pub u8);

impl Deref for Register {
    type Target = u8;

    fn deref(&self) -> &u8 {
        &self.0
    }
}

impl Register {
    pub fn as_u32(self) -> u32 {
        *self as u32
    }

    pub fn from_str(r: &str) -> Option<Self> {
        Some(Register(match r {
            "X0" => 0,
            "X1" => 1,
            "X2" => 2,
            "X3" => 3,
            "X4" => 4,
            "X5" => 5,
            "X6" => 6,
            "X7" => 7,
            "X8" => 8,
            "X9" => 9,
            "X10" => 10,
            "X11" => 11,
            "X12" => 12,
            "X13" => 13,
            "X14" => 14,
            "X15" => 15,
            "X16" | "IP0" => 16,
            "X17" | "IP1" => 17,
            "X18" => 18,
            "X19" => 19,
            "X20" => 20,
            "X21" => 21,
            "X22" => 22,
            "X23" => 23,
            "X24" => 24,
            "X25" => 25,
            "X26" => 26,
            "X27" => 27,
            "X28" | "SP" => 28,
            "X29" | "FR" => 29,
            "X30" | "LR" => 30,
            "X31" | "XZR" => 31,
            _ => return None,
        }))
    }

    pub fn to_str(i: usize) -> String {
        let special = match i {
            16 => "(IP0) ",
            17 => "(IP1) ",
            28 => " (SP) ",
            29 => " (FR) ",
            30 => " (LR) ",
            31 => "(XZR) ",
            _ => "      ",
        };
        format!("{}X{}:{}", special, i, if i < 10 { " " } else { "" })
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            16 => write!(f, "IP0"),
            17 => write!(f, "IP1"),
            28 => write!(f, "SP"),
            29 => write!(f, "FR"),
            30 => write!(f, "LR"),
            31 => write!(f, "XZR"),
            _ => write!(f, "X{}", self.0),
        }
    }
}

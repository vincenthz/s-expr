/// Type of group
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum GroupKind {
    /// Group of ()
    Paren,
    /// Group of {}
    Brace,
    /// Group of []
    Bracket,
}

/// Atom literal (Number, Bytes, String, or Ident)
#[derive(Clone, Debug)]
pub enum Atom<'a> {
    /// Integral number literal
    Integral(ANum<'a>),
    /// Decimal number literal (e.g. `12.34`)
    Decimal(ADecimal<'a>),
    /// Bytes literal
    Bytes(ABytes<'a>),
    /// String literal
    String(AStr<'a>),
    /// Ident
    Ident(&'a str),
}

impl<'a> Atom<'a> {
    /// Get the Number in an Atom if the right variant, or None
    pub fn number(&self) -> Option<&ANum<'a>> {
        match self {
            Atom::Integral(num) => Some(num),
            _ => None,
        }
    }

    /// Get the Decimal in an Atom if the right variant, or None
    pub fn decimal(&self) -> Option<&ADecimal<'a>> {
        match self {
            Atom::Decimal(dec) => Some(dec),
            _ => None,
        }
    }

    /// Get the Bytes in an Atom if the right variant, or None
    pub fn bytes(&self) -> Option<&ABytes<'a>> {
        match self {
            Atom::Bytes(bytes) => Some(bytes),
            _ => None,
        }
    }

    /// Get the String in an Atom if the right variant, or None
    pub fn string(&self) -> Option<&AStr<'a>> {
        match self {
            Atom::String(str) => Some(str),
            _ => None,
        }
    }

    /// Get the Ident in an Atom if the right variant, or None
    pub fn ident(&self) -> Option<&'a str> {
        match self {
            Atom::Ident(ident) => Some(ident),
            _ => None,
        }
    }
}

/// A String literal, that may contains escapes
#[derive(Clone, Debug)]
pub struct AStr<'a> {
    pub has_escape: bool,
    pub raw_data: &'a str,
}

impl<'a> AStr<'a> {
    pub fn to_string(&self) -> String {
        self.raw_data.to_string()
    }
}

/// A Bytes literal
#[derive(Clone, Debug)]
pub struct ABytes<'a>(pub &'a str);

/// Supported number base
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ANumBase {
    /// Binary Base (2), made of '0'..'1'
    Binary = 2,
    /// Decimal Base (10), made of '0'..'9'
    Decimal = 10,
    /// Hexadecimal Base (16), made of '0'..'9', 'a'..'f', 'A'..'F'
    Hexadecimal = 16,
}

impl ANumBase {
    /// Return the radix number associated with the support base
    pub fn to_radix(self) -> u32 {
        self as u32
    }

    pub fn from_radix(v: u32) -> Option<Self> {
        if v == 2 {
            Some(Self::Binary)
        } else if v == 10 {
            Some(Self::Decimal)
        } else if v == 16 {
            Some(Self::Hexadecimal)
        } else {
            None
        }
    }
}

/// Integral Number
#[derive(Clone, Debug)]
pub struct ANum<'a> {
    pub base: ANumBase,
    pub dat: &'a str,
}

impl<'a> ANum<'a> {
    /// Get the base of the data
    pub fn base(&self) -> ANumBase {
        self.base
    }

    /// Get the radix of the data, which is either 2 (binary), 10 (decimal) or 16 (hexadecimal)
    pub fn radix(&self) -> u32 {
        self.base.to_radix()
    }

    /// Get the data associated with the number, which depending on the radix is
    /// either binary, decimal and hexadecimal. it also might contains _ separators
    pub fn raw_data(&self) -> &'a str {
        self.dat
    }

    /// Get the digits associated with the number, which depending on the radix is
    /// either binary, decimal and hexadecimal. The '_' characters are filtered away
    pub fn digits(&self) -> String {
        self.dat.chars().filter(|c| *c != '_').collect::<String>()
    }

    /// Try to parse the ANum into a u8, which will raise an error if there's an overflow
    pub fn to_u8(&self) -> Result<u8, core::num::ParseIntError> {
        u8::from_str_radix(&self.digits(), self.base.to_radix())
    }

    /// Try to parse the ANum into a u16, which will raise an error if there's an overflow
    pub fn to_u16(&self) -> Result<u16, core::num::ParseIntError> {
        u16::from_str_radix(&self.digits(), self.base.to_radix())
    }

    /// Try to parse the ANum into a u32, which will raise an error if there's an overflow
    pub fn to_u32(&self) -> Result<u32, core::num::ParseIntError> {
        u32::from_str_radix(&self.digits(), self.base.to_radix())
    }

    /// Try to parse the ANum into a u64, which will raise an error if there's an overflow
    pub fn to_u64(&self) -> Result<u64, core::num::ParseIntError> {
        u64::from_str_radix(&self.digits(), self.base.to_radix())
    }

    /// Try to parse the ANum into a u128, which will raise an error if there's an overflow
    pub fn to_u128(&self) -> Result<u128, core::num::ParseIntError> {
        u128::from_str_radix(&self.digits(), self.base.to_radix())
    }
}

/// Decimal Number (e.g. `1.3`)
#[derive(Clone, Debug)]
pub struct ADecimal<'a> {
    pub raw_integral: &'a str,
    pub raw_fractional: &'a str,
}

impl<'a> ADecimal<'a> {
    /// Get the data associated with the integral number. All '_' characters are filtered away
    pub fn integral(&self) -> String {
        self.raw_integral
            .chars()
            .filter(|c| *c != '_')
            .collect::<String>()
    }

    /// Get the data associated with the fractional number. All '_' characters are filtered away
    pub fn fractional(&self) -> String {
        self.raw_fractional
            .chars()
            .filter(|c| *c != '_')
            .collect::<String>()
    }
}

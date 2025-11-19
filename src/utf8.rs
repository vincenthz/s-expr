//! need utf8 parsing capability, as the tokenizer work on bytes slice
//! but the utf8 module of rust is private / ongoing stabilisation,
//! so borrow the table and the utf8_char_width function to
//! write our own parser

// * https://tools.ietf.org/html/rfc3629
// * accessible tweaked copy of https://doc.rust-lang.org/src/core/str/validations.rs.html#246
const UTF8_CHAR_WIDTH: &[u8; 256] = &[
    // 1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 1
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 2
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 3
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 4
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 5
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 6
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 7
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 8
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 9
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // A
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // B
    0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // C
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // D
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // E
    4, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // F
];

const fn utf8_char_width(b: u8) -> usize {
    UTF8_CHAR_WIDTH[b as usize] as usize
}

#[allow(unused)]
pub(crate) const fn extended_math_operator(ch: char) -> bool {
    let c = ch as u32;
    (c >= 0x2200 && c <= 0x22FF) || (c >= 0x2A00 && c <= 0x2AFF)
}

#[allow(unused)]
pub(crate) const fn extended_math_alphanumeric(ch: char) -> bool {
    let c = ch as u32;
    (c >= 0x1D400 && c <= 0x1D7FF)
}

#[derive(Debug, Clone)]
pub enum NextCharError {
    EmptyDataStream,
    IncompleteUtf8Sequence(u8),
    InvalidUtf8Sequence,
    InvalidUtf8ContByte,
}

#[derive(Clone, Copy)]
pub(crate) struct MovementInBytes(pub(crate) usize);

/// produce the next character available in the data stream and the number of bytes to advance,
/// or if anything wrong with the input stream in term of size or validation, an error
pub(crate) fn next_char(
    data: &[u8],
    index: usize,
) -> Result<Option<(char, MovementInBytes)>, NextCharError> {
    // if we're not at the end of the stream, there should be at least 1 byte
    if index == data.len() {
        return Ok(None);
    }
    if index > data.len() {
        return Err(NextCharError::EmptyDataStream);
    }

    // get the UTF8 leading character to find how many bytes we need to consume
    let h = data[index];
    let nb_chars = utf8_char_width(h);

    if nb_chars == 0 {
        return Err(NextCharError::InvalidUtf8Sequence);
    }

    // check if we can consume $nb_chars bytes
    if index + nb_chars > data.len() {
        return Err(NextCharError::IncompleteUtf8Sequence(h));
    }

    const fn mask_cont(v: u8) -> u32 {
        (v & 0b0011_1111) as u32
    }

    const fn mask_head(mask: u8, v: u8) -> u32 {
        (v & mask) as u32
    }

    const fn is_cont(v: u8) -> bool {
        const CONT_MASK: u8 = 0b1100_0000;
        const CONT_ESEQ: u8 = 0b1000_0000;
        (v & CONT_MASK) == CONT_ESEQ
    }

    const fn u32_to_char(
        c: u32,
        nb_chars: usize,
    ) -> Result<Option<(char, MovementInBytes)>, NextCharError> {
        // const-hack since map is not const
        match char::from_u32(c) {
            None => Err(NextCharError::InvalidUtf8Sequence),
            Some(c) => Ok(Some((c, MovementInBytes(nb_chars)))),
        }
    }

    const MASK2: u8 = 0b0001_1111;
    const MASK3: u8 = 0b0000_1111;
    const MASK4: u8 = 0b0000_0111;

    match nb_chars {
        0 => Err(NextCharError::InvalidUtf8Sequence),
        1 => Ok(Some((h.into(), MovementInBytes(nb_chars)))),
        2 => {
            let b2 = data[index + 1];
            if !is_cont(b2) {
                return Err(NextCharError::InvalidUtf8ContByte);
            }
            u32_to_char(mask_head(MASK2, h) << 6 | mask_cont(b2), nb_chars)
        }
        3 => {
            let b2 = data[index + 1];
            let b3 = data[index + 2];
            if !is_cont(b2) || !is_cont(b3) {
                return Err(NextCharError::InvalidUtf8ContByte);
            }
            u32_to_char(
                mask_head(MASK3, h) << 12 | mask_cont(b2) << 6 | mask_cont(b3),
                nb_chars,
            )
        }
        4 => {
            let b2 = data[index + 1];
            let b3 = data[index + 2];
            let b4 = data[index + 3];
            if !is_cont(b2) || !is_cont(b3) || !is_cont(b4) {
                return Err(NextCharError::InvalidUtf8ContByte);
            }
            u32_to_char(
                mask_head(MASK4, h) << 18
                    | mask_cont(b2) << 12
                    | mask_cont(b3) << 6
                    | mask_cont(b4),
                nb_chars,
            )
        }
        _ => Err(NextCharError::InvalidUtf8Sequence),
    }
}

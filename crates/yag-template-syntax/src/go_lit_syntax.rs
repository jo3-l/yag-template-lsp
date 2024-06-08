use std::borrow::Cow;
use std::error::Error;
use std::{fmt, iter};

use rowan::{TextRange, TextSize};
use unscanny::{Pattern, Scanner};

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum EscapeError {
    LoneSlash,
    NewlineAfterSlash,
    TooShort,
    OutOfRangeOctalEscape,
    RepresentsInvalidChar,
    UnrecognizedEscape,
}

impl fmt::Display for EscapeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use EscapeError::*;
        f.write_str(match self {
            LoneSlash => "unexpected lone `\\`",
            NewlineAfterSlash => "unexpected newline after `\\`",
            TooShort => "escape sequence too short",
            OutOfRangeOctalEscape => "octal escape value over 255",
            RepresentsInvalidChar => "escape sequence represents invalid unicode character",
            UnrecognizedEscape => "unrecognized escape sequence",
        })
    }
}

impl Error for EscapeError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub(crate) enum EscapeContext {
    StringLiteral,
    CharacterLiteral,
}

/// Iterate over the escape sequences that appear in the source string, where
/// the sequences are interpreted according to the [Go specification for
/// interpreted string and rune literals].
///
/// [Go specification]: https://go.dev/ref/spec#Rune_literals
pub(crate) fn iter_escape_sequences<'s>(src: &'s str, ctx: EscapeContext) -> EscapeSequences<'s> {
    EscapeSequences {
        s: Scanner::new(src),
        ctx,
    }
}

pub(crate) struct EscapeSequences<'s> {
    s: Scanner<'s>,
    ctx: EscapeContext,
}

impl<'s> Iterator for EscapeSequences<'s> {
    // each iteration produces `(range of escape sequence, unescaped character or error)`
    type Item = (TextRange, Result<char, EscapeError>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(slash_offset) = self.s.after().find('\\') {
            self.s.jump(self.s.cursor() + slash_offset);

            let start = TextSize::new(self.s.cursor() as u32);
            self.s.expect('\\');
            let result = match self.s.eat() {
                Some(c) => scan_escape_sequence(&mut self.s, c, self.ctx),
                None => Err(EscapeError::LoneSlash),
            };
            let end = TextSize::new(self.s.cursor() as u32);
            Some((TextRange::new(start, end), result))
        } else {
            None
        }
    }
}

/// Scan an escape sequence in a string or a rune literal. The caller should
/// already have read the slash and the first character after. That is, given an
/// escape sequence such as `\u0021`, the cursor should be at `\u|0021`.
pub(crate) fn scan_escape_sequence(
    s: &mut Scanner,
    char_after_backslash: char,
    ctx: EscapeContext,
) -> Result<char, EscapeError> {
    let unescaped = match char_after_backslash {
        'a' => '\u{0007}',
        'b' => '\u{0008}',
        'f' => '\u{000C}',
        'n' => '\n',
        'r' => '\r',
        't' => '\t',
        'v' => '\u{000B}',
        '\\' => '\\',
        '"' if ctx == EscapeContext::StringLiteral => '"',
        '\'' if ctx == EscapeContext::CharacterLiteral => '\'',
        'x' => scan_hex_escape(s, 2)?,
        'u' => scan_hex_escape(s, 4)?,
        'U' => scan_hex_escape(s, 8)?,
        '0'..='7' => {
            let octal_digits = s.eat_while_bounded(|c| matches!(c, '0'..='7'), 3);
            if octal_digits.len() < 3 {
                return Err(EscapeError::TooShort);
            }

            let value = u32::from_str_radix(octal_digits, 8)
                .expect("octal_digits always represent an octal number fitting in a u32");
            if value > 255 {
                return Err(EscapeError::OutOfRangeOctalEscape);
            }
            char::from_u32(value).ok_or(EscapeError::RepresentsInvalidChar)?
        }
        '\n' => return Err(EscapeError::NewlineAfterSlash),
        _ => return Err(EscapeError::UnrecognizedEscape),
    };
    Ok(unescaped)
}

fn scan_hex_escape(s: &mut Scanner, expected_digits: usize) -> Result<char, EscapeError> {
    let hex_digits = s.eat_while_bounded(char::is_ascii_hexdigit, expected_digits);
    if hex_digits.len() < expected_digits {
        return Err(EscapeError::TooShort);
    }

    let value = u32::from_str_radix(hex_digits, 16)
        .expect("hex_digits should always represent a hexadecimal number fitting in a u32");
    char::from_u32(value).ok_or(EscapeError::RepresentsInvalidChar)
}

trait ScannerExt<'s> {
    fn eat_while_bounded<T>(&mut self, pat: impl Pattern<T> + Copy, max: usize) -> &'s str;
}

impl<'s> ScannerExt<'s> for Scanner<'s> {
    fn eat_while_bounded<T>(&mut self, pat: impl Pattern<T> + Copy, max: usize) -> &'s str {
        let start = self.cursor();
        let mut consumed = 0;
        while self.at(pat) && consumed < max {
            self.eat();
            consumed += 1;
        }
        self.from(start)
    }
}

/// Return a copy of the string — which is expected to be a Go string literal
/// with the opening and closing `"` delimiters stripped — with escape sequences
/// replaced with the character they represent (or REPLACEMENT CHARACTER U+FFFD
/// in the case of erroneous escape sequences.)
///
/// See also [iter_escape_sequences].
pub(crate) fn interpret_string_content<'s>(src: &'s str) -> Cow<'s, str> {
    if src.find('\\').is_none() {
        return Cow::Borrowed(src);
    }

    let mut out = String::with_capacity(src.len());
    let mut last_end = 0;
    for (range, unescaped) in iter_escape_sequences(src, EscapeContext::StringLiteral) {
        out.push_str(&src[last_end..range.start().into()]);
        out.push(unescaped.unwrap_or('\u{FFFD}'));
        last_end = range.end().into();
    }
    out.push_str(&src[last_end..]);
    Cow::Owned(out)
}

/// Make a best effort to parse the input as a Go integer literal.
pub(crate) fn parse_int(input: &str) -> Result<i64, ()> {
    let (base, stripped) = prepare_for_parse(input);
    // prepare_for_parse changes the input, so the error message can be
    // misleading; replace it with Err(())
    i64::from_str_radix(&stripped, base).map_err(|_| ())
}

/// Make a best effort to parse the input as a Go floating-point literal.
///
/// TODO: Support hexadecimal floating-point literals?
pub(crate) fn parse_float(input: &str) -> Result<f64, ()> {
    let (base, stripped) = prepare_for_parse(input);
    if base == 10 {
        stripped.parse().map_err(|_| ())
    } else {
        Err(())
    }
}

pub(crate) fn scan_numeric_base_prefix(s: &mut Scanner) -> Option<u32> {
    let base = if s.eat_if("0x") || s.eat_if("0X") {
        16
    } else if s.eat_if("0o") || s.eat_if("0O") {
        8
    } else if s.eat_if("0b") || s.eat_if("0B") {
        2
    } else {
        return None;
    };
    Some(base)
}

/// Return `(base, input with base prefix and underscores stripped)`.
fn prepare_for_parse(input: &str) -> (u32, String) {
    let mut s: Scanner = Scanner::new(input);
    let mut buf = String::with_capacity(input.len());
    if s.eat_if(['+', '-']) {
        buf.push_str(s.before());
    }

    let base = scan_numeric_base_prefix(&mut s);

    // Go permits an underscore after the base prefix or between successive
    // digits; see https://go.dev/ref/spec#Integer_literals. Strip these out
    // before passing to Rust's parser.
    if base.is_some() {
        s.eat_if('_');
    }
    buf.extend(skip_underscores_between_digits(s.after()));

    (base.unwrap_or(10), buf)
}

fn skip_underscores_between_digits(s: &str) -> impl Iterator<Item = char> + '_ {
    iter_with_prev_and_next(s.chars()).filter_map(|(prev, cur, next)| {
        let prev_is_digit = prev.is_some_and(|c| c.is_ascii_hexdigit());
        let next_is_digit = next.is_some_and(|c| c.is_ascii_hexdigit());
        if prev_is_digit && cur == '_' && next_is_digit {
            None
        } else {
            Some(cur)
        }
    })
}

fn iter_with_prev_and_next<I, T>(it: I) -> impl Iterator<Item = (Option<T>, T, Option<T>)>
where
    I: Iterator<Item = T> + Clone,
{
    let slow = iter::once(None).chain(it.clone().map(Some));
    let fast = it.clone().map(Some).skip(1).chain(iter::once(None));
    iter::zip(iter::zip(slow, it), fast).map(|((a, b), c)| (a, b, c))
}

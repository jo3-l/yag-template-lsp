use core::fmt;
use core::fmt::Write;
use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::error::Error;

use foldhash::{HashMap, HashMapExt};
use unscanny::Scanner;

pub mod bundled_envdefs;

#[derive(Debug)]
pub struct EnvDefs {
    pub funcs: HashMap<String, Func>,
}

#[derive(Debug, Clone)]
pub struct Func {
    pub name: String,
    pub params: Vec<Param>,
    pub doc: String,
}

impl Func {
    pub fn signature(&self) -> String {
        let mut buf = String::new();
        buf.push_str("func ");
        buf.push_str(&self.name);

        buf.push('(');
        let mut params = self.params.iter();
        if let Some(first_param) = params.next() {
            write!(buf, "{first_param}").unwrap();
            for param in params {
                buf.push_str(", ");
                write!(buf, "{param}").unwrap();
            }
        }
        buf.push(')');
        buf
    }
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub is_optional: bool,
    pub is_variadic: bool,
}

impl fmt::Display for Param {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)?;
        if self.is_optional {
            f.write_str("?")?;
        }
        if self.is_variadic {
            f.write_str("...")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub src_name: String,
    pub lineno: usize,
    pub message: String,
}

impl ParseError {
    pub(crate) fn new(src_name: String, lineno: usize, message: impl Into<String>) -> Self {
        Self {
            src_name,
            lineno,
            message: message.into(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.src_name, self.lineno, self.message)
    }
}

impl Error for ParseError {}

#[derive(Debug)]
pub struct EnvDefSource {
    name: Cow<'static, str>,
    data: Cow<'static, str>,
}

impl EnvDefSource {
    pub fn new(name: impl Into<String>, data: impl Into<String>) -> Self {
        Self {
            name: Cow::Owned(name.into()),
            data: Cow::Owned(data.into()),
        }
    }

    pub const fn new_static(name: &'static str, data: &'static str) -> Self {
        Self {
            name: Cow::Borrowed(name),
            data: Cow::Borrowed(data),
        }
    }
}

pub fn parse(sources: &[EnvDefSource]) -> Result<EnvDefs, ParseError> {
    let mut defs = EnvDefs { funcs: HashMap::new() };
    for src in sources {
        process_source(&mut defs, src)?;
    }
    Ok(defs)
}

fn process_source(defs: &mut EnvDefs, src: &EnvDefSource) -> Result<(), ParseError> {
    macro_rules! bail {
        ($msg:expr, $lineno:expr) => {
            return Err(ParseError::new(src.name.to_string(), $lineno, $msg))
        };
    }

    let mut funcs: Vec<(Func, usize)> = vec![];
    for (lineno, line) in src.data.lines().enumerate() {
        if line.starts_with("==") {
            // comment; ignore
        } else if line.chars().all(char::is_whitespace) {
            // blank line, possibly separating paragraphs in function documentation
            if let Some((f, _)) = funcs.last_mut() {
                if !f.doc.is_empty() {
                    // might result in some trailing blank lines, but it's OK:
                    // we'll trim the documentation at the end
                    f.doc.push('\n');
                }
            }
        } else if line.starts_with("func") {
            match parse_func_signature(line) {
                Ok(f) => funcs.push((f, lineno)),
                Err(msg) => bail!(msg, lineno),
            }
        } else if let Some(doc_line) = line.strip_prefix('\t') {
            match funcs.last_mut() {
                Some((f, _)) => {
                    f.doc.push_str(doc_line);
                    f.doc.push('\n');
                }
                None => bail!("unexpected indented line not part of function documentation", lineno),
            }
        } else {
            bail!(
                "could not interpret line as comment, function signature, or indented documentation",
                lineno
            )
        }
    }

    for (func, lineno) in funcs {
        match defs.funcs.entry(func.name.clone()) {
            Entry::Occupied(_) => bail!(format!("duplicate definition for function {}", func.name), lineno),
            Entry::Vacant(entry) => entry.insert(Func {
                doc: func.doc.trim().to_string(),
                ..func
            }),
        };
    }
    Ok(())
}

macro_rules! ensure {
    ($cond:expr, $err:expr) => {
        if !$cond {
            return Err($err.into());
        }
    };
}

fn parse_func_signature(line: &str) -> Result<Func, String> {
    let mut s = Scanner::new(line);

    // Parse the function name.
    s.expect("func");
    s.eat_whitespace();
    let name = s.eat_while(char::is_ascii_alphanumeric);

    // Parse the parameter list.
    let mut params = vec![];
    ensure!(s.eat_if('('), "expected '(' after function name");
    while !s.done() && !s.at(')') {
        // Ensure there's a comma if this isn't the first parameter.
        s.eat_whitespace();
        if !params.is_empty() {
            ensure!(s.eat_if(','), "expected ',' separating parameters");
            s.eat_whitespace();
        }

        // Parse the parameter name.
        ensure!(s.at(char::is_ascii_alphanumeric), "expected parameter name");
        let param_name = s.eat_while(char::is_ascii_alphanumeric);

        // Parse trailing modifiers.
        let is_optional = s.eat_if('?');
        let is_variadic = s.eat_if("...");
        if is_variadic && is_optional {
            return Err(format!(
                "parameter {param_name} cannot be marked as both variadic and optional; variadic implies optional"
            ));
        }
        params.push(Param {
            name: param_name.into(),
            is_optional,
            is_variadic,
        });
    }
    ensure!(s.eat_if(')'), "expected ')' concluding parameter list");

    s.eat_whitespace();
    ensure!(s.done(), "expected line to end after parameter list");
    Ok(Func {
        name: name.into(),
        params,
        doc: String::new(),
    })
}

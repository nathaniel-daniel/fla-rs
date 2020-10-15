use std::{
    convert::{
        TryFrom,
        TryInto,
    },
    str::FromStr,
};

#[derive(Debug, serde::Deserialize)]
pub struct Edge {
    #[serde(rename = "fillStyle1")]
    pub fill_style_1: Option<u64>,

    #[serde(rename = "strokeStyle")]
    pub stroke_style: Option<u64>,

    pub edges: Option<EdgeDefinition>,
}

impl Edge {
    pub fn get_edge_definition_commands(&self) -> Option<&[EdgeDefinitionCommand]> {
        Some(&self.edges.as_ref()?.commands)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum FromStrError {
    #[error("invalid char in numeric '{0}'")]
    InvalidCharInNumeric(char),

    #[error("Unexpected EOF")]
    UnexpectedEOF,

    #[error("Unexpected selection mask '{0}'")]
    UnexpectedSelectionMask(u8),

    #[error("invalid fixed point char")]
    InvalidFixedPointChar,

    #[error("Unknown Command")]
    UnknownCommand(char),

    #[error(transparent)]
    ParseFloat(#[from] std::num::ParseFloatError),

    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
}

#[derive(Debug, serde::Deserialize)]
#[serde(try_from = "String")]
pub struct EdgeDefinition {
    pub commands: Vec<EdgeDefinitionCommand>,
}

impl<'a> TryFrom<&'a str> for EdgeDefinition {
    type Error = FromStrError;

    fn try_from(input: &'a str) -> Result<Self, Self::Error> {
        let mut lexer = EdgeDefinitionLexer::new(input);
        let commands = lexer.lex_all()?;
        Ok(Self { commands })
    }
}

impl TryFrom<String> for EdgeDefinition {
    type Error = FromStrError;

    fn try_from(input: String) -> Result<Self, Self::Error> {
        input.as_str().try_into()
    }
}

/// !(x,y) moveTo
/// /(x,y)+ lineTo
/// |(x,y)+ lineTo
/// [(x1 y1 ex ey)+ curveTo (quadratic)
/// ](x1 y1 ex ey)+ curveTo (quadratic)
/// ((pBCPx pBCPy)? ; x1 y1 x2 y2 ex ey (({Q,q,P,p})? x y)+ curveTo (cubic start)
/// )(nBCPx nBCPy)? ; curveTo (cubic end)
/// Sn selection (n=bitmask, 1:fillStyle0, 2:fillStyle1, 4:stroke)
/// #aaaaaa.bb is a signed fixed point 32 bit number
pub struct EdgeDefinitionLexer<'a> {
    iter: std::str::CharIndices<'a>,
    peek: Option<(usize, char)>,
    input: &'a str,
}

impl<'a> EdgeDefinitionLexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut iter = input.char_indices();
        let peek = iter.next();

        Self { iter, peek, input }
    }

    pub fn next_char(&mut self) -> Option<(usize, char)> {
        let ret = self.peek;
        self.peek = self.iter.next();

        ret
    }

    pub fn peek_char(&mut self) -> Option<(usize, char)> {
        self.peek
    }

    pub fn lex_all(&mut self) -> Result<Vec<EdgeDefinitionCommand>, FromStrError> {
        let mut ret = Vec::new();

        while let Some(cmd) = self.lex_cmd()? {
            ret.push(cmd);
        }

        Ok(ret)
    }

    pub fn lex_cmd(&mut self) -> Result<Option<EdgeDefinitionCommand>, FromStrError> {
        let cmd = loop {
            match self.next_char() {
                Some((_, c)) => {
                    if !c.is_whitespace() {
                        break c;
                    }
                }
                None => return Ok(None),
            };
        };

        match cmd {
            '!' => {
                let x = self.read_numeric()?.ok_or(FromStrError::UnexpectedEOF)?;
                let y = self.read_numeric()?.ok_or(FromStrError::UnexpectedEOF)?;

                Ok(Some(EdgeDefinitionCommand::MoveTo(x, y)))
            }
            '|' | '/' => {
                let x = self.read_numeric()?.ok_or(FromStrError::UnexpectedEOF)?;
                let y = self.read_numeric()?.ok_or(FromStrError::UnexpectedEOF)?;

                Ok(Some(EdgeDefinitionCommand::LineTo(x, y)))
            }
            '[' | ']' => {
                let x1 = self.read_numeric()?.ok_or(FromStrError::UnexpectedEOF)?;
                let y1 = self.read_numeric()?.ok_or(FromStrError::UnexpectedEOF)?;
                let ex = self.read_numeric()?.ok_or(FromStrError::UnexpectedEOF)?;
                let ey = self.read_numeric()?.ok_or(FromStrError::UnexpectedEOF)?;

                Ok(Some(EdgeDefinitionCommand::CurveTo(x1, y1, ex, ey)))
            }
            'S' => {
                let c = self.next_char().ok_or(FromStrError::UnexpectedEOF)?.1;
                let mask = c
                    .to_digit(10)
                    .ok_or(FromStrError::InvalidCharInNumeric(c))? as u8;
                let selection_mask = SelectionMask::from_bits(mask)
                    .ok_or(FromStrError::UnexpectedSelectionMask(mask))?;
                Ok(Some(EdgeDefinitionCommand::Selection(selection_mask)))
            }
            c => Err(FromStrError::UnknownCommand(c)),
        }
    }

    pub fn peek_char_ignore_whitespace(&mut self) -> Option<(usize, char)> {
        loop {
            let peek = self.peek_char()?;
            if !peek.1.is_whitespace() {
                return Some(peek);
            }

            // Consume whitespace
            let _ = self.next_char().is_some();
        }
    }

    pub fn read_digits(&mut self, base: u32) -> Result<Option<&'a str>, FromStrError> {
        let (start_index, start_char) = match self.peek_char_ignore_whitespace() {
            Some(v) => v,
            None => return Ok(None),
        };

        if !start_char.is_digit(base) {
            return Err(FromStrError::InvalidCharInNumeric(start_char));
        }

        let _ = self.next_char().is_some();

        let mut current_index;
        loop {
            if let Some((index, c)) = self.peek_char() {
                current_index = index;

                if !c.is_digit(base) {
                    break;
                }
            } else {
                current_index = self.input.len();
                break;
            }

            let _ = self.next_char().is_some();
        }

        let digits = &self.input[start_index..current_index];

        Ok(Some(digits))
    }

    pub fn read_numeric(&mut self) -> Result<Option<f64>, FromStrError> {
        let (_, start_char) = match self.peek_char_ignore_whitespace() {
            Some(v) => v,
            None => return Ok(None),
        };

        if start_char == '#' {
            return Ok(self.read_fixed_point()?);
        }

        let mut modifier = 1.0;

        if start_char == '-' {
            modifier = -1.0;
            let _ = self.next_char().is_some();
        }

        let mut ret = match self.read_digits(10)? {
            Some(digits) => modifier * f64::from_str(digits)?,
            None => return Ok(None),
        };

        if let Some((_, '.')) = self.peek_char() {
            let _ = self.next_char().is_some();
            let digits = self.read_digits(10)?.ok_or(FromStrError::UnexpectedEOF)?;
            ret += f64::from_str(digits)? / digits.len() as f64;
        }

        Ok(Some(modifier * ret))
    }

    pub fn read_fixed_point(&mut self) -> Result<Option<f64>, FromStrError> {
        let (_start_index, start_char) = loop {
            match self.peek_char() {
                Some(v) => {
                    if !v.1.is_whitespace() {
                        break v;
                    }
                    // Consume whitespace
                    let _ = self.next_char().is_some();
                }
                None => return Ok(None),
            };
        };

        if start_char != '#' {
            return Err(FromStrError::InvalidFixedPointChar);
        }

        // Consume #
        let _ = self.next_char().is_some();

        let whole = self.read_digits(16)?.ok_or(FromStrError::UnexpectedEOF)?;

        if '.' != self.next_char().ok_or(FromStrError::UnexpectedEOF)?.1 {
            return Err(FromStrError::InvalidFixedPointChar);
        }

        let frac = self.read_digits(16)?.ok_or(FromStrError::UnexpectedEOF)?;

        let whole = f64::from(u32::from_str_radix(whole, 16)?);
        let frac = f64::from(u32::from_str_radix(frac, 16)?) / 256.0;
        let num = whole + frac;

        Ok(Some(num))
    }
}

#[derive(Debug)]
pub enum EdgeDefinitionCommand {
    MoveTo(f64, f64),
    LineTo(f64, f64),
    CurveTo(f64, f64, f64, f64),
    Selection(SelectionMask),
}

bitflags::bitflags! {
    pub struct SelectionMask: u8 {
        const FILLSTYLE0 = 1;
        const FILLSTYLE1 = 2;
        const STROKE = 4;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const EDGE_DEF_1:&str = "!280 250S2[280 263 272 272!272 272[263 280 250 280!250 280[238 280 229 272!229 272[220 263 220 250!220 250[220 238 229 229!229 229[238 220 250 220!250 220[263 220 272 229!272 229[280 238 280 250";
    const EDGE_DEF_2: &str = "!1904 192[1904 876 1418 1358!1418 1358[936 1844 252 1844!252 1844[-432 1844 -918 1358!-918 1358[-1400 876 -1400 192!-1400 192[-1400 -492 -918 -977!-918 -977[-432 -1460 252 -1460!252 -1460[936 -1460 1418 -977!1418 -977[1904 -492 1904 192";
    const EDGE_DEF_3: &str = "!264.5 42[#108.C5 #2A.F8 265 44!265 44[#10D.CE #3B.39 273 71.5";

    #[test]
    fn parse_edge_def_1() {
        EdgeDefinition::try_from(EDGE_DEF_1).unwrap();
    }

    #[test]
    fn parse_edge_def_2() {
        EdgeDefinition::try_from(EDGE_DEF_2).unwrap();
    }

    #[test]
    fn parse_edge_def_3() {
        EdgeDefinition::try_from(EDGE_DEF_3).unwrap();
    }
}

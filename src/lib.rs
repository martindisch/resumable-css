use cssparser::{ParseError, Parser, Token};

pub fn parse<'i>(input: &mut Parser<'i, '_>) -> Result<Vec<Token<'i>>, ParseError<'i, ()>> {
    let mut tokens = Vec::new();

    while let Ok(token) = input.next() {
        if let Ok(block_type) = BlockType::try_from(token) {
            tokens.push(token.clone());
            tokens.extend(input.parse_nested_block(|input| parse(input))?);
            tokens.push(block_type.closing_token());
        } else {
            tokens.push(token.clone());
        }
    }

    Ok(tokens)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockType {
    Parenthesis,
    SquareBracket,
    CurlyBracket,
}

impl TryFrom<&Token<'_>> for BlockType {
    type Error = ();

    fn try_from(value: &Token) -> Result<Self, Self::Error> {
        match value {
            Token::Function(_) | Token::ParenthesisBlock => Ok(BlockType::Parenthesis),
            Token::SquareBracketBlock => Ok(BlockType::SquareBracket),
            Token::CurlyBracketBlock => Ok(BlockType::CurlyBracket),
            _ => Err(()),
        }
    }
}

impl BlockType {
    fn closing_token(&self) -> Token<'static> {
        match self {
            BlockType::Parenthesis => Token::CloseParenthesis,
            BlockType::SquareBracket => Token::CloseSquareBracket,
            BlockType::CurlyBracket => Token::CloseCurlyBracket,
        }
    }
}

#[cfg(test)]
mod tests {
    use cssparser::{ParserInput, ToCss};

    use super::*;

    #[test]
    fn complete() {
        let css = r#"
.foo {
  .fancy {
    color: blue;
  }

  &:hover {
    color: orange;
  }
}

body {
  color: green;
}
"#;

        let mut input = ParserInput::new(css);
        let mut parser = Parser::new(&mut input);

        let tokens = parse(&mut parser).unwrap();
        let css = tokens.iter().map(|t| t.to_css_string()).collect::<String>();

        assert_eq!(
            ".foo{.fancy{color:blue;}&:hover{color:orange;}}body{color:green;}",
            css
        );
    }

    #[test]
    fn partial() {
        let partial = r#"
.foo {
  .fancy {
    /* hello .world { color: red; } */
    color: blue;
"#;

        let mut input = ParserInput::new(partial);
        let mut parser = Parser::new(&mut input);

        let tokens = parse(&mut parser).unwrap();
        let css = tokens.iter().map(|t| t.to_css_string()).collect::<String>();

        // This may be a bit surprising: the parser adds closing tokens to make
        // partial CSS valid
        assert_eq!(".foo{.fancy{color:blue;}}", css);
    }
}

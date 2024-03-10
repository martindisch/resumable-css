use cssparser::{ParseError, Parser, Token};

pub fn parse_flat<'i>(input: &mut Parser<'i, '_>) -> Result<Vec<Token<'i>>, ParseError<'i, ()>> {
    let mut tokens = Vec::new();

    while let Ok(token) = input.next() {
        tokens.push(token.clone());
    }

    Ok(tokens)
}

pub fn parse_nested<'i>(input: &mut Parser<'i, '_>) -> Result<Vec<Token<'i>>, ParseError<'i, ()>> {
    let mut tokens = Vec::new();

    while let Ok(token) = input.next() {
        if let Ok(block_type) = BlockType::try_from(token) {
            tokens.push(token.clone());
            tokens.extend(input.parse_nested_block(|input| parse_nested(input))?);
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
    fn flat_complete() {
        let css = r#"
.foo {
  .fancy {
    color: blue;
  }
}

body {
  color: green;
}
"#;

        let mut input = ParserInput::new(css);
        let mut parser = Parser::new(&mut input);

        let tokens = parse_flat(&mut parser).unwrap();
        let css = tokens.iter().map(|t| t.to_css_string()).collect::<String>();

        // This demonstrates one particularity of this parser, namely that it
        // flattens blocks and doesn't automatically descend
        assert_eq!(
            [
                Token::Delim('.'),
                Token::Ident("foo".into()),
                Token::CurlyBracketBlock,
                Token::Ident("body".into()),
                Token::CurlyBracketBlock
            ],
            &tokens[..]
        );
        // The blocks are also represented only by their opening token, which
        // results in this not really useful representation
        assert_eq!(".foo{body{", css);
    }

    #[test]
    fn flat_partial() {
        let partial_1 = r#"
.foo {
  .fancy {
    color: blue;
"#;
        let partial_2 = r#"
  }
}

body {
  color: green;
}
"#;

        let mut input = ParserInput::new(partial_1);
        let mut parser = Parser::new(&mut input);
        let tokens = parse_flat(&mut parser).unwrap();
        // Because it doesn't descend, it doesn't mind unclosed blocks
        assert_eq!(
            [
                Token::Delim('.'),
                Token::Ident("foo".into()),
                Token::CurlyBracketBlock
            ],
            &tokens[..]
        );

        let mut input = ParserInput::new(partial_2);
        let mut parser = Parser::new(&mut input);
        let tokens = parse_flat(&mut parser).unwrap();
        // Here the closing tokens appear, but that's considered an error based
        // on the intended behavior (closing tokens are usually swallowed)
        assert_eq!(
            [
                Token::CloseCurlyBracket,
                Token::CloseCurlyBracket,
                Token::Ident("body".into()),
                Token::CurlyBracketBlock
            ],
            &tokens[..]
        );
    }

    #[test]
    #[should_panic]
    fn flat_resumed() {
        let partial_1 = r#"
.foo {
  .fancy {
    color: blue;
"#;
        let partial_2 = r#"
  }
}

body {
  color: green;
}
"#;

        let mut input = ParserInput::new(partial_1);
        let mut parser = Parser::new(&mut input);
        parse_flat(&mut parser).unwrap();
        let state = parser.state();

        let mut input = ParserInput::new(partial_2);
        // We can't change the input of a parser, so we create a new instance
        // even though that's wrong
        let mut parser = Parser::new(&mut input);
        parser.reset(&state);

        // This panics at a char boundary because we restored state for a
        // different parser/input
        parse_flat(&mut parser).unwrap();
    }

    #[test]
    fn nested_complete() {
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

        let tokens = parse_nested(&mut parser).unwrap();
        let css = tokens.iter().map(|t| t.to_css_string()).collect::<String>();

        assert_eq!(
            ".foo{.fancy{color:blue;}&:hover{color:orange;}}body{color:green;}",
            css
        );
    }

    #[test]
    fn nested_partial() {
        let partial = r#"
.foo {
  .fancy {
    /* hello .world { color: red; } */
    color: blue;
"#;

        let mut input = ParserInput::new(partial);
        let mut parser = Parser::new(&mut input);

        let tokens = parse_nested(&mut parser).unwrap();
        let css = tokens.iter().map(|t| t.to_css_string()).collect::<String>();

        // This may be a bit surprising: because we have to manually push
        // closing tags due to how `Parser::next` swallows them, we now add
        // closing tokens that make invalid CSS valid
        assert_eq!(".foo{.fancy{color:blue;}}", css);
    }
}

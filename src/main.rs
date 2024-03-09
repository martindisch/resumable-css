use cssparser::{Parser, ParserInput, ToCss};

fn main() {
    println!("This is a tests-only project, run `cargo test`.");

    let css = r#"
.foo {
  .fancy {
    color: blue;
  }

  &:hover {
    color: orange;
  }
}

.bar {
  color: green;
}
"#;

    let mut input = ParserInput::new(css);
    let mut parser = Parser::new(&mut input);
    let mut tokens = Vec::new();

    while let Ok(token) = parser.next() {
        tokens.push(token.clone());
    }

    println!("{tokens:#?}");

    let css = tokens.iter().map(|t| t.to_css_string()).collect::<String>();
    println!("{css}");
}

#[cfg(test)]
mod tests {
    #[test]
    fn complete() {
        assert!(true);
    }
}

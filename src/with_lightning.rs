#[cfg(test)]
mod tests {
    use lightningcss::{
        printer::PrinterOptions,
        stylesheet::{ParserOptions, StyleSheet},
    };

    #[test]
    fn nested_complete() {
        let stylesheet = StyleSheet::parse(
            r#"
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
"#,
            ParserOptions::default(),
        )
        .unwrap();

        assert_eq!(
            ".foo{& .fancy{color:#00f}&:hover{color:orange}}body{color:green}",
            stylesheet
                .to_css(PrinterOptions {
                    minify: true,
                    ..Default::default()
                })
                .unwrap()
                .code
        );
    }

    #[test]
    fn nested_partial_start() {
        let stylesheet = StyleSheet::parse(
            r#"
.foo {
  .fancy {
    /* hello .world { color: red; } */
    color: blue;
"#,
            ParserOptions::default(),
        )
        .unwrap();

        // It looks like this parser also adds missing closing tokens
        assert_eq!(
            ".foo{& .fancy{color:#00f}}",
            stylesheet
                .to_css(PrinterOptions {
                    minify: true,
                    ..Default::default()
                })
                .unwrap()
                .code
        );
    }

    #[test]
    fn nested_partial_end() {
        let stylesheet = StyleSheet::parse(
            r#"
  }
}

body {
  color: green;
}
"#,
            ParserOptions {
                error_recovery: true,
                ..Default::default()
            },
        )
        .unwrap();

        // While it doesn't error our with error_recovery: true, it doesn't
        // parse anything either
        assert_eq!(
            "",
            stylesheet
                .to_css(PrinterOptions {
                    minify: true,
                    ..Default::default()
                })
                .unwrap()
                .code
        );
    }
}

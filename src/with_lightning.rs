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
}

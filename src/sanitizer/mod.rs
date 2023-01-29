pub mod sanitizer {
    pub enum StringReplacement<'a> {
        EncodedValue { to: &'a str, from: &'a str },
    }

    pub fn sanitize(s: &str) -> String {
        let mut sanitized_string = s.to_owned();

        // remove encoded values
        let encoded_values = vec![StringReplacement::EncodedValue {
            to: "&",
            from: "&amp;",
        }];

        for encoded_value in encoded_values {
            sanitized_string = match encoded_value {
                StringReplacement::EncodedValue { to, from } => sanitized_string.replace(from, to),
            }
        }

        sanitized_string
    }
}

#[cfg(test)]
pub mod tests {
    use super::sanitizer::sanitize;

    #[test]
    fn removes_html_encoded_values() {
        let result = sanitize(
            "He won the pentathlon &amp; the decathlon at the summer Olympics in Stockholm",
        );

        let expected = "He won the pentathlon & the decathlon at the summer Olympics in Stockholm";

        assert_eq!(result, expected);
    }
}

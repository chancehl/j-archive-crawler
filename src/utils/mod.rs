pub mod sanitizer {
    use regex::Regex;

    pub enum StringReplacement<'a> {
        EncodedValue { to: &'a str, from: &'a str },
    }

    pub enum PatternReplacement<'a> {
        RegexValue { pattern: Regex, to: &'a str },
    }

    /// Removes invalid characters & sequences from a string
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

        // remove invalid patterns
        let invalid_patterns = vec![
            PatternReplacement::RegexValue {
                pattern: Regex::new(r"</.+>").unwrap(), // closing tags
                to: " ",
            },
            PatternReplacement::RegexValue {
                pattern: Regex::new(r"<.+>").unwrap(),
                to: " ",
            },
        ];

        for invalid_pattern in invalid_patterns {
            sanitized_string = match invalid_pattern {
                PatternReplacement::RegexValue { to, pattern } => {
                    pattern.replace(&sanitized_string, to).to_string()
                }
            }
        }

        // Trim start, end of string
        sanitized_string = sanitized_string.trim().to_string();

        sanitized_string
    }
}

#[cfg(test)]
pub mod tests {

    pub mod sanitize_tests {
        use super::super::sanitizer::sanitize;

        #[test]
        fn removes_html_encoded_values() {
            let result = sanitize(
                "He won the pentathlon &amp; the decathlon at the summer Olympics in Stockholm",
            );

            let expected =
                "He won the pentathlon & the decathlon at the summer Olympics in Stockholm";

            assert_eq!(result, expected);
        }

        #[test]
        fn removes_html_elements() {
            let result = sanitize("100 YEARS AGO<span class=\"nobreak\">--</span>1912");
            let expected = "100 YEARS AGO -- 1912";

            assert_eq!(result, expected);
        }

        #[test]
        fn trims_str() {
            let result = sanitize(" sotto vocce ");
            let expected = "sotto vocce";

            assert_eq!(result, expected);
        }
    }
}

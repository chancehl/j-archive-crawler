pub mod sanitizer {
    use regex::Regex;

    pub enum Replacement<'a> {
        String { to: &'a str, from: &'a str },
        Regex { pattern: Regex, to: &'a str },
    }

    /// Removes invalid characters & sequences from a string
    pub fn sanitize(s: &str) -> String {
        let mut sanitized_string = s.to_owned();

        // remove encoded values
        let replacements = vec![
            Replacement::String {
                to: "&",
                from: "&amp;", // encoded ampersand
            },
            Replacement::Regex {
                pattern: Regex::new(r"</.+>").unwrap(), // closing html tags
                to: " ",
            },
            Replacement::Regex {
                pattern: Regex::new(r"<.+>").unwrap(), // opening html tags
                to: " ",
            },
        ];

        for replacement in replacements {
            sanitized_string = match replacement {
                Replacement::String { to, from } => sanitized_string.replace(from, to),
                Replacement::Regex { pattern, to } => {
                    pattern.replace(&sanitized_string, to).to_string()
                }
            }
        }

        // trim start, end of string
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

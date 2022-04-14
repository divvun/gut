use anyhow::Result;
use regex::{Error as RegexError, Regex, RegexBuilder};
use std::collections::BTreeMap;

pub fn generate_file_paths(
    replacements: &BTreeMap<String, String>,
    files: Vec<&str>,
) -> Result<Vec<(String, String)>> {
    let mut results = vec![];
    for file in files {
        let result = generate_string(replacements, file)?;
        results.push((file.to_string(), result));
    }
    Ok(results)
}

pub fn generate_string(replacements: &BTreeMap<String, String>, content: &str) -> Result<String> {
    let mut result = content.to_string();
    for (pattern, replace) in replacements {
        let re = to_regex(pattern)?;
        result = re.replace_all(result.as_str(), &replace[..]).into_owned();
    }
    Ok(result)
}

fn to_regex(s: &str) -> Result<Regex, RegexError> {
    RegexBuilder::new(s).case_insensitive(true).build()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    #[test]
    fn test_generate_file_paths_single() {
        let files = vec!["src/a.txt", "src/__UND__/__UND__.txt", "lang-__UND__.txt"];

        let mut rep = BTreeMap::new();
        rep.insert("__UND__".to_string(), "en".to_string());

        let results = super::generate_file_paths(&rep, files).unwrap();

        let expected = vec![
            ("src/a.txt".to_string(), "src/a.txt".to_string()),
            (
                "src/__UND__/__UND__.txt".to_string(),
                "src/en/en.txt".to_string(),
            ),
            ("lang-__UND__.txt".to_string(), "lang-en.txt".to_string()),
        ];

        assert_eq!(results, expected);
    }

    #[test]
    fn test_generate_file_paths_2_patterns() {
        let files = vec![
            "src/a.txt",
            "src/__UND__/__UND__.txt",
            "src/__ABC__/__UND__.txt",
            "lang-__UND__.txt",
            "lang-__UND____ABC__.txt",
        ];

        let mut rep = BTreeMap::new();
        rep.insert("__UND__".to_string(), "en".to_string());
        rep.insert("__ABC__".to_string(), "abc".to_string());

        let results = super::generate_file_paths(&rep, files).unwrap();

        let expected = vec![
            ("src/a.txt".to_string(), "src/a.txt".to_string()),
            (
                "src/__UND__/__UND__.txt".to_string(),
                "src/en/en.txt".to_string(),
            ),
            (
                "src/__ABC__/__UND__.txt".to_string(),
                "src/abc/en.txt".to_string(),
            ),
            ("lang-__UND__.txt".to_string(), "lang-en.txt".to_string()),
            (
                "lang-__UND____ABC__.txt".to_string(),
                "lang-enabc.txt".to_string(),
            ),
        ];

        assert_eq!(results, expected);
    }
}

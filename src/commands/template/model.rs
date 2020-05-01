use std::collections::HashMap;

#[derive(Debug)]
pub struct TemplateDelta {
    pub name: String,
    pub patterns: Vec<String>,
    pub rev_id: usize,
    pub required: Vec<String>,
    pub optional: Vec<String>,
    pub ignored: Vec<String>,
}

#[derive(Debug)]
pub struct TargetDelta {
    pub template: String,
    pub replacements: HashMap<String, String>,
    pub rev_id: usize,
    pub template_sha: String,
}

pub fn temp_sample() -> TemplateDelta {
    TemplateDelta {
        name: "Language Template".to_string(),
        patterns: vec!["__UND__".to_string()],
        rev_id: 2,
        required: vec![
            "src/a.txt".to_string(),
            "src/__UND__/__UND__.txt".to_string(),
            "lang-__UND__.txt".to_string(),
        ],
        optional: vec!["b.txt".to_string()],
        ignored: vec!["c.txt".to_string()],
    }
}

pub fn target_delta_sample() -> TargetDelta {
    let mut rep = HashMap::new();
    rep.insert("__UND__".to_string(), "en".to_string());
    TargetDelta {
        template: "???".to_string(),
        replacements: rep,
        rev_id: 1,
        template_sha: "ab4139e82667a373b7ca56f70bfa27c6fb116c85".to_string(),
    }
}

use std::str;
use git2::{Error, Index, Repository, Diff, Oid, Tree, DiffOptions, DiffFile, DiffHunk, DiffLine, DiffDelta, DiffFormat};
use std::collections::HashMap;
use git2;
use super::common::*;
use anyhow::{anyhow, Result};
//use std::fs::write;

pub fn diff_to_patch(diff: &Diff) -> Result<Vec<PatchFile>> {

    let mut file_map: HashMap<String, PatchFile> = HashMap::new();

    //diff.foreach(
            //&mut |_file, _progress| { true },
            //None,
            //Some(&mut |delta, hunk| {
                //log::info!("hunk_cb path: {:?}, hunk: {:?}", delta.new_file().path(), str::from_utf8(hunk.header()).unwrap());
                //true
            //}),
            //Some(&mut |delta, _hunk, line| {
                //println!("line_cb path {:?}", delta.new_file().path());
                //if let Some(new_file) = delta.new_file().path().and_then(|p| p.to_str()) {
                    //let old_file = delta.old_file().path().and_then(|p| p.to_str()).unwrap();

                    //file_map.entry(new_file.to_string()).or_insert_with(|| PatchFile::new(old_file, new_file));

                        //if let Some(file) = file_map.get(new_file) {
                            //if let Some(patch_line) = diff_line_to_patch_line(&line) {
                                //let new_path_file = file.add_line(&patch_line);
                                //file_map.insert(old_file.to_string(), new_path_file);
                            //}
                        //}
                //}
                //true
            //})
        //)?;

        diff.print(DiffFormat::Patch, |delta, _hunk, line| {
                println!("line_cb path {:?}", delta.new_file().path());
                if let Some(new_file) = delta.new_file().path().and_then(|p| p.to_str()) {
                    let old_file = delta.old_file().path().and_then(|p| p.to_str()).unwrap();

                    file_map.entry(new_file.to_string()).or_insert_with(|| PatchFile::new(old_file, new_file));

                        if let Some(file) = file_map.get(new_file) {
                            if let Some(patch_line) = diff_line_to_patch_line(&line) {
                                let new_path_file = file.add_line(&patch_line);
                                file_map.insert(old_file.to_string(), new_path_file);
                            }
                        }
                }
                true
        })?;


    let mut v = vec![];
    for p in file_map.values() {
        v.push(p.clone())
    }
    Ok(v)
}

fn diff_line_to_patch_line(diff_line: &DiffLine) -> Option<PatchLine> {
    match diff_line.origin() {
        ' ' => Some(PatchLine::Move{old_line_no: diff_line.old_lineno().unwrap(), new_line_no: diff_line.new_lineno().unwrap(), content: str::from_utf8(diff_line.content()).unwrap().to_string()}),
        '+' => Some(PatchLine::Add{line_no: diff_line.new_lineno().unwrap(), content:str::from_utf8(diff_line.content()).unwrap().to_string()}),
        '-' => Some(PatchLine::Delete{line_no: diff_line.old_lineno().unwrap(), content:str::from_utf8(diff_line.content()).unwrap().to_string()}),
        'F' => Some(PatchLine::Info{content:str::from_utf8(diff_line.content()).unwrap().to_string()}),
        'H' => Some(PatchLine::Hunk{content:str::from_utf8(diff_line.content()).unwrap().to_string()}),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatchLine {
    Add {
        line_no : u32,
        content: String,
    },
    Move {
        old_line_no: u32,
        new_line_no: u32,
        content: String,
    },
    Delete {
        line_no: u32,
        content: String,
    },
    Hunk {
        content: String,
    },
    Info {
        content: String,
    }
}

impl PatchLine {
    pub fn to_content(&self) -> String {
        match self {
            PatchLine::Add {line_no, content} => format!("+{}", content),
            PatchLine::Move {old_line_no, new_line_no, content} => format!(" {}", content),
            PatchLine::Delete {line_no, content} => format!("-{}", content),
            PatchLine::Hunk {content} => format!("{}", content),
            PatchLine::Info {content} => format!("{}", content),
        }
    }

    pub fn apply_patterns(&self, reps: &HashMap<String, String>) -> Result<PatchLine> {
        let pl = match self {
            PatchLine::Add {line_no, content} => PatchLine::Add{line_no: *line_no, content: generate_string(reps, content.as_str())?},
            PatchLine::Move {old_line_no, new_line_no, content} => PatchLine::Move{old_line_no: *old_line_no, new_line_no: *new_line_no, content: generate_string(reps, content.as_str())?},
            PatchLine::Delete {line_no, content} => PatchLine::Delete{line_no: *line_no, content: generate_string(reps, content.as_str())?},
            PatchLine::Hunk {content} => PatchLine::Hunk{content: generate_string(reps, content.as_str())?},
            PatchLine::Info {content} => PatchLine::Info{content: generate_string(reps, content.as_str())?},
        };
        Ok(pl)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatchFile {
    pub old_file: String,
    pub new_file: String,
    pub lines: Vec<PatchLine>,
}

impl PatchFile {

    pub fn add_line(&self, line: &PatchLine) -> PatchFile {
        let mut lines = self.lines.clone();
        lines.push(line.clone());

        PatchFile {
            old_file: self.old_file.clone(),
            new_file: self.new_file.clone(),
            lines,
        }
    }

    pub fn new(old_file: &str, new_file: &str) -> PatchFile {
        PatchFile {
            old_file: old_file.to_string(),
            new_file: new_file.to_string(),
            lines: vec![],
        }
    }

    pub fn apply_patterns(&self, reps: &HashMap<String, String>) -> Result<PatchFile> {
        let old_file = generate_string(reps, self.old_file.as_str())?;
        let new_file = generate_string( reps, self.new_file.as_str())?;
        let lines: Vec<Result<PatchLine>> = self.lines.iter().map(|l| l.apply_patterns(reps)).collect();
        let lines: Result<Vec<_>> = lines.into_iter().collect();
        let lines = lines?;
        Ok(PatchFile {
            old_file, new_file, lines
        })
    }

    pub fn to_content(&self) -> String {
        let contents: Vec<String> = self.lines.iter().map(|f| f.to_content()).collect();
        contents.join("")
    }

}

pub fn to_content(files: &Vec<PatchFile>) -> String {
    let contents: Vec<String> = files.iter().map(|f| f.to_content()).collect();
    contents.join("")
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::{PatchLine, PatchFile};

    fn lines_sample_1() -> Vec<PatchLine> {
        vec![
            PatchLine::Info{content: "diff --git a/README.md b/README.md
index 9939b16..68b2be5 100644
--- a/README.md
+++ b/README.md".to_string()},
            PatchLine::Hunk{content: "@@ -1,3 +1,7 @@".to_string()},
            PatchLine::Delete{line_no: 1, content: "# __UND__".to_string()},
            PatchLine::Add{line_no: 1, content: "# Hello __UND__".to_string()},
            PatchLine::Add{line_no: 2, content: "".to_string()},
            PatchLine::Add{line_no: 3, content: "rev 2".to_string()},
            PatchLine::Move{old_line_no: 2, new_line_no: 4, content: "".to_string()},
            PatchLine::Move{old_line_no: 3, new_line_no: 5, content: "This is a repo for __UND__".to_string()},
            PatchLine::Add{line_no: 6, content: "".to_string()},
            PatchLine::Add{line_no: 7, content: "And __UND__ is great".to_string()},
        ]
    }

    #[test]
    fn test_patch_file_to_content() {

        let lines = lines_sample_1();

        let file = PatchFile {
            old_file : "".to_string(),
            new_file : "".to_string(),
            lines,
        };
        let content = file.to_content();

        let expected = "diff --git a/README.md b/README.md
index 9939b16..68b2be5 100644
--- a/README.md
+++ b/README.md
@@ -1,3 +1,7 @@
-# __UND__
+# Hello __UND__
+
+rev 2
 
 This is a repo for __UND__
+
+And __UND__ is great".to_string();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_patch_line_apply_patterns() {
        let lines = lines_sample_1();

        let mut reps = HashMap::new();
        reps.insert("__UND__".to_string(), "en".to_string());

        let results: Vec<PatchLine> = lines.iter().map(|l| l.apply_patterns(&reps).unwrap()).collect();

        let expected = vec![
            PatchLine::Info{content: "diff --git a/README.md b/README.md
index 9939b16..68b2be5 100644
--- a/README.md
+++ b/README.md".to_string()},
            PatchLine::Hunk{content: "@@ -1,3 +1,7 @@".to_string()},
            PatchLine::Delete{line_no: 1, content: "# en".to_string()},
            PatchLine::Add{line_no: 1, content: "# Hello en".to_string()},
            PatchLine::Add{line_no: 2, content: "".to_string()},
            PatchLine::Add{line_no: 3, content: "rev 2".to_string()},
            PatchLine::Move{old_line_no: 2, new_line_no: 4, content: "".to_string()},
            PatchLine::Move{old_line_no: 3, new_line_no: 5, content: "This is a repo for en".to_string()},
            PatchLine::Add{line_no: 6, content: "".to_string()},
            PatchLine::Add{line_no: 7, content: "And en is great".to_string()},
        ];

        assert_eq!(results, expected);
    }

    #[test]
    fn test_patch_file_apply_patterns() {

        let lines = lines_sample_1();

        let file = PatchFile {
            old_file: "src/__UND__/__UND__.txt".to_string(),
            new_file: "src/__UND__/__UND__.txt".to_string(),
            lines,
        };

        let mut reps = HashMap::new();
        reps.insert("__UND__".to_string(), "en".to_string());

        let result = file.apply_patterns(&reps).unwrap();

        let lines_expected = vec![
            PatchLine::Info{content: "diff --git a/README.md b/README.md
index 9939b16..68b2be5 100644
--- a/README.md
+++ b/README.md".to_string()},
            PatchLine::Hunk{content: "@@ -1,3 +1,7 @@".to_string()},
            PatchLine::Delete{line_no: 1, content: "# en".to_string()},
            PatchLine::Add{line_no: 1, content: "# Hello en".to_string()},
            PatchLine::Add{line_no: 2, content: "".to_string()},
            PatchLine::Add{line_no: 3, content: "rev 2".to_string()},
            PatchLine::Move{old_line_no: 2, new_line_no: 4, content: "".to_string()},
            PatchLine::Move{old_line_no: 3, new_line_no: 5, content: "This is a repo for en".to_string()},
            PatchLine::Add{line_no: 6, content: "".to_string()},
            PatchLine::Add{line_no: 7, content: "And en is great".to_string()},
        ];

        let expected_file = PatchFile {
            old_file: "src/en/en.txt".to_string(),
            new_file: "src/en/en.txt".to_string(),
            lines: lines_expected,
        };

        assert_eq!(result, expected_file);
    }
}

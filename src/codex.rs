
use core::fmt;

use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::collections::HashMap;


pub struct Codex {
    history: Vec<CodexAction>,
    words: HashMap<String, WordEntry>
}

#[derive(Clone, Debug)]
pub enum CodexAction {
    Introduce (String, Confidence, String), // Word, conf, Definition
    Practice (String, Correctness) 
}

#[derive(Clone, Copy, Debug)]
pub enum Confidence {
    Known,
    PartiallyKnown,
    Unknown
}

#[derive(Clone, Copy, Debug)]
pub enum Correctness {
    Correct,
    PartiallyCorrect,
    Incorrect,
}

pub struct WordEntry {
    word: String,
    knowledge_score: f32,  // Between 0 and 1, weighted more heavily based on recent practices.
    definition: String,
}

pub struct VocabParseError; // TODO: Specialize this a bit.


/* Coversion to and from strings */

impl FromStr for Confidence {
    type Err = VocabParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "known" => Ok(Confidence::Known),
            "partially_known" => Ok(Confidence::PartiallyKnown),
            "unknown" => Ok(Confidence::Unknown),
            _ => Err(VocabParseError),
        }
    }
}

impl fmt::Display for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Confidence::Known => "known",
            Confidence::PartiallyKnown => "partially_known",
            Confidence::Unknown => "unknown",
        })
    }
}

impl FromStr for Correctness {
    type Err = VocabParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "correct" => Ok(Correctness::Correct),
            "partially_correct" => Ok(Correctness::PartiallyCorrect),
            "incorrect" => Ok(Correctness::Incorrect),
            _ => Err(VocabParseError),
        }
    }
}

impl fmt::Display for Correctness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Correctness::Correct => "correct",
            Correctness::PartiallyCorrect => "partially_correct",
            Correctness::Incorrect => "incorrect",
        })
    }
}

impl fmt::Display for CodexAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            CodexAction::Introduce(word, confidence, def) => "introduce$".to_owned() + word + "$" + &confidence.to_string() + "$" + def,
            CodexAction::Practice(word, correctness) => "practice$".to_owned() + word + "$" + &correctness.to_string(),
        })
    }
}


/* Conversions to float */

impl Confidence {
    fn to_float(&self) -> f32 {
        match self {
            Confidence::Known => 0.9,
            Confidence::PartiallyKnown => 0.5,
            Confidence::Unknown => 0.1,
        }
    }
}

impl Correctness {
    fn to_float(&self) -> f32 {
        match self {
            Correctness::Correct => 1.0,
            Correctness::PartiallyCorrect => 0.5,
            Correctness::Incorrect => 0.0,
        }
    }
}

/* Codex operations */

impl Codex {
    pub fn from_file(path: &PathBuf) -> Option<Codex> {
        let mut file = std::fs::File::open(path).expect("File should open");

        let mut buf = String::new();
        file.read_to_string(&mut buf).ok()?;

        let mut codex = Codex {history: vec![], words: HashMap::new()}; 

        buf.lines()
            .filter(|s| s.trim() != "")
            .map(|s| Ok(match s.trim().split('$').collect::<Vec<&str>>()[..] {
                ["introduce", x, confidence, def] => codex.process_action(CodexAction::Introduce(x.to_owned(), confidence.parse()?, def.to_owned())),
                ["practice", x, correctness] => codex.process_action(CodexAction::Practice(x.to_owned(), correctness.parse()?)),
                _ => Err(VocabParseError {})?
            }))
            .collect::<Result<(), VocabParseError>>()
            .ok()?;

        Some(codex)
    }

    pub fn to_file(&self, path: &PathBuf) -> bool {
        let mut file = match std::fs::File::create(path) {
            Ok(file) => file,
            Err(_) => {
                println!("Something went wrong opening file... Saving to backup.codex instead");
                std::fs::File::create("backup.codex").expect("Backup will open.")
            }
        };

        file.write(self.history.iter()
            .map(|e| e.to_string())
            .collect::<Vec<String>>()
            .join("\n")
            .as_bytes()
        ).is_ok()
    }

    pub fn process_action(&mut self, action: CodexAction) { // TODO: Remove expect, change return type
        self.history.push(action.clone());

        match action {
            CodexAction::Introduce(word, conf, def) => {
                self.words.insert(word.clone(), WordEntry {word, knowledge_score: conf.to_float(), definition: def});
            }
            CodexAction::Practice(word, corr) => {
                let entry = self.words.get_mut(&word).expect("word exists");
                entry.knowledge_score =  0.25 * corr.to_float() + 0.75 * entry.knowledge_score;
            }
        };
    }

    pub fn contains(&self, word: &str) -> bool {
        self.words.contains_key(word)
    }
}


mod prompt;
mod codex;


use codex::{Codex, Confidence, CodexAction};
use prompt::Prompt;

use std::path::PathBuf;


#[derive(Clone, Copy)]
enum MainAction {
    Exit,
    Introduce,
    Practice
}


fn get_file_path() -> PathBuf {
    return loop {
        let file_prompt = prompt::ValidatedFieldPrompt::new(
            "Enter a file path...", 
            |s| {
                let mut path = PathBuf::from(s);
                
                path.pop();

                if !path.is_dir() && path.to_str() != Some("")  {
                    println!("Invalid Directory: {}", path.display());
                    return false;
                }

                true
            }
        );

        let path = PathBuf::from(match file_prompt.run() {
            Some(s) => s,
            None => continue,
        });

        if path.is_file() {
            break path
        }
        else {
            let create_file_anyways = prompt::ChoicePrompt::<bool>::new("That file does not exist. Should we create it? (y/n)")
                .add_choice(vec!["yes", "y", "Y"], true)
                .add_choice(vec!["no", "n", "N"], false)
                .run_with_reprompt("Please enter \"y\" or \"n\"... ");

            if create_file_anyways {
                std::fs::File::create(&path).expect("File could be created.");
                break path
            }
        }
    };
}

fn introduce(codex: &mut Codex) {
    println!("Introducing new vocab. Type \"!done\" to finish.");

    loop {
        let prompt = prompt::ValidatedFieldPrompt::new("Type a word to add (or !done): ", |s| {
            if s.contains('$') {
                println!("Word cannot contain $");
                false
            }
            else if s.contains(' ') {
                println!("Word cannot contain space");
                false
            }
            else if s.trim().len() == 0 {
                println!("Word not found");
                false
            }
            else if s.trim().chars().nth(0) == Some('!') && s.trim() != "!done" {
                println!("\"!\" command not recognized.");
                false
            }
            else {
                true
            }
        });
        
        let word = prompt.run_with_reprompt("").trim().to_owned();

        if word == "!done" {
            println!("Finished introducing words.");
            break;
        }

        if codex.contains(&word) {
            println!("Note: \"{}\" already in codex. This will overwrite it. Type !done to abort.", word);
        }

        let prompt = prompt::ValidatedFieldPrompt::new("Type the definition: ", |s| {
            if s.contains('$') {
                println!("Defintion cannot contain $");
                false
            }
            else if s.trim().len() == 0 {
                println!("Definition not found");
                false
            }
            else if word.trim().chars().nth(0) == Some('!') && word.trim() != "!done" {
                println!("\"!\" command not recognized.");
                false
            }
            else {
                true
            }
        });

        let def = prompt.run_with_reprompt("");
        if def == "!done" {
            println!("Finished introducing words.");
            break;
        }

        let mut prompt = prompt::ChoicePrompt::<Confidence>::new("How confidently do you know this word?");
        prompt.add_choice(vec!["Known", "known", "k"], Confidence::Known);
        prompt.add_choice(vec!["Partial", "partial", "p"], Confidence::PartiallyKnown);
        prompt.add_choice(vec!["Unknown", "unknown", "u"], Confidence::Unknown);

        let conf = prompt.run_with_reprompt("Unrecognized option.");

        codex.process_action(CodexAction::Introduce(word.clone(), conf, def));

        println!("Added \"{}\" to codex", word);
    }
}


fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    println!("Welcome to Rust Vocab!");

    println!("First, we'll need to open (or create) a codex file.");

    let path = get_file_path();

    let mut codex = codex::Codex::from_file(&path).expect("File parsed to Codex.");

    println!("File opened.");

    loop {
        let mut main_prompt = prompt::ChoicePrompt::<MainAction>::new("What would you like to do next?");
        main_prompt
            .add_choice(vec!["quit", "q", "exit"], MainAction::Exit)
            .add_choice(vec!["introduce", "i"], MainAction::Introduce)
            .add_choice(vec!["practice", "p"], MainAction::Practice);

        match main_prompt.run_with_reprompt("Sorry, I didn't understand that...") {
            MainAction::Exit => {
                println!("Saving and Exiting.");
                break;
            }
            MainAction::Introduce => introduce(&mut codex),
            MainAction::Practice => println!("Practicing vocab [not yet implemented]"),
        }
    }

    println!("Saving to original file...");

    codex.to_file(&path);

    println!("Done!");
}

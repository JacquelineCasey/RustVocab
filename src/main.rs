
mod prompt;
mod codex;


use codex::{Codex, Confidence, Correctness, CodexAction};
use prompt::{Prompt, ChoicePrompt, ValidatedFieldPrompt};

use std::path::PathBuf;


#[derive(Clone, Copy)]
enum MainAction {
    Exit,
    Introduce,
    Practice
}


fn get_file_path() -> PathBuf {
    return loop {
        let file_prompt = ValidatedFieldPrompt::new(
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
            Some(s) => s.trim().to_owned(),
            None => continue,
        });

        if path.is_file() {
            break path
        }
        else {
            let create_file_anyways = ChoicePrompt::<bool>::new("That file does not exist. Should we create it? (y/n)")
                .add_choice(vec!["yes", "Yes", "y", "Y"], true)
                .add_choice(vec!["no", "No", "n", "N"], false)
                .run_with_reprompt("Please enter \"y\" or \"n\"...");

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
        let word = ValidatedFieldPrompt::new("Type a word to add (or !done): ", |s| {
            if s.contains('$') {
                println!("Word cannot contain '$'.");
                false
            }
            else if s.contains(' ') {
                println!("Word cannot contain space.");
                false
            }
            else if s.trim().len() == 0 {
                println!("Word not found.");
                false
            }
            else if s.trim().chars().nth(0) == Some('!') && s.trim() != "!done" {
                println!("\"!\" command not recognized.");
                false
            }
            else {
                true
            }
        }).run_with_reprompt("").trim().to_owned();

        if word == "!done" {
            println!("Finished introducing words.");
            break;
        }

        if codex.contains(&word) {
            println!("Note: \"{}\" already in codex. This will overwrite it. Type !done to abort.", word);
        }

        let def = ValidatedFieldPrompt::new("Type the definition: ", |s| {
            if s.contains('$') {
                println!("Defintion cannot contain '$'.");
                false
            }
            else if s.trim().len() == 0 {
                println!("Definition not found.");
                false
            }
            else if word.trim().chars().nth(0) == Some('!') && word.trim() != "!done" {
                println!("\"!\" command not recognized.");
                false
            }
            else {
                true
            }
        }).run_with_reprompt("").trim().to_owned();

        if def == "!done" {
            println!("Finished introducing words.");
            break;
        }

        let conf = ChoicePrompt::<Confidence>::new("How confidently do you know this word? ([k]nown, [p]artially known, [u]nknown)")
            .add_choice(vec!["known", "Known", "k", "K"], Confidence::Known)
            .add_choice(vec!["partial", "Partial", "p", "P"], Confidence::PartiallyKnown)
            .add_choice(vec!["unknown", "Unknown", "u", "U"], Confidence::Unknown)
            .run_with_reprompt("Unrecognized option.");

        codex.process_action(CodexAction::Introduce(word.clone(), conf, def));

        println!("Added \"{}\" to codex.", word);
    }
}

fn practice(codex: &mut Codex) {
    let num = ValidatedFieldPrompt::new(
        "How many words?", 
        |s| s.trim().parse::<i32>().is_ok() && s.trim().parse::<i32>().unwrap() > 0
    ).run();

    if let Some(num) = num {
        let num: usize = num.trim()
            .parse::<i32>()
            .unwrap()
            .try_into()
            .expect("checked into to usize should work.");
        
        for (word, def) in codex.generate_practice_set(num) {
            println!("Do you known \"{}\"?", word);
            println!("Hit enter to show definition...");

            let mut buf = String::new();
            std::io::stdin().read_line(&mut buf).expect("stdin should work.");

            println!("\"{}\": {}", word, def);

            let correctness = ChoicePrompt::<Correctness>::new("Were you correct? ([c]orrect, [p]artially correct, [i]ncorrect)")
                .add_choice(vec!["correct", "Correct", "c", "C"], Correctness::Correct)
                .add_choice(vec!["partially correct", "Partially correct", "partial", "Partial", "p", "P"], Correctness::PartiallyCorrect)
                .add_choice(vec!["incorrect", "Incorrect", "i", "I"], Correctness::Incorrect)
                .run_with_reprompt("");

            codex.process_action(CodexAction::Practice(word, correctness));
        }
    }
}


fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    println!("Welcome to Rust Vocab!");

    println!("First, we'll need to open (or create) a codex file.");

    let path = get_file_path();

    let mut codex = codex::Codex::from_file(&path).expect("File parsed to Codex.");

    println!("File opened. Found {} words.", codex.num_words());

    loop {
        let action = ChoicePrompt::<MainAction>::new("What would you like to do next? ([q]uit, [i]ntroduce, [p]ractice)")
            .add_choice(vec!["quit", "Quit", "q", "Q", "exit", "Exit"], MainAction::Exit)
            .add_choice(vec!["introduce", "Introduce", "i", "I"], MainAction::Introduce)
            .add_choice(vec!["practice", "Practice", "p", "P"], MainAction::Practice)
            .run_with_reprompt("Sorry, I didn't understand that...");

        match action {
            MainAction::Exit => {
                println!("Saving and Exiting.");
                break;
            }
            MainAction::Introduce => {
                introduce(&mut codex);
                println!("Now there are {} words.", codex.num_words());
            }
            MainAction::Practice => practice(&mut codex),
        }
    }

    println!("Saving to original file...");

    codex.to_file(&path);

    println!("Done!");
}

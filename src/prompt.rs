
use std::io::{self, Write};


const PROMPT_PREFIX: &str = "> ";


/* Prompt Interface */

pub trait Prompt<T: Clone> {
    fn get_text(&self) -> &str;

    fn handle_input(&self, input: &str) -> Option<T>;

    // TODO: add help functionality

    fn run(&self) -> Option<T> {
        println!("{}", self.get_text());
        print!("{}", PROMPT_PREFIX);
        io::stdout().flush().ok()?;
        
        let mut buf = String::new();
        
        let ret_val = match io::stdin().read_line(&mut buf) {
            Err(_) => panic!("stdin failed!"), // TODO: More graceful handling of this error
            Ok(_) => self.handle_input(buf.trim())
        };

        println!("");

        ret_val
    }

    fn run_with_reprompt(&self, reprompt_text: &str) -> T {
        loop {
            let result = self.run();
            if let Some(answer) = result {
                return answer;
            }
            
            if reprompt_text != "" {
                println!("{}", reprompt_text);
            }
        }
    }
}


/* Concrete Prompts */

pub struct ChoicePrompt<'a, T: Clone> {
    text: &'a str,
    items: Vec<ChoicePromptItem<'a, T>>
}

struct ChoicePromptItem<'a, T: Clone> {
    options: Vec<&'a str>,
    result: T,

    // TODO: Add help behavior, ie a description field. We could also do "did you mean?"
}

impl<'a, T: Clone> Prompt<T> for ChoicePrompt<'a, T> {
    fn get_text(&self) -> &str {
        &self.text
    }

    fn handle_input(&self, input: &str) -> Option<T> {
        Some(self.items.iter().find(
            |item| item.options.contains(&input.trim())
        )?.result.clone())
    }
}

impl<'a, T: Clone> ChoicePrompt<'a, T> {
    pub fn new(text: &'a str) -> ChoicePrompt<'a, T> {
        ChoicePrompt {text, items: vec![]}
    }

    // I like the builder pattern for these.
    pub fn add_choice<'b>(&'b mut self, options: Vec<&'a str>, result: T) -> &'b mut Self {
        self.items.push(ChoicePromptItem {options, result});
        self
    }
}


pub struct ValidatedFieldPrompt<'a, F>
    where F: Fn(&str) -> bool
{   
    text: &'a str,
    validator: F  // Allowed to print diagnostic information.

    // We could maybe add choices here as well, for something like "!exit" that can appear in here too.
}

impl<'a, F> Prompt<String> for ValidatedFieldPrompt<'a, F> 
    where F: Fn(&str) -> bool
{
    fn get_text(&self) -> &str {
        self.text
    }

    fn handle_input(&self, input: &str) -> Option<String> {
        match (self.validator)(input) {
            true => Some(input.to_owned()),
            false => None
        }
    }
}

impl<'a, F> ValidatedFieldPrompt<'a, F> 
    where F: Fn(&str) -> bool
{
    pub fn new(text: &'a str, validator: F) -> ValidatedFieldPrompt<'a, F> {
        ValidatedFieldPrompt {text, validator}
    }
}

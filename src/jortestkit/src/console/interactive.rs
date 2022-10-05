use crate::console::style;
use dialoguer::Input;
use thiserror::Error;
pub trait InteractiveCommandExec {
    fn parse_and_exec(
        &mut self,
        tokens: Vec<String>,
        console: ConsoleWriter,
    ) -> Result<(), InteractiveCommandError>;
}

#[derive(Debug)]
pub struct UserInteraction {
    console_writer: ConsoleWriter,
    exit_phrase: String,
    command_prefix: String,
}

impl UserInteraction {
    pub fn new(
        app_name: String,
        title: String,
        prompt: String,
        exit_phrase: String,
        command_prefix: String,
        description: Vec<String>,
    ) -> Self {
        Self {
            console_writer: ConsoleWriter::new(app_name, title, prompt, description),
            exit_phrase,
            command_prefix,
        }
    }

    pub fn interact<E: InteractiveCommandExec>(
        &self,
        executor: &mut E,
    ) -> Result<(), InteractiveCommandError> {
        self.console_writer.show_info();
        loop {
            self.console_writer.show_prompt();
            let tokens = self.read_line()?;

            if self.is_exit_command(&tokens) {
                return Ok(());
            }
            executor.parse_and_exec(tokens, self.console_writer.clone())?;
        }
    }

    fn read_line(&self) -> Result<Vec<String>, InteractiveCommandError> {
        let input: String = Input::new().with_prompt(&self.command_prefix).interact()?;
        Ok(input
            .split_ascii_whitespace()
            .map(|x| x.to_owned())
            .collect())
    }

    fn is_exit_command(&self, tokens: &[String]) -> bool {
        tokens
            .first()
            .unwrap()
            .eq_ignore_ascii_case(&self.exit_phrase)
    }
}

#[derive(Clone, Debug)]
pub struct ConsoleWriter {
    app_name: String,
    title: String,
    prompt: String,
    description: Vec<String>,
}

impl ConsoleWriter {
    pub fn new(app_name: String, title: String, prompt: String, description: Vec<String>) -> Self {
        Self {
            app_name,
            title,
            prompt,
            description,
        }
    }

    pub fn format_error(&self, err: InteractiveCommandError) {
        println!("{}", style::error.apply_to(format!("Error: {}", err)));
    }

    pub fn show_help(&self, error: InteractiveCommandError) {
        let message = format!("{}", error);
        //workaround for not showing app name
        println!("{}", message.replace(&self.app_name, ""));
    }

    pub fn show_prompt(&self) {
        println!("{}", style::success.apply_to(self.prompt.to_string()));
    }

    pub fn show_info(&self) {
        println!("----------------------------------------------------------------");
        println!(
            "{}",
            style::success.apply_to(&format!("Welcome to {}.", self.title))
        );
        for desciption_line in &self.description {
            println!("{}", style::success.apply_to(desciption_line.to_string()));
        }
        println!();
        println!(
            "{}",
            style::success.apply_to("Type help for more informations.")
        );
        println!("----------------------------------------------------------------");
    }
}

#[derive(Error, Debug)]
pub enum InteractiveCommandError {
    #[error("Custom error {0}")]
    UserError(String),
    #[error("io error")]
    IoError(#[from] std::io::Error),
}

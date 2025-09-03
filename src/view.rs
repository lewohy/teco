use std::time::Duration;

use crate::model::{TecoCase, Token, TokenizedContent};
use bat::Input;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

pub struct TecoSpinner {
    spinner: indicatif::ProgressBar,
}

impl TecoSpinner {
    pub fn new(case: &TecoCase) -> Self {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::with_template(
                format!(
                    "  {} {} {}",
                    " {prefix} ".bold(),
                    "{spinner}",
                    "{msg}"
                )
                .as_str(),
            )
            .unwrap(),
        );
        spinner.set_prefix(format!("{}", case.name));
        spinner.set_message(format!("Running...",));
        spinner.enable_steady_tick(Duration::from_millis(120));

        Self { spinner }
    }

    pub fn success(&self) {
        self.spinner.finish();

        self.spinner.set_style(
            ProgressStyle::with_template(
                format!("  {}", " {prefix}  {msg} ".on_bright_green()).as_str(),
            )
            .unwrap(),
        );
        self.spinner
            .set_message(format!("{}", format!(" Passed")));
    }

    pub fn fail(&self) {
        self.spinner.finish();

        self.spinner.set_style(
            ProgressStyle::with_template(
                format!("  {}", " {prefix}  {msg} ".on_bright_red()).as_str(),
            )
            .unwrap(),
        );
        self.spinner
            .set_message(format!("{}", format!(" Failed")));
    }

    pub fn unknown(&self) {
        self.spinner.finish();

        self.spinner.set_style(
            ProgressStyle::with_template(
                format!("  {}", " {prefix}  {msg} ".on_bright_yellow()).as_str(),
            )
            .unwrap(),
        );
        self.spinner
            .set_message(format!("{}", format!(" Unknown")));
    }
}

pub fn print(content: &str, title: Option<&str>) {
    bat::PrettyPrinter::new()
        .input(match title {
            Some(title) => Input::from_bytes(content.as_bytes()).title(title),
            None => Input::from_bytes(content.as_bytes()),
        })
        .header(match title {
            Some(_) => true,
            None => false,
        })
        .language("txt")
        .line_numbers(true)
        .grid(true)
        .print()
        .expect("bat::PrettyPrinter::print() 호출 실패");
}

pub fn print_tokenized_lines(tokenized: &TokenizedContent, title: Option<&str>) {
    let content = {
        let mut content = String::new();

        for line in &tokenized.lines {
            let token_strs: Vec<String> = line
                .iter()
                .map(|token| match token {
                    Token::Word(s) => s.to_string(),
                })
                .collect();

            content.push_str(&format!("{}\n", token_strs.join(" ")));
        }

        content
    };

    print(&content, title);
}

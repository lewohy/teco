use std::time::Duration;

use bat::Input;
use indicatif::{ProgressBar, ProgressStyle};

pub struct TecoSpinner {
    spinner: indicatif::ProgressBar,
}

impl TecoSpinner {
    pub fn new(message: String) -> Self {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::with_template(
                format!("{}  {}  {}", "{prefix}", "{spinner}", "{msg}").as_str(),
            )
            .unwrap(),
        );
        spinner.set_prefix(message);
        spinner.set_message(format!("Running...",));
        spinner.enable_steady_tick(Duration::from_millis(120));

        Self { spinner }
    }

    pub fn finish(self, message: String) {
        self.spinner.finish_and_clear();

        println!("{}", message);
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

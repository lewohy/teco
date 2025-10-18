mod model;
mod view;

use clap::Parser;
use colored::Colorize;
use spdlog::prelude::*;
use std::{
    io::{Read, Write},
    path::PathBuf,
    process::{Child, Command, Stdio},
    vec,
};

use crate::{
    model::{TecoCase, TecoResult, Token, TokenizedContent},
    view::TecoSpinner,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// 실행할 명령
    /// windows에서는 cmd /C "명령" 형태로 실행됨
    /// 그 외는 sh -c '명령' 형태로 실행됨
    #[arg(long)]
    command: String,

    /// *.in과 *.out 파일이 있는 디렉토리
    #[arg(long)]
    case_dir: String,

    /// input의 출력 여부
    #[arg(long, default_value_t = false)]
    show_input: bool,
}

fn run(child: Child, case: TecoCase) -> TecoResult {
    let mut input_file = std::fs::File::open(&case.input_file).expect("입력 파일을 열 수 없음");

    let input = {
        let mut input = String::new();
        input_file
            .read_to_string(&mut input)
            .expect("입력 파일을 읽을 수 없음");

        input
    };

    let output = {
        let mut stdin = child.stdin.expect("자식 프로세스의 stdin을 얻을 수 없음");
        stdin
            .write_all(input.as_bytes())
            .expect("자식 프로세스에 입력을 쓸 수 없음");
        drop(stdin);

        let mut stdout = child.stdout.expect("자식 프로세스의 stdout을 얻을 수 없음");

        let mut output = String::new();
        stdout
            .read_to_string(&mut output)
            .expect("자식 프로세스의 출력을 읽을 수 없음");

        output
    };

    let mut child_stderr = child.stderr.expect("자식 프로세스의 stderr을 얻을 수 없음");

    let mut stderr_content = String::new();
    child_stderr
        .read_to_string(&mut stderr_content)
        .expect("자식 프로세스의 stderr을 읽을 수 없음");

    TecoResult {
        case,
        input,
        output: TokenizedContent::new(output),
        stderr_content,
    }
}

fn main() {
    let args = Args::parse();
    let cases_path: PathBuf = args.case_dir.as_str().into();
    let cases = TecoCase::from_path(cases_path);

    info!("{} Cases loaded", cases.len());

    for case in cases {
        let spinner = TecoSpinner::new(&case);

        let mut command = if cfg!(target_os = "windows") {
            let mut command = Command::new("cmd");
            command.arg("/C").arg(&args.command);
            command
        } else {
            let mut command = Command::new("sh");
            command.arg("-c").arg(&args.command);
            command
        };

        let child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("프로세스를 실행할 수 없음");

        let result = run(child, case);

        match result.case.get_expected_tokens() {
            Some(expected) => {
                let mut output_tokens = result.output.tokens();

                let mut expected_content = String::new();
                let mut output_content = String::new();

                for expected_line in expected.lines {
                    let next_tokens = output_tokens
                        .by_ref()
                        .take(expected_line.len())
                        .collect::<Vec<&Token>>();

                    expected_content.push_str(&format!(
                        "{}\n",
                        expected_line
                            .iter()
                            .map(|token| match token {
                                Token::Word(s) => s.to_string(),
                            })
                            .collect::<Vec<String>>()
                            .join(" ")
                    ));

                    output_content.push_str(&format!(
                        "{}\n",
                        next_tokens
                            .iter()
                            .map(|token| match token {
                                Token::Word(s) => s.to_string(),
                            })
                            .collect::<Vec<String>>()
                            .join(" ")
                    ));
                }

                if expected_content == output_content {
                    spinner.success();

                    if args.show_input {
                        view::print(&result.input, Some("입력 내용"));
                    }
                } else {
                    spinner.fail();

                    if args.show_input {
                        view::print(&result.input, Some("입력 내용"));
                    }

                    let mut diff_lines: Vec<String> = vec![];

                    for (expected, actual) in expected_content
                        .lines()
                        .map(|line| line.trim())
                        .zip(output_content.lines().map(|line| line.trim()))
                    {
                        if expected == actual {
                            diff_lines.push(format!("{}", expected));
                        } else {
                            diff_lines.push(format!(
                                "{} {} {}",
                                expected.red().bold(),
                                " => ".on_blue(),
                                actual.green().bold()
                            ));
                        }
                    }

                    view::print(&diff_lines.join("\n"), Some("Comparison"));
                }
            }
            None => {
                spinner.unknown();

                if args.show_input {
                    view::print(&result.input, Some("입력 내용"));
                }

                view::print_tokenized_lines(&result.output, Some("stdout"));
            }
        }

        if !result.stderr_content.is_empty() {
            view::print(&result.stderr_content, Some("stderr"));
        }
    }
}

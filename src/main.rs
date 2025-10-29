mod model;
mod token;
mod view;

use clap::{Parser, ValueEnum};
use colored::Colorize;
use itertools::Itertools;
use std::{
    io::{Read, Write},
    path::PathBuf,
    process::{Child, Command, Stdio},
    vec,
};

use crate::{
    model::{TecoCase, TecoExecution, TecoResult},
    token::{construct_string, tokenize, Token},
    view::TecoSpinner,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum ShowCondition {
    Pass,
    Fail,
    Unknown,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// 실행할 명령
    /// windows에서는 cmd /C "명령" 형태로 실행됨
    /// 그 외는 sh -c '명령' 형태로 실행됨
    #[arg(long)]
    command: String,

    /// 테스트 케이스가 위치한 디렉토리
    #[arg(long)]
    case_dir: String,

    /// input의 출력 조건
    #[arg(
        long,
        value_enum,
        value_delimiter=',',
        num_args=1..,
        default_values_t=Vec::<ShowCondition>::new(),
        require_equals=true
    )]
    show_input: Vec<ShowCondition>,

    /// stdout 출력 조건
    #[arg(
        long,
        value_enum,
        value_delimiter=',',
        num_args=1..,
        default_values_t=Vec::<ShowCondition>::new(),
        require_equals=true
    )]
    show_stdout: Vec<ShowCondition>,

    /// stderr 출력 조건
    #[arg(
        long,
        value_enum,
        value_delimiter=',',
        num_args=1..,
        default_values_t=Vec::<ShowCondition>::new(),
        require_equals=true
    )]
    show_stderr: Vec<ShowCondition>,

    /// fail의 경우 diff 출력 여부
    #[arg(long)]
    show_diff: bool,

    /// input 파일의 확장자
    #[arg(long, default_value = "in")]
    input_ext: String,

    /// output 파일의 확장자
    #[arg(long, default_value = "out")]
    output_ext: String,
}

fn run(child: Child, case: TecoCase) -> TecoExecution {
    {
        let mut stdin = child.stdin.expect("자식 프로세스의 stdin을 얻을 수 없음");
        stdin
            .write_all(case.input_content.as_bytes())
            .expect("자식 프로세스에 입력을 쓸 수 없음");
    }

    let stdout_tokens = {
        let mut stdout = child.stdout.expect("자식 프로세스의 stdout을 얻을 수 없음");

        let mut stdout_content = String::new();
        stdout
            .read_to_string(&mut stdout_content)
            .expect("자식 프로세스의 출력을 읽을 수 없음");

        tokenize(stdout_content)
    };

    let stderr_content = {
        let mut stderr = child.stderr.expect("자식 프로세스의 stderr을 얻을 수 없음");

        let mut stderr_content = String::new();
        stderr
            .read_to_string(&mut stderr_content)
            .expect("자식 프로세스의 stderr을 읽을 수 없음");

        stderr_content
    };

    match &case.expected_tokens {
        Some(expected_tokens) => {
            let (expected_content, stdout_content) = {
                let mut stdout_token_iter = stdout_tokens.iter().filter_map(|token| match token {
                    Token::Word(word) => Some(word.clone()),
                    Token::Newline => None,
                });

                let mut expected_content_lines: Vec<Vec<String>> = vec![vec![]];
                let mut stdout_content_lines: Vec<Vec<String>> = vec![vec![]];

                for expected_token in expected_tokens {
                    match expected_token {
                        Token::Word(word) => {
                            expected_content_lines
                                .last_mut()
                                .unwrap()
                                .push(word.clone());
                            stdout_content_lines
                                .last_mut()
                                .unwrap()
                                .push(stdout_token_iter.next().unwrap());
                        }
                        Token::Newline => {
                            expected_content_lines.push(vec![]);
                            stdout_content_lines.push(vec![]);
                        }
                    }
                }

                (
                    expected_content_lines
                        .into_iter()
                        .map(|line| line.join(" "))
                        .join("\n"),
                    stdout_content_lines
                        .into_iter()
                        .map(|line| line.join(" "))
                        .join("\n"),
                )
            };

            if stdout_content == expected_content {
                TecoExecution {
                    case,
                    stdout_content,
                    stderr_content,
                    result: TecoResult::Pass,
                }
            } else {
                TecoExecution {
                    case,
                    stdout_content,
                    stderr_content,
                    result: TecoResult::Fail { expected_content },
                }
            }
        }
        None => {
            let stdout_content = construct_string(stdout_tokens);

            TecoExecution {
                case,
                stdout_content,
                stderr_content,
                result: TecoResult::Unknown,
            }
        }
    }
}

fn main() {
    let args = Args::parse();
    let cases_path: PathBuf = args.case_dir.as_str().into();
    let cases = TecoCase::from_path(cases_path, &args.input_ext, &args.output_ext);
    let total_cases = cases.len();

    for (index, case) in cases.into_iter().enumerate() {
        let spinner = TecoSpinner::new(format!(
            "{:>2}/{:<2}  {:<20}",
            index + 1,
            total_cases,
            case.name.as_str().bold()
        ));

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

        let execution = run(child, case);

        spinner.finish(format!(
            "{:>2}/{:<2}  {:<20}  {}",
            index + 1,
            total_cases,
            execution.case.name.bold(),
            match execution.result {
                TecoResult::Pass { .. } => "PASS".green().bold(),
                TecoResult::Fail { .. } => "FAIL".red().bold(),
                TecoResult::Unknown { .. } => "UNKNOWN".yellow().bold(),
            }
        ));

        let current_condition = match execution.result {
            TecoResult::Pass { .. } => ShowCondition::Pass,
            TecoResult::Fail { .. } => ShowCondition::Fail,
            TecoResult::Unknown { .. } => ShowCondition::Unknown,
        };

        if args.show_input.contains(&current_condition) {
            view::print(&execution.case.input_content, Some("input"));
        }

        if args.show_stdout.contains(&current_condition) {
            view::print(&execution.stdout_content, Some("stdout"));
        }

        if args.show_stderr.contains(&current_condition) && !execution.stderr_content.is_empty() {
            view::print(&execution.stderr_content, Some("stderr"));
        }

        match execution.result {
            TecoResult::Pass => {}
            TecoResult::Fail { expected_content } => {
                if args.show_diff {
                    let mut diff_lines: Vec<String> = vec![];

                    for (expected, actual) in expected_content
                        .lines()
                        .map(|line| line.trim())
                        .zip(execution.stdout_content.lines().map(|line| line.trim()))
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
            TecoResult::Unknown => {}
        }
    }
}

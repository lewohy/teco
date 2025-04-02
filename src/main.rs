use bat::Input;
use clap::Parser;
use colored::Colorize;
use indicatif::ProgressBar;
use spdlog::prelude::*;
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::Duration,
    vec,
};

#[derive(Debug)]
struct TestCase {
    name: String,
    input_file_path: PathBuf,
    output_file_path: Option<PathBuf>,
}

#[derive(Debug)]
struct TestCaseResult<'a> {
    test_case: &'a TestCase,
    input_content: String,
    stdout_content: String,
    stderr_content: String,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// 타겟 바이너리 경로
    #[arg(short, long)]
    target: String,

    /// *.in과 *.out 파일이 있는 디렉토리
    #[arg(short, long)]
    case_dir: String,

    /// input의 출력 여부
    #[arg(short, long, default_value_t = false)]
    show_input: bool,
}

impl Ord for TestCase {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        return self.name.cmp(&other.name);
    }
}

impl Eq for TestCase {}

impl PartialOrd for TestCase {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        return Some(self.name.cmp(&other.name));
    }
}

impl PartialEq for TestCase {
    fn eq(&self, other: &Self) -> bool {
        return self.name == other.name;
    }
}

fn get_test_cases(path: &Path) -> Vec<TestCase> {
    let mut paths: Vec<TestCase> = vec![];

    for entry in path
        .read_dir()
        .expect("테스트 케이스 디렉토리를 열 수 없음")
    {
        let input_file_path = entry.expect("entry를 얻을 수 없음").path();

        if !input_file_path.is_file() {
            continue;
        }

        let filename = input_file_path
            .file_name()
            .expect("파일 이름을 얻을 수 없음")
            .to_str()
            .expect("파일 이름을 문자열로 변환할 수 없음");

        if !filename.ends_with(".in") {
            continue;
        }

        let test_name = filename
            .split('.')
            .next()
            .expect("확장자를 분리할 수 없음")
            .to_string();
        let output_file_path = match input_file_path
            .parent()
            .expect("부모 디렉토리를 얻을 수 없음")
            .join(format!("{}.out", test_name))
        {
            path if path.exists() => Some(path),
            _ => None,
        };

        paths.push(TestCase {
            name: test_name,
            input_file_path,
            output_file_path,
        });
    }

    paths.sort();

    paths
}

fn run<'a>(child: &'a mut Child, test_case: &'a TestCase) -> TestCaseResult<'a> {
    let mut input_file =
        std::fs::File::open(&test_case.input_file_path).expect("입력 파일을 열 수 없음");

    let mut input = String::new();
    input_file
        .read_to_string(&mut input)
        .expect("입력 파일을 읽을 수 없음");

    {
        let mut child_stdin = child
            .stdin
            .take()
            .expect("자식 프로세스의 stdin을 얻을 수 없음");
        child_stdin
            .write_all(input.as_bytes())
            .expect("자식 프로세스에 입력을 쓸 수 없음");
    }

    let mut child_stdout = child
        .stdout
        .take()
        .expect("자식 프로세스의 stdout을 얻을 수 없음");

    let mut stdout_content = String::new();
    child_stdout
        .read_to_string(&mut stdout_content)
        .expect("자식 프로세스의 출력을 읽을 수 없음");

    let mut child_stderr = child
        .stderr
        .take()
        .expect("자식 프로세스의 stderr을 얻을 수 없음");

    let mut stderr_content = String::new();
    child_stderr
        .read_to_string(&mut stderr_content)
        .expect("자식 프로세스의 stderr을 읽을 수 없음");

    TestCaseResult {
        test_case,
        input_content: input,
        stdout_content,
        stderr_content,
    }
}

fn print(content: &str, title: Option<&str>) {
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

fn main() {
    let args = Args::parse();

    let test_case_path = Path::new(args.case_dir.as_str());

    let test_cases = get_test_cases(&test_case_path);

    info!("{}개의 테스트 케이스 로드됨", test_cases.len());

    for (index, test_case) in test_cases.iter().enumerate() {
        info!(
            "{} Case{}: {}",
            if index < test_cases.len() - 1 {
                "├"
            } else {
                "└"
            },
            index,
            test_case.name
        );
    }

    for test_case in &test_cases {
        let spinner = ProgressBar::new_spinner();
        spinner.set_message(format!("{} 테스트 중...", test_case.name));
        spinner.enable_steady_tick(Duration::from_millis(120));

        let mut child = Command::new(Path::new(&args.target))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("프로세스를 실행할 수 없음");

        let test_case_result = run(&mut child, test_case);
        spinner.finish_and_clear();

        match test_case.output_file_path {
            Some(ref path) => {
                let mut output_file = std::fs::File::open(path).expect("출력 파일을 열 수 없음");
                let mut expected_output = String::new();
                output_file
                    .read_to_string(&mut expected_output)
                    .expect("출력 파일을 읽을 수 없음");

                if test_case_result.stdout_content == expected_output {
                    println!(
                        "{}",
                        format!(" {}:  테스트 성공", test_case_result.test_case.name).cyan()
                    );

                    if args.show_input {
                        print(&test_case_result.input_content, Some("입력 내용"));
                    }
                } else {
                    println!(
                        "{}",
                        format!(" {}:  테스트 실패", test_case_result.test_case.name).red()
                    );

                    if args.show_input {
                        print(&test_case_result.input_content, Some("입력 내용"));
                    }

                    let mut diff_lines: Vec<String> = vec![];

                    for (expected, actual) in expected_output.lines().map(|line| line.trim()).zip(
                        test_case_result
                            .stdout_content
                            .lines()
                            .map(|line| line.trim()),
                    ) {
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

                    print(&diff_lines.join("\n"), Some("Comparison"));
                }

                if !test_case_result.stderr_content.is_empty() {
                    print(&test_case_result.stderr_content, Some("stderr"));
                }
            }
            None => {
                println!(
                    "{}",
                    format!(" {}:  예시 출력파일이 존재하지 않음", test_case.name).yellow()
                );

                if args.show_input {
                    print(&test_case_result.input_content, Some("입력 내용"));
                }

                if !test_case_result.stderr_content.is_empty() {
                    print(&test_case_result.stderr_content, Some("stderr"));
                }

                print(&test_case_result.stdout_content, Some("stdout"));
            }
        }
    }
}

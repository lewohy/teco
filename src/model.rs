use std::{fs::File, io::Read, path::PathBuf};

use crate::token::{tokenize, Token};

#[derive(Debug, Clone)]
pub struct TecoCase {
    pub name: String,
    pub input_content: String,
    pub expected_tokens: Option<Vec<Token>>,
}

#[derive(Debug, Clone)]
pub enum TecoResult {
    Pass,
    Fail { expected_content: String },
    Unknown,
}

#[derive(Debug, Clone)]
pub struct TecoExecution {
    pub case: TecoCase,
    pub stdout_content: String,
    pub stderr_content: String,
    pub result: TecoResult,
}

impl TecoCase {
    pub fn from_path(path: PathBuf, input_ext: &str, output_ext: &str) -> Vec<TecoCase> {
        let mut cases: Vec<TecoCase> = vec![];

        for entry in path
            .read_dir()
            .expect("테스트 케이스 디렉토리를 열 수 없음")
        {
            let input_file_path = entry.unwrap().path();

            let input_content = {
                let mut input_file =
                    std::fs::File::open(&input_file_path).expect("입력 파일을 열 수 없음");
                let mut input = String::new();

                input_file
                    .read_to_string(&mut input)
                    .expect("입력 파일을 읽을 수 없음");

                input
            };

            let filename = input_file_path
                .file_name()
                .expect("파일 이름을 얻을 수 없음")
                .to_str()
                .expect("파일 이름을 문자열로 변환할 수 없음");

            let input_ext_part = format!(".{}", input_ext);

            if !filename.ends_with(&input_ext_part) {
                continue;
            }

            let test_name = filename.trim_end_matches(&input_ext_part).to_string();
            let expected_file_path = match input_file_path
                .parent()
                .expect("input 파일의 부모 디렉토리를 얻을 수 없음")
                .join(format!("{}.{}", test_name, output_ext))
            {
                path if path.exists() => Some(path),
                _ => None,
            };
            let expected_content = match expected_file_path {
                Some(path) => {
                    let mut content = String::new();
                    File::open(path)
                        .expect("expect 파일을 열 수 없음")
                        .read_to_string(&mut content)
                        .unwrap();

                    Some(content)
                }
                None => None,
            };

            cases.push(TecoCase {
                name: test_name,
                input_content,
                expected_tokens: match expected_content {
                    Some(content) => Some(tokenize(content)),
                    None => None,
                },
            });
        }

        cases.sort();

        cases
    }
}

impl Ord for TecoCase {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        return self.name.cmp(&other.name);
    }
}

impl Eq for TecoCase {}

impl PartialOrd for TecoCase {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        return Some(self.name.cmp(&other.name));
    }
}

impl PartialEq for TecoCase {
    fn eq(&self, other: &Self) -> bool {
        return self.name == other.name;
    }
}

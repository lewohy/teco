use std::{io::Read, path::PathBuf};

#[derive(Debug)]
pub enum Token {
    Word(String),
    // Space(String),
}

#[derive(Debug)]
pub struct TokenizedContent {
    pub lines: Vec<Vec<Token>>,
}

#[derive(Debug)]
pub struct TecoCase {
    pub name: String,
    pub input_file: PathBuf,
    pub expected_file: Option<PathBuf>,
}

#[derive(Debug)]
pub struct TecoResult {
    pub case: TecoCase,
    pub input: String,
    pub output: TokenizedContent,
    pub stderr_content: String,
}

impl Token {
    pub fn tokenize(source: &str) -> Vec<Self> {
        return source
            .split_whitespace()
            .map(|s| Token::Word(s.to_string()))
            .collect();
    }
}

impl TokenizedContent {
    pub fn new(content: String) -> Self {
        Self {
            lines: content
                .lines()
                .map(|line| Token::tokenize(line))
                .collect::<Vec<Vec<Token>>>()
                .into(),
        }
    }

    pub fn tokens(&self) -> impl Iterator<Item = &Token> {
        self.lines.iter().flatten()
    }
}

impl TecoCase {
    pub fn from_path(path: PathBuf, input_ext: &str, output_ext: &str) -> Vec<TecoCase> {
        let mut paths: Vec<TecoCase> = vec![];

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

            let input_ext_part = format!(".{}", input_ext);

            if !filename.ends_with(&input_ext_part) {
                continue;
            }

            let test_name = filename.trim_end_matches(&input_ext_part).to_string();
            let output_file_path = match input_file_path
                .parent()
                .expect("부모 디렉토리를 얻을 수 없음")
                .join(format!("{}.{}", test_name, output_ext))
            {
                path if path.exists() => Some(path),
                _ => None,
            };

            paths.push(TecoCase {
                name: test_name,
                input_file: input_file_path,
                expected_file: output_file_path,
            });
        }

        paths.sort();

        paths
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

impl TecoCase {
    pub fn get_expected_tokens(&self) -> Option<TokenizedContent> {
        if let Some(expected_path) = &self.expected_file {
            let mut expected_file =
                std::fs::File::open(expected_path).expect("예시 파일을 열 수 없음");

            let expected = {
                let mut content = String::new();
                expected_file
                    .read_to_string(&mut content)
                    .expect("기대 출력 파일을 읽을 수 없음");

                content
            };

            return Some(TokenizedContent::new(expected));
        }

        return None;
    }
}

# teco

## Install

https://crates.io/crates/teco

```sh
cargo install teco
```

## Usage

```sh
Usage: teco [OPTIONS] --command <COMMAND> --case-dir <CASE_DIR>

Options:
      --command <COMMAND>             실행할 명령 windows에서는 cmd /C "명령" 형태로 실행됨 그 외는 sh -c '명령' 형태로 실행됨
      --case-dir <CASE_DIR>           테스트 케이스가 위치한 디렉토리
      --show-input=<SHOW_INPUT>...    input의 출력 조건 [possible values: pass, fail, unknown]
      --show-stdout=<SHOW_STDOUT>...  stdout 출력 조건 [possible values: pass, fail, unknown]
      --show-stderr=<SHOW_STDERR>...  stderr 출력 조건 [possible values: pass, fail, unknown]
      --show-diff                     fail의 경우 diff 출력 여부
      --input-ext <INPUT_EXT>         input 파일의 확장자 [default: in]
      --output-ext <OUTPUT_EXT>       output 파일의 확장자 [default: out]
  -h, --help                          Print help
  -V, --version                       Print version
```



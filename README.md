# teco

## Usage

```sh
Usage: teco [OPTIONS] --command <COMMAND> --case-dir <CASE_DIR>

Options:
      --command <COMMAND>    실행할 명령 windows에서는 cmd /C "명령" 형태로 실행됨 그 외는 sh -c '명령' 형태로 실행됨
      --case-dir <CASE_DIR>  *.in과 *.out 파일이 있는 디렉토리
      --show-input           input의 출력 여부
  -h, --help                 Print help
  -V, --version              Print version
```


- [ ] target에 arguments를 넣을 수 있도록 수정
- [ ] input, output 파일을 지정할 수 있도록 수정

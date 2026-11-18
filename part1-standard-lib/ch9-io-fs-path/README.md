# ch9. 입출력·파일·경로: `Read`/`Write`/`BufRead`/`Seek`, `std::fs`, `Path`/`PathBuf`

앞 챕터들은 입출력을 필요한 만큼만 썼습니다. [Part 0 ch2](../../part0-getting-started/ch2-guessing-game/README.md)에서 `stdin().read_line`으로 한 줄을 읽었고, [Part 0 ch9](../../part0-getting-started/ch9-error-handling/README.md)에서 `File::open`의 에러를 `ErrorKind`로 나눴고, [Part 0 ch12](../../part0-getting-started/ch12-io-project-minigrep/README.md)에서 `fs::read_to_string`과 `eprintln!`을, [Part 0 ch20](../../part0-getting-started/ch20-web-server-project/README.md)에서 `TcpStream`을 `BufReader`로 감싸 줄 단위로 읽었습니다. 그리고 몇 가지를 이 챕터로 미뤄 두었습니다.

- [Part 1 ch3](../ch3-strings-fmt/README.md): `fmt::Write`와 짝이 되는 `io::Write`, 파일과 표준 출력에 `write!`/`writeln!`으로 쓰기
- [Part 1 ch6](../ch6-core-traits/README.md): 파일 함수들이 `P: AsRef<Path>`로 `&str`, `String`, `PathBuf`를 모두 받는 이유
- [Part 1 ch8](../ch8-errors-panic/README.md): `io::Error`의 세부(`ErrorKind`, 운영체제 에러 번호, `io::Error::new`/`other`)와, `io::Error`를 감싸 "어느 파일에서" 실패했는지 알리는 방법

이 챕터는 세 모듈을 다룹니다. `std::io`는 바이트를 읽고 쓰는 **방법**(트레이트와 버퍼, 표준 입출력, 에러)을, `std::fs`는 파일과 디렉터리라는 **대상**을, `std::path`는 그 대상의 **이름**인 경로를 다룹니다.

- 패키지: `p1-ch9-io-fs-path`
- 기준 문서: [`std::io`](https://doc.rust-lang.org/std/io/index.html), [`std::fs`](https://doc.rust-lang.org/std/fs/index.html), [`std::path`](https://doc.rust-lang.org/std/path/index.html)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (Edition 2024), macOS. 경로 예제의 결과는 유닉스(macOS, Linux) 기준이고, 운영체제의 에러 메시지와 에러 번호는 macOS에서 확인한 값입니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. `Read`·`Write`·`BufRead`·`Seek`의 역할과 핵심 메서드를 설명할 수 있다. `read`가 요청보다 적게 읽을 수 있는 이유와 `read_exact`·`write_all`이 필요한 이유를 설명하고, 리더와 라이터를 제네릭으로 받는 함수를 작성해 `&[u8]`·`Vec<u8>`·`Cursor` 같은 메모리 버퍼로 시험할 수 있다.
2. 버퍼링이 시스템 호출 횟수를 줄이는 원리를 설명하고 `BufReader`/`BufWriter`를 알맞게 쓸 수 있다. `BufWriter`를 `flush` 없이 버리면 생기는 문제와 표준 출력의 줄 버퍼링, `stdout().lock()`을 설명할 수 있다.
3. 표준 입력을 줄 단위로 읽어 EOF에서 끝나는 프로그램을 작성하고, 결과는 표준 출력에, 안내는 표준 에러에 나눠 쓸 수 있다.
4. `std::fs`의 편의 함수와 `File`·`OpenOptions`로 파일을 읽고 쓰고 덧붙이며, 메타데이터 확인, 복사·이름 바꾸기·삭제, 디렉터리 생성·순회·삭제를 할 수 있다. `io::Error`의 `kind()`와 운영체제 에러 번호를 읽고, 경로를 담은 에러 타입으로 감싸 보고할 수 있다.
5. `Path`/`PathBuf`로 경로를 구성 요소 단위로 나누고 조립할 수 있다. `display()`가 필요한 이유, `AsRef<Path>`를 받는 이유, 경로 연산과 `exists`·`canonicalize`처럼 파일 시스템에 묻는 연산의 차이를 설명할 수 있다.

## 2. 핵심 개념

### 2.1 네 트레이트와 그것을 구현한 타입

입출력의 중심은 두 트레이트입니다. `Read`를 구현한 타입을 **리더(reader)**, `Write`를 구현한 타입을 **라이터(writer)** 라고 부릅니다(`std::io` 문서의 표현). 두 트레이트는 바이트를 다룹니다. 문자열은 바이트를 UTF-8로 해석한 결과일 뿐이라서, 문자열을 쓰고 읽는 메서드도 결국 바이트를 주고받습니다.

| 트레이트 | 하는 일 | 필수 메서드 |
| --- | --- | --- |
| `Read` | 바이트를 받아 오기 | `read` |
| `Write` | 바이트를 내보내기 | `write`, `flush` |
| `BufRead: Read` | 안쪽 버퍼를 이용한 읽기(줄 단위 등) | `fill_buf`, `consume` |
| `Seek` | 읽기·쓰기 위치 옮기기 | `seek` |

이 트레이트들은 파일만의 것이 아닙니다. 1.100.0 문서의 구현 목록에서 자주 쓰는 것을 골랐습니다.

| 타입 | `Read` | `Write` | `BufRead` | `Seek` |
| --- | --- | --- | --- | --- |
| `File`, `&File` | O | O | | O |
| `TcpStream`, `&TcpStream` (`ch12-net`) | O | O | | |
| `Stdin` | O | | | |
| `StdinLock` (`stdin().lock()`) | O | | O | |
| `Stdout`, `Stderr`, `StdoutLock`, `StderrLock` | | O | | |
| `&[u8]` | O | | O | |
| `Vec<u8>` | | O | | |
| `&mut [u8]` (크기가 정해진 버퍼) | | O | | |
| `Cursor<T>` | O | O (`T`가 `Vec<u8>`, `&mut [u8]` 등일 때) | O | O |
| `BufReader<R>` | O | | O | O (`R: Seek`일 때) |
| `BufWriter<W>` | | O | | O (`W: Seek`일 때) |
| `io::empty()` | O | O | O | O |
| `io::sink()` | | O | | |

표가 말해 주는 것은 **"입출력 코드를 파일 없이 시험할 수 있다"** 는 점입니다. 함수가 `File` 대신 `impl Read`나 `impl Write`를 받으면, 실제 프로그램에서는 파일이나 표준 입력을 넘기고 시험할 때는 바이트 슬라이스나 `Vec<u8>`을 넘깁니다(2.6절).

### 2.2 `Read`: "최대 n바이트"를 채워 주는 트레이트

```rust
fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>
```

`read`는 `buf`를 **최대** `buf.len()`바이트까지 채우고 실제로 채운 개수 `n`을 돌려줍니다. 문서가 정한 약속은 다음과 같습니다.

- `0 <= n <= buf.len()`입니다. `n`이 요청보다 작아도 에러가 아니고, 아직 끝이 아닐 수도 있습니다. 파이프나 네트워크에서는 데이터가 조금씩 도착하므로 이런 일이 흔합니다.
- `n == 0`이면 더 읽을 것이 없다는 뜻(EOF, 입력의 끝)입니다(`buf`가 빈 경우 제외).
- `ErrorKind::Interrupted` 에러는 신호(signal) 때문에 잠깐 끊긴 것이라 다시 시도하면 됩니다.

[`01_read_write_traits.rs`](examples/01_read_write_traits.rs)의 `Trickle`은 한 번에 5바이트만 내주는 리더입니다. 8바이트 버퍼를 넘겨도 5, 5, 3, 0바이트가 돌아옵니다. 그래서 "정확히 n바이트"나 "끝까지"가 필요할 때는 `read`를 반복해 주는 제공 메서드를 씁니다.

| 메서드 | 하는 일 | 끝에서 |
| --- | --- | --- |
| `read_exact(&mut buf)` | `buf`를 꽉 채울 때까지 반복 | 모자라면 `UnexpectedEof` 에러 |
| `read_to_end(&mut vec)` | EOF까지 읽어 `Vec<u8>` 뒤에 덧붙임 | 읽은 바이트 수 |
| `read_to_string(&mut s)` | EOF까지 읽어 `String` 뒤에 덧붙임 | UTF-8이 아니면 `InvalidData` 에러 |
| `bytes()` | 한 바이트씩 내주는 반복자 | 버퍼 없는 리더에는 느림(2.7절) |
| `take(n)`, `chain(r)`, `by_ref()` | n바이트까지만 / 이어 읽기 / 소비하지 않고 빌려 쓰기 | |

`read_to_string`은 끝까지 모은 뒤 UTF-8을 검사하므로, 2바이트씩 잘려 들어온 한글도 제대로 읽습니다(예제 01의 2번 구역). 반대로 직접 `read`한 조각을 `String::from_utf8`로 바꾸면 글자 중간에서 잘려 실패할 수 있습니다.

**왜 `&mut self`인가.** 읽고 나면 리더 안의 **읽기 위치**가 앞으로 움직입니다. `&[u8]`이면 슬라이스의 앞부분이 잘려 나가고, `File`이면 운영체제가 관리하는 파일 위치가 바뀝니다. 그래서 "읽기만 하는" 함수도 리더를 `&mut`로 받아야 합니다(4.5절).

**`Read` 직접 구현하기.** 안정 버전에서는 `read` 하나만 구현하면 나머지 메서드가 모두 따라옵니다. 1.100.0 문서는 `read`를 제공 메서드로 표시하고 "`read_buf`와 `read` 중 적어도 하나는 구현해야 한다"고 적지만, `read_buf`는 nightly 전용 실험 API입니다. 아무것도 구현하지 않으면 E0046 ``missing one of `read_buf`, `read` in implementation``이 납니다.

### 2.3 `Write`: `write`와 `write_all`, 그리고 두 `Write`

`write`도 `read`와 같은 계약입니다. **일부만 쓰고** 쓴 개수를 돌려줄 수 있습니다. 예제 01의 `Narrow`는 한 번에 4바이트만 받으므로 `write(b"hello, world")`가 `Ok(4)`를 돌려줍니다. 그래서 보통은 다 쓸 때까지 반복해 주는 `write_all`을 씁니다. `write`의 반환값을 버리는 코드(`file.write(buf)?;`)는 거의 항상 버그라서 clippy의 `unused_io_amount`가 **기본으로 에러(deny)** 를 냅니다. `read`도 마찬가지입니다.

`flush`는 중간에 모아 둔 데이터를 최종 목적지까지 내보내라는 요청입니다. 모아 두는 것이 없는 라이터(`Vec<u8>`, `File`)에서는 할 일이 없고, `BufWriter`나 표준 출력처럼 버퍼가 있는 라이터에서 의미가 있습니다(2.7절).

**`write!`가 두 트레이트에 모두 쓰이는 이유.** Part 1 ch3에서 본 것처럼 `write!(dst, ...)`는 `dst.write_fmt(format_args!(...))`로 펼쳐집니다. `write_fmt`라는 메서드는 두 트레이트에 모두 있습니다.

| | `std::fmt::Write` | `std::io::Write` |
| --- | --- | --- |
| 받는 것 | 문자열(`&str`) | 바이트(`&[u8]`) |
| 구현한 타입 | `String`, `Formatter` | `File`, `Vec<u8>`, `Stdout`, `TcpStream`, … |
| `write!`의 반환 | `fmt::Result` | `io::Result<()>` |
| 실패 | `String`에 쓰기는 실패하지 않음 | 디스크 가득 참, 연결 끊김 등 언제든 가능 |

어느 쪽 `write_fmt`가 불릴지는 **스코프에 가져온 트레이트**가 정합니다. 파일에 쓰는데 ch3의 습관대로 `use std::fmt::Write;`를 쓰면 E0599 ``cannot write into `File` ``이 납니다(4.1절). 한 파일에서 둘 다 필요하면 한쪽을 `use std::fmt::Write as _;`로 이름 없이 가져옵니다.

```rust
use std::fmt::Write as _; // String용
use std::io::Write;       // 바이트용

let mut bytes: Vec<u8> = Vec::new();
writeln!(bytes, "{}+{}={}", 2, 3, 2 + 3)?;          // io::Result<()>
let mut text = String::new();
writeln!(text, "{}+{}={}", 2, 3, 2 + 3).unwrap();   // fmt::Result
```

`print!`/`println!`은 `io::stdout()`에 대한 `write!`에 가깝지만, 실패하면 `Err`를 돌려주는 대신 **패닉**합니다(2.8절).

### 2.4 `BufRead`: 줄 단위로 읽기

줄 단위로 읽으려면 줄바꿈이 나올 때까지 미리 읽어 둔 버퍼가 필요합니다. `BufRead`는 그런 버퍼를 가진 리더의 트레이트입니다. `File`은 버퍼가 없어서 `BufRead`가 아니므로 `BufReader`로 감싸야 합니다(4.2절). `&[u8]`, `Cursor`, `StdinLock`은 이미 메모리나 버퍼에 데이터가 있어서 바로 `BufRead`입니다.

| 메서드 | 결과 | 줄 끝 문자 |
| --- | --- | --- |
| `lines()` | `io::Result<String>`을 내주는 반복자 | `\n`과 `\r\n` 모두 떼어 냄 |
| `read_line(&mut s)` | 읽은 바이트 수 (0이면 EOF), `s` 뒤에 덧붙임 | 남겨 둠 |
| `split(b';')` | `io::Result<Vec<u8>>`을 내주는 반복자 | 구분자를 떼어 냄 |
| `read_until(b';', &mut v)` | 읽은 바이트 수, `v` 뒤에 덧붙임 | 구분자까지 남김 |

[`02_bufread_lines.rs`](examples/02_bufread_lines.rs)에서 확인한 세부 사항입니다.

- 마지막 줄에 줄바꿈이 없어도 한 줄로 나옵니다. `split(b';')`은 끝에 구분자가 있어도 빈 조각을 하나 더 만들지 않습니다(`"a;bb;;ccc;"` → 4조각).
- `read_line`은 Part 0 ch2에서 본 것처럼 **덧붙이므로**, 버퍼를 재사용할 때는 매번 `clear()`합니다. 줄마다 새 `String`을 만드는 `lines()`보다 할당이 적습니다.
- 어떤 줄이 UTF-8이 아니면 `lines()`는 그 줄에서 `InvalidData` 에러를 냅니다. 바이트가 필요하면 `split(b'\n')`으로 받고 `String::from_utf8_lossy`로 바꿉니다.
- `&[u8]`에는 같은 이름의 고유 메서드 `slice::split`(클로저를 받음)이 있어서 `bytes.split(b';')`는 그쪽이 골라지고 E0277이 납니다. 고유 메서드가 트레이트 메서드보다 먼저 골라지기 때문입니다. `Cursor::new(bytes).split(b';')`나 `BufRead::split(bytes, b';')`로 부릅니다.

`fill_buf`/`consume`은 버퍼를 직접 들여다보는 저수준 메서드로, `BufRead`를 직접 구현할 때가 아니면 쓸 일이 드뭅니다.

### 2.5 `Seek`와 `Cursor`

리더와 라이터에는 다음에 읽거나 쓸 **위치**가 있습니다. `Seek`는 그 위치를 옮깁니다.

```rust
cursor.seek(SeekFrom::Start(2))?;    // 처음에서 2바이트
cursor.seek(SeekFrom::Current(2))?;  // 지금 위치에서 2바이트 뒤로 (음수면 앞으로)
cursor.seek(SeekFrom::End(-2))?;     // 끝에서 2바이트 앞
cursor.stream_position()?;           // 지금 위치 (1.51)
cursor.rewind()?;                    // 처음으로 = seek(SeekFrom::Start(0)) (1.55)
```

`seek`는 옮긴 뒤의 위치(처음부터의 바이트 수)를 돌려줍니다. `Cursor<T>`는 메모리 버퍼에 이 위치를 붙여 주는 타입입니다. `Vec<u8>`은 `Write`만, `&[u8]`은 `Read`만 구현하지만, `Cursor<Vec<u8>>`은 넷을 모두 구현하므로 "파일처럼" 쓰고, 되감고, 다시 읽을 수 있습니다. 위치를 옮긴 뒤 쓰면 그 자리를 **덮어씁니다**(끼워 넣지 않음, 예제 01의 5번 구역).

`File`의 위치는 열린 파일 하나에 하나뿐이고 읽기와 쓰기가 함께 씁니다. 그래서 `read_to_string`으로 끝까지 읽은 파일을 다시 읽으면 0바이트가 나오고, `rewind()`해야 처음부터 다시 읽힙니다(예제 05의 4번 구역). `Read` 문서도 같은 상황을 경고합니다.

### 2.6 리더와 라이터를 제네릭으로 받기

```rust
fn longest_line(input: impl BufRead) -> io::Result<Option<(usize, String)>>
fn number_lines(input: impl BufRead, out: &mut impl Write) -> io::Result<Stats>
```

이 함수들은 데이터가 어디서 오고 어디로 가는지 모릅니다. 예제 02는 같은 `longest_line`에 `&[u8]`, `Cursor<Vec<u8>>`, `BufReader<File>`, `io::empty()`를 넘기고, `#[cfg(test)]` 테스트에서는 바이트 슬라이스만으로 시험합니다. 예제 04의 `number_lines`는 실제로는 표준 입출력을, 시험할 때는 `&[u8]`과 `Vec<u8>`을 받습니다.

- **값으로 받을까, `&mut`로 받을까.** 표준 라이브러리에는 `impl<R: Read + ?Sized> Read for &mut R`(`Write`, `BufRead`도 같음)이 있어서, `impl Read`를 받는 함수에 `&mut reader`를 넘겨도 됩니다. 함수가 끝난 뒤에도 그 리더를 계속 쓰려면 `&mut`로 넘깁니다. 라이터는 보통 호출한 쪽이 이어서 쓰거나 `flush`해야 하므로 `&mut impl Write`로 받는 경우가 많습니다.
- **실행 중에 고를 때.** 입력을 표준 입력과 바이트 슬라이스 중에서 실행 중에 골라야 한다면 `Box<dyn BufRead>`로 담습니다(예제 04). `Box<B>`에도 `BufRead` 구현이 있습니다. 트레이트 객체는 Part 0 ch17에서 배웠습니다.

### 2.7 버퍼링: 시스템 호출을 줄이는 방법

`File`이나 `TcpStream`의 `read`/`write`는 부를 때마다 운영체제에 **시스템 호출(system call)** 을 합니다. 시스템 호출은 프로그램과 커널 사이를 오가므로 일반 함수 호출보다 훨씬 비쌉니다. 작은 조각을 수천 번 보내는 대신 메모리에 모았다가 큰 덩어리로 보내는 것이 **버퍼링(buffering)** 입니다.

[`03_buffering.rs`](examples/03_buffering.rs)는 안쪽 `write`/`read`가 불린 횟수를 세는 래퍼(`CountingWriter`, `CountingReader`)로 이 차이를 보여 줍니다. 진짜 파일이라면 이 횟수가 곧 시스템 호출 횟수입니다.

| 작업 | 버퍼 없이 | `BufWriter`/`BufReader` |
| --- | --- | --- |
| `writeln!`으로 2000줄(16890바이트) 쓰기 | `write` 6000번 | `write` 3번 |
| 10000바이트를 `bytes()`로 한 바이트씩 읽기 | `read` 10001번 | `read` 3번 |

`writeln!(w, "줄 {i}")` 한 번이 `write`를 세 번 부르는 것에 주목하세요. 서식 문자열의 조각(`"줄 "`, 숫자, `"\n"`)마다 따로 씁니다. 버퍼 없는 리더에 `bytes()`를 부르는 코드는 clippy가 `unbuffered_bytes`로 경고합니다.

`BufReader::new`/`BufWriter::new`의 기본 버퍼 크기는 문서에 "현재 8 KiB, 바뀔 수 있음"으로 적혀 있고, `with_capacity`로 정할 수 있습니다. `BufWriter` 문서는 버퍼링이 **도움이 되지 않는** 경우도 적어 둡니다. 큰 덩어리를 한 번에 쓰거나 몇 번만 쓸 때, 그리고 `Vec<u8>`처럼 메모리에 쓸 때입니다.

**`BufWriter`는 `flush`하고 버립니다.** `BufWriter`가 버려질 때(drop) 남은 내용을 내보내려 시도하지만, 그때 난 에러는 **버려집니다**. drop에는 에러를 돌려줄 방법이 없기 때문입니다. 예제 03의 3번 구역은 8바이트짜리 `&mut [u8]`에 15바이트를 씁니다. `write_all`은 `BufWriter`의 버퍼에 담기기만 하므로 성공하고, flush 없이 버리면 앞 8바이트만 써진 채 아무 에러도 보이지 않습니다. 명시적으로 `flush()`를 부르면 `WriteZero: failed to write the buffered data`를 받습니다. 문서도 "버리기 전에 `flush`를 부르는 것이 매우 중요하다"고 강조합니다. 감싼 라이터를 되찾는 `into_inner()`도 flush를 하며, 실패하면 `Err`를 돌려줍니다.

`std::process::exit`으로 프로그램을 끝내면 소멸자가 실행되지 않아 `BufWriter`에 남은 내용은 아예 사라집니다(`ch10-env-process-time`).

**`LineWriter`와 줄 버퍼링.** `LineWriter`는 줄바꿈(`\n`)을 만날 때마다 그때까지의 내용을 내보내는 라이터입니다. 예제 03의 4번 구역에서 세 줄을 쓰면 `write` 3번, 줄바꿈 없이 쓴 조각은 `flush`할 때까지 남습니다. 표준 출력이 이 방식(줄 버퍼링)입니다.

### 2.8 표준 입출력

| 함수 | 핸들 | 버퍼 | 잠근 핸들 |
| --- | --- | --- | --- |
| `io::stdin()` | `Stdin` (`Read`, `read_line`, `lines`) | 있음 | `StdinLock` (`Read` + `BufRead`) |
| `io::stdout()` | `Stdout` (`Write`) | 줄 버퍼링 | `StdoutLock` (`Write`) |
| `io::stderr()` | `Stderr` (`Write`) | 없음 | `StderrLock` (`Write`) |

**핸들은 공유되고, 잠금이 있습니다.** `io::stdout()`이 돌려주는 핸들은 모두 프로세스 전체가 함께 쓰는 하나의 버퍼를 가리키고, 쓸 때마다 뮤텍스를 잠급니다. 그래서 `println!`을 1만 번 부르면 잠금도 1만 번입니다. 많이 출력할 때는 `lock()`으로 한 번 잠그고, `BufWriter`로 감쌉니다. 예제 03의 5번 구역에서 세 줄을 `StdoutLock`에 직접 쓰면 `write` 9번, `BufWriter`를 거치면 1번입니다. 잠금을 쥐고 있는 동안에는 다른 스레드가 표준 출력에 쓸 수 없으므로 다 쓴 잠금은 일찍 버립니다(`drop`). 같은 스레드에서 잠금을 쥔 채 `println!`을 부르는 것은 이 툴체인에서 멈추지 않고 출력됐습니다. 여러 스레드가 같은 출력에 쓰는 문제는 `ch11-threads-sync`에서 다룹니다.

**줄 버퍼링과 `flush`.** 문서는 표준 출력이 "터미널에 연결되면 줄 버퍼링"이라고 설명합니다. 줄바꿈 없이 `print!`한 내용은 버퍼에 남을 수 있으므로, 입력을 받기 전에 안내 문구를 보여 주려면 직접 `flush`합니다. 이 툴체인(macOS)에서는 파이프나 파일로 보낼 때도 줄 단위로 비웠지만, 문서가 약속한 것은 터미널의 경우뿐이므로 거기에 기대지 않습니다. 표준 에러는 버퍼가 없어 바로 나갑니다.

```rust
print!("이름을 입력하세요: ");
io::stdout().flush()?; // 줄바꿈이 없으니 직접 비웁니다.
io::stdin().read_line(&mut name)?;
```

**표준 입력 읽는 방법.**

| 하고 싶은 일 | 코드 |
| --- | --- |
| 한 줄 | `io::stdin().read_line(&mut line)?` (0이면 EOF) |
| 줄마다 | `for line in io::stdin().lines()` (1.62) 또는 `io::stdin().lock().lines()` |
| 전부 한 번에 | `io::read_to_string(io::stdin())?` (1.65) |
| 바이트로 전부 | `io::stdin().lock().read_to_end(&mut bytes)?` |

파이프로 들어온 입력이 끝나면 `lines()`가 `None`을 내고 반복이 끝납니다. Part 0 ch2의 숫자 맞히기 게임처럼 `read_line`의 반환값 0을 무시하면 끝없이 빈 문자열을 읽게 되므로 주의합니다. 터미널에서 입력을 끝내려면 macOS와 Linux에서는 Ctrl-D를 누릅니다.

**결과는 표준 출력, 나머지는 표준 에러.** Part 0 ch12에서 본 대로입니다. [`04_stdin_stdout.rs`](examples/04_stdin_stdout.rs)는 번호를 붙인 줄을 표준 출력에, 요약을 표준 에러에 씁니다. 표준 출력을 `flush`한 뒤에 요약을 쓰므로 둘을 같은 곳(`2>&1`)으로 보내도 순서가 섞이지 않습니다. 예제 04는 표준 입력이 터미널이면 사용자 입력을 기다리는 대신 내장된 예시 문장을 읽습니다. 이를 위해 쓴 `is_terminal()`은 `IsTerminal` 트레이트(1.70)의 메서드로 핸들이 터미널에 연결돼 있는지 알려 주며, 터미널 여부에 따라 동작을 바꾸는 설계는 Part 7(`ch3-output`, `ch7-humans-machines-docs`)에서 다룹니다.

**`println!`은 패닉하고, `writeln!`은 `Err`를 돌려줍니다.** `| head -n 2`처럼 출력을 받는 쪽이 먼저 끝나면, 그 뒤의 쓰기는 `BrokenPipe` 에러가 됩니다. `println!`은 쓰기 에러를 만나면 패닉합니다. 예제 03을 `| head -n 3`으로 실행하면 이렇게 끝납니다(스레드 ID는 실행마다 다름).

```text
thread 'main' (…) panicked at /rustc/…/library/std/src/io/stdio.rs:1209:9:
failed printing to stdout: Broken pipe (os error 32)
```

예제 04는 `writeln!`으로 쓰고, `main`에서 `BrokenPipe`만 골라 조용히 끝냅니다. 출력이 많은 명령줄 도구라면 이렇게 처리하는 것이 좋습니다.

### 2.9 파일: `std::fs`와 `File`

**편의 함수.** 파일 전체를 한 번에 다룰 때는 열고 닫는 일까지 해 주는 함수를 씁니다.

| 함수 | 하는 일 |
| --- | --- |
| `fs::read(p)` / `fs::read_to_string(p)` | 전체를 `Vec<u8>` / `String`으로 |
| `fs::write(p, contents)` | 없으면 만들고, 있으면 비우고 씀 (`contents: AsRef<[u8]>`이라 `&str`, `String`, `Vec<u8>` 모두 됨) |
| `fs::copy(from, to)` | 내용과 권한 비트를 복사, 복사한 바이트 수를 돌려줌 |
| `fs::rename(from, to)` | 이름 바꾸기·옮기기, `to`가 있으면 대체 |
| `fs::remove_file(p)` | 파일 삭제 |
| `fs::metadata(p)` | 메타데이터 (심볼릭 링크를 따라감. 링크 자체는 `symlink_metadata`) |
| `fs::exists(p)` | 있는지 `io::Result<bool>`로 (1.81) |

**`File`과 `OpenOptions`.** 조금씩 쓰거나 읽을 때는 `File`을 엽니다. `File::open`은 읽기 전용, `File::create`는 쓰기 전용(있으면 내용을 비움)입니다. 그 밖의 조합은 `OpenOptions`(또는 `File::options()`, 1.58)로 정합니다.

| 하고 싶은 일 | `OpenOptions` | 같은 일을 하는 함수 |
| --- | --- | --- |
| 읽기 | `.read(true)` | `File::open` |
| 새로 쓰기 (있으면 비움) | `.write(true).create(true).truncate(true)` | `File::create` |
| 끝에 덧붙이기 | `.append(true).create(true)` | |
| 없을 때만 새로 만들기 | `.write(true).create_new(true)` | `File::create_new` (1.77) |
| 내용을 유지하며 읽고 쓰기 | `.read(true).write(true)` | |

**`create_new`가 필요한 이유.** "파일이 없는지 확인하고, 없으면 만든다"를 두 단계로 하면, 그 사이에 다른 프로그램이 같은 파일을 만들 수 있습니다. 확인한 시점과 사용하는 시점 사이의 이 경쟁을 TOCTOU(time of check to time of use)라고 하며, `std::fs` 모듈 문서가 따로 절을 두어 경고합니다. `create_new`는 확인과 생성을 운영체제가 한 번에 하므로 이 틈이 없습니다. 이미 있으면 `AlreadyExists`로 실패합니다(예제 05의 3번 구역). 같은 이유로 `exists()`로 확인한 뒤 여는 것보다 그냥 열고 에러를 처리하는 편이 안전합니다.

**`File`에는 버퍼가 없습니다.** 문서도 "`File`은 읽기와 쓰기를 버퍼링하지 않는다"고 적어 둡니다. `writeln!(file, ..)`마다 시스템 호출이 일어나므로, 많이 쓸 때는 `BufWriter::new(file)`로 감쌉니다(2.7절). `File`은 버려질 때 닫히며, 닫을 때의 에러는 무시됩니다. 데이터가 디스크까지 내려갔는지 확인해야 한다면 `sync_all()`을 부릅니다.

**메타데이터.** `Metadata`의 주요 메서드입니다.

| 메서드 | 돌려주는 것 |
| --- | --- |
| `len()` | 크기(바이트) |
| `is_file()`, `is_dir()`, `is_symlink()` | 종류 |
| `modified()`, `accessed()`, `created()` | `io::Result<SystemTime>` (플랫폼에 따라 지원하지 않으면 `Err`) |
| `permissions().readonly()` | 읽기 전용인지 |

시각 값 자체(`SystemTime`)는 `ch10-env-process-time`에서 다룹니다. 유닉스 권한 비트(예: `0o644`)처럼 플랫폼에만 있는 정보는 `std::os::unix::fs::PermissionsExt` 같은 플랫폼 전용 확장 트레이트로 읽습니다. 이 챕터에서는 그런 확장이 있다는 것만 알아 둡니다.

### 2.10 디렉터리

| 함수 | 하는 일 | 이미 있거나 비어 있지 않으면 |
| --- | --- | --- |
| `fs::create_dir(p)` | 디렉터리 하나 만들기 | `AlreadyExists`, 부모가 없으면 `NotFound` |
| `fs::create_dir_all(p)` | 중간 디렉터리까지 모두 만들기 | 이미 있어도 성공 |
| `fs::read_dir(p)` | 항목을 내주는 반복자 (`io::Result<DirEntry>`) | |
| `fs::remove_dir(p)` | 빈 디렉터리 삭제 | `DirectoryNotEmpty` |
| `fs::remove_dir_all(p)` | 안의 내용까지 모두 삭제 | |

**`read_dir`의 순서는 보장되지 않습니다.** 문서는 순서가 플랫폼과 파일 시스템에 따라 다르고 호출할 때마다 바뀔 수 있다고 말합니다. 같은 결과가 필요하면 모아서 정렬합니다. `.`과 `..`은 나오지 않습니다.

```rust
let mut entries = fs::read_dir(dir)?.collect::<io::Result<Vec<_>>>()?;
entries.sort_by_key(|entry| entry.file_name());
```

`DirEntry`에서는 `path()`(디렉터리 경로에 이름을 붙인 `PathBuf`), `file_name()`(`OsString`), `file_type()`, `metadata()`를 얻습니다. `file_type()`은 심볼릭 링크를 따라가지 않으므로, 재귀 순회에서 이것으로 디렉터리인지 판단하면 디렉터리를 가리키는 링크 때문에 끝없이 도는 일이 없습니다.

**재귀 순회.** 표준 라이브러리에는 하위 디렉터리까지 내려가는 함수가 없으므로 `read_dir`을 재귀로 부르는 함수를 만듭니다([`07_directories.rs`](examples/07_directories.rs)의 `collect_files`).

```rust
fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            collect_files(&entry.path(), out)?;
        } else {
            out.push(entry.path());
        }
    }
    Ok(())
}
```

`remove_dir_all`은 되돌릴 수 없습니다. 문서도 "조심해서 쓰라(Use carefully!)"고 적어 둡니다. 경로를 계산해서 넘길 때는 엉뚱한 디렉터리가 되지 않는지 반드시 확인합니다.

### 2.11 `io::Error` 자세히

**`io::Result<T>`** 는 `Result<T, io::Error>`의 별칭입니다(Part 0 ch9). `io::Error`는 크게 두 출처에서 옵니다.

1. **운영체제가 알려 준 실패.** 운영체제는 실패를 번호(유닉스의 `errno`)로 알려 줍니다. `raw_os_error()`가 그 번호를 `Some(i32)`로, `kind()`가 번호를 플랫폼과 무관한 `ErrorKind`로 바꾼 값을 돌려줍니다. 메시지 끝의 `(os error 2)`가 그 번호입니다.
2. **Rust 쪽에서 판단한 실패.** UTF-8이 아님(`InvalidData`), 버퍼를 다 채우지 못함(`UnexpectedEof`), 우리 코드가 만든 에러 등입니다. `raw_os_error()`는 `None`입니다.

[`06_io_errors.rs`](examples/06_io_errors.rs)의 1번 구역이 실제로 일으킨 실패입니다(macOS).

| 상황 | `kind()` | OS 번호 |
| --- | --- | --- |
| 없는 파일 열기 | `NotFound` | 2 |
| 있는 파일에 `create_new` | `AlreadyExists` | 17 |
| 파일 아래 경로(`data.txt/child`) 열기 | `NotADirectory` | 20 |
| 비지 않은 디렉터리에 `remove_dir` | `DirectoryNotEmpty` | 66 (Linux는 39) |
| 디렉터리를 `read_to_string` | `IsADirectory` | 21 |
| UTF-8이 아닌 파일을 `read_to_string` | `InvalidData` | 없음 |
| 4바이트 파일에 `read_exact(8)` | `UnexpectedEof` | 없음 |

번호는 운영체제마다 다르므로 프로그램의 판단은 `kind()`로 합니다. `ErrorKind`는 배리언트가 늘어날 수 있는 열거형이라 `match`에 `_` 갈래가 필요합니다(Part 0 ch9). 같은 실수가 플랫폼에 따라 다른 종류가 되기도 합니다. 예를 들어 디렉터리에 `remove_file`을 부르면 macOS에서는 `PermissionDenied`(`Operation not permitted (os error 1)`)가 나옵니다.

**직접 만들기.**

| 방법 | 쓰는 때 | 예제 02의 결과 |
| --- | --- | --- |
| `io::Error::new(kind, 내용)` | 종류를 정해 만들 때. 내용은 `&str`, `String`, 다른 에러 | `InvalidData: invalid digit found in string` |
| `io::Error::other(내용)` (1.74) | 종류가 `Other`일 때의 줄임 | `Other: 설정 형식이 잘못됐습니다` |
| `io::Error::from(kind)` | 메시지 없이 종류만 (할당 없음) | `NotFound: entity not found` |
| `io::Error::from_raw_os_error(2)` | 운영체제 에러 번호에서 | `NotFound: No such file or directory (os error 2)` |

`new`와 `other`의 내용 타입은 `E: Into<Box<dyn Error + Send + Sync>>`입니다. Part 1 ch8에서 본 `Box<dyn Error>`로의 변환이 그대로 쓰이므로, 문자열이나 에러 값을 그대로 넘길 수 있습니다. 넘긴 에러는 `get_ref()`로 빌려 보거나, `into_inner()`로 꺼내거나, `downcast::<E>()`(1.79)로 원래 타입을 되찾을 수 있습니다. 반환 타입을 `io::Result` 하나로 유지하고 싶은 함수, 예를 들어 `Read`나 `Write`를 직접 구현하는 함수 안에서 이 방법을 씁니다. 이렇게 만든 `io::Error`는 `Display`에 안쪽 에러의 메시지를 그대로 보여 주는 대신 `source()`로는 그 에러를 내주지 않습니다(ch8의 "`source`나 `Display` 중 하나만" 원칙).

### 2.12 "어느 파일에서?": 경로를 담아 감싸기

`io::Error`의 메시지에는 **경로가 없습니다.** 설정 파일, 데이터 파일, 로그 파일을 차례로 여는 프로그램이 `?`로 에러를 올려 보내기만 하면 사용자는 `No such file or directory (os error 2)`만 보고, 어느 파일이 없는지 알 수 없습니다. Part 0 ch12의 minigrep도 그랬습니다. 경로는 fs 함수를 부르는 자리에서만 알 수 있으므로, 거기서 감쌉니다.

```rust
#[derive(Debug)]
struct FileError {
    action: &'static str, // "읽지", "만들지" 같은 동작
    path: PathBuf,
    source: io::Error,
}
// Display: "파일을 {action} 못했습니다: {path.display()}"
// Error::source: Some(&self.source)

fn read_text(path: &Path) -> Result<String, FileError> {
    fs::read_to_string(path).map_err(|source| FileError {
        action: "읽지",
        path: path.to_path_buf(),
        source,
    })
}
```

이것은 ch8 2.3절의 설계 원칙(이 층의 실패는 `Display`로, 원인은 `source()`로)을 파일에 적용한 것입니다. 보고서는 이렇게 나옵니다(경로는 환경마다 다름).

```text
    에러: 파일을 읽지 못했습니다: <임시 디렉터리>/p1-ch9-06-<프로세스 ID>/app.conf
      원인: No such file or directory (os error 2)
```

호출한 쪽은 `wrapped.source.kind() == ErrorKind::NotFound`처럼 원인의 종류도 그대로 물어볼 수 있습니다. 같은 원인 타입(`io::Error`)이 여러 파일에서 오고 위치가 중요하므로 `From` 대신 `map_err`를 쓴다는 점도 ch8 2.4절의 기준과 같습니다.

반환 타입을 `io::Error`로 유지해야 한다면 `io::Error::new(e.kind(), format!("{}: {e}", path.display()))`처럼 메시지에 경로를 붙인 새 에러를 만드는 방법도 있습니다. 종류는 남지만 `raw_os_error()`는 `None`이 되고 원래 에러는 문자열로만 남습니다(예제 06의 3번 구역 (다)). 라이브러리라면 앞의 방법이 낫습니다.

### 2.13 `Path`와 `PathBuf`

**`str`과 `String`의 관계와 같습니다.** `Path`는 크기를 모르는 빌린 경로(`&Path`로 씀), `PathBuf`는 소유한 경로입니다. `&PathBuf`는 역참조 강제 변환으로 `&Path`가 되므로 함수는 `&Path`로 받습니다. 두 타입은 문자열 대신 운영체제의 문자열 타입(`OsStr`/`OsString`) 위에 만들어져 있습니다. 운영체제의 경로는 UTF-8이 아닐 수 있기 때문이며, `OsStr` 자체는 `ch14-ffi-any-misc`에서 다룹니다.

| 하고 싶은 일 | `&Path`의 메서드 | `PathBuf`를 고치는 메서드 |
| --- | --- | --- |
| 만들기 | `Path::new("a/b")` (할당 없음) | `PathBuf::from("a/b")`, `to_path_buf()` |
| 붙이기 | `join(p)` → 새 `PathBuf` | `push(p)` |
| 마지막 구성 요소 떼기 | `parent()` → `Option<&Path>` | `pop()` |
| 이름 | `file_name()`, `file_stem()`, `extension()` → `Option<&OsStr>` | `set_file_name()` |
| 확장자 바꾸기 | `with_extension("html")` | `set_extension("html")` |
| 앞부분 떼기 | `strip_prefix(base)` → `Result<&Path, _>` | |
| 구성 요소 | `components()`, `iter()`, `ancestors()` | |
| 출력 | `display()`, `to_str()` → `Option<&str>`, `to_string_lossy()` | |

[`08_paths.rs`](examples/08_paths.rs)에서 확인한 함정은 다음과 같습니다.

- **절대 경로를 `join`/`push`하면 앞의 경로가 통째로 바뀝니다.** `Path::new("/srv/app").join("/etc/hosts")`는 `"/etc/hosts"`입니다. 사용자 입력을 붙일 때 특히 조심해야 하고, clippy가 `join_absolute_paths`로 경고합니다.
- `file_stem`은 **마지막** 점 앞, `extension`은 마지막 점 뒤입니다. `report.tar.gz`의 확장자는 `gz`, 줄기는 `report.tar`입니다. 점으로 시작하는 `.bashrc`는 확장자가 없습니다.
- `parent()`는 구성 요소가 하나뿐인 상대 경로(`notes/`)에서 `Some("")`를, 루트(`/`)에서 `None`을 돌려줍니다.

**문자열이 아니라 구성 요소 단위입니다.** 경로를 `/`로 나눈 조각 하나하나를 구성 요소(component)라고 합니다. `components()`는 반복되는 구분자, 중간의 `.`, 끝의 `/`를 무시하지만 `..`은 그대로 둡니다. `b`가 심볼릭 링크라면 `a/b/../c`가 `a/c`와 다른 곳일 수 있기 때문입니다.

| 식 | 결과 | 문자열로 했다면 |
| --- | --- | --- |
| `Path::new("a//b/") == Path::new("a/b")` | `true` | `false` |
| `Path::new("/etc/passwd").starts_with("/e")` | `false` (구성 요소 `etc` ≠ `e`) | `true` |
| `Path::new("/etc/resolv.conf").ends_with("conf")` | `false` (확장자는 `extension()`으로) | `true` |
| `"/usr//local/./bin/../lib/"`의 구성 요소 | `RootDir`, `usr`, `local`, `bin`, `ParentDir`, `lib` | |

그래서 경로를 `format!("{dir}/{name}")`이나 `+`로 이어 붙이지 않고 `join`을 씁니다. 구분자가 운영체제마다 다르다는 점(유닉스 `/`, Windows `\`)도 `join`이 처리해 줍니다.

**`display()`가 필요한 이유.** `Path`와 `OsStr`은 UTF-8이 아닐 수 있어서 `Display`를 구현하지 않습니다. `println!("{}", path)`는 E0277이고(4.3절), `path.display()`가 돌려주는 값은 UTF-8이 아닌 부분을 대체 문자(U+FFFD)로 바꿔 출력합니다. `{:?}`는 따옴표와 이스케이프가 붙는 디버그 형식으로 바로 쓸 수 있습니다. `&str`이 꼭 필요하면 실패할 수 있는 `to_str()`(`Option`)이나 손실이 있는 `to_string_lossy()`를 씁니다. `OsStr`에도 `display()`(1.87)가 있어 `entry.file_name().display()`처럼 씁니다.

**`AsRef<Path>`: ch6의 약속.** `std::fs`의 함수는 모두 이런 모양입니다.

```rust
pub fn read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String>
```

`str`, `String`, `OsStr`, `OsString`, `Path`, `PathBuf`, `Cow<'_, OsStr>`, `Component` 등이 `AsRef<Path>`를 구현하고(1.100.0 문서의 `AsRef` 구현 목록), `&T`에 대한 포괄 구현 덕분에 그 참조들도 됩니다. 그래서 호출하는 쪽은 가진 것을 그대로 넘기고, 함수 안에서는 `path.as_ref()`로 `&Path`를 얻습니다. ch6 2.5절에서 "`AsRef`는 반사적이지 않다"고 했지만, `Path`는 `impl AsRef<Path> for Path`를 따로 두어 이 빈틈을 메웁니다. 예제 08의 `kind_of(path: impl AsRef<Path>)`가 `&str`, `&String`, `String`, `&PathBuf`, `&Path`, `PathBuf`를 모두 받습니다. `String`이나 `PathBuf`를 값으로 넘기면 이동하므로, 뒤에서 또 쓸 값은 `&`로 넘깁니다.

**경로 연산은 파일 시스템을 보지 않습니다.** `join`, `parent`, `extension`, `components`, `starts_with`는 문자열만 보고 계산하므로 없는 경로에도 동작합니다. 파일 시스템에 묻는 것은 다음 메서드들입니다.

| 메서드 | 하는 일 | 없는 경로에서 |
| --- | --- | --- |
| `exists()` | 있는지 `bool`로. 권한 문제 등 모든 에러를 `false`로 바꿈 | `false` |
| `try_exists()` (1.63), `fs::exists()` (1.81) | 있는지 확인할 수 없으면 `Err` | `Ok(false)` |
| `is_file()`, `is_dir()`, `metadata()` | 종류, 메타데이터 | `false`, `Err` |
| `canonicalize()` | `..`와 심볼릭 링크를 풀어 절대 경로로 | `Err(NotFound)` |

예제 08의 7번 구역에서 `sub/../data.txt`는 구성 요소에 `..`이 남아 있지만 `exists()`는 `true`입니다(운영체제가 따라가 줌). `canonicalize()` 뒤에는 `..`이 사라집니다. macOS의 임시 디렉터리는 `/var/…` 아래에 있는데 `/var`가 `/private/var`를 가리키는 심볼릭 링크라서, `canonicalize` 결과가 `temp_dir()`이 준 경로와 달라집니다. 경로를 비교할 때 두 쪽 모두 `canonicalize`해야 하는 이유입니다. 상대 경로를 현재 디렉터리 기준 절대 경로로 바꾸는 `std::path::absolute`(1.79)도 있지만 현재 디렉터리는 `ch10-env-process-time`에서 다룹니다.

## 3. 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`examples/01_read_write_traits.rs`](examples/01_read_write_traits.rs) | `read`의 부분 읽기, `read_exact`/`read_to_end`/`read_to_string`, `write`와 `write_all`, 두 `Write`와 `write!`, `Cursor`와 `Seek` | 1 |
| [`examples/02_bufread_lines.rs`](examples/02_bufread_lines.rs) | `lines`/`read_line`/`split`/`read_until`, UTF-8이 아닌 줄, `impl BufRead` 함수와 테스트 | 1 |
| [`examples/03_buffering.rs`](examples/03_buffering.rs) | 호출 횟수로 본 `BufWriter`/`BufReader`, flush 없이 버린 `BufWriter`, `LineWriter`, `stdout().lock()` | 2 |
| [`examples/04_stdin_stdout.rs`](examples/04_stdin_stdout.rs) | 표준 입력 줄 단위 읽기, 표준 출력·표준 에러 나누기, `BrokenPipe` 처리 | 2, 3 |
| [`examples/05_files.rs`](examples/05_files.rs) | 편의 함수, `File`/`OpenOptions`(덧붙이기, `create_new`), 파일 위치, 메타데이터, 복사·이름 바꾸기·삭제 | 4 |
| [`examples/06_io_errors.rs`](examples/06_io_errors.rs) | `kind()`와 OS 에러 번호, `io::Error` 만들기, 경로를 담아 감싸기 | 4 |
| [`examples/07_directories.rs`](examples/07_directories.rs) | `create_dir_all`, 정렬한 `read_dir`, 재귀 순회, 확장자로 거르기, `remove_dir`/`remove_dir_all` | 4 |
| [`examples/08_paths.rs`](examples/08_paths.rs) | `Path`/`PathBuf` 나누기·조립, 구성 요소 단위 비교, `display()`, `AsRef<Path>`, `exists`/`canonicalize` | 5 |
| [`compile_fail/`](compile_fail) | 입출력·경로에서 자주 만나는 컴파일 에러 5개 (4절) | 1, 3, 5 |

모든 명령은 저장소 루트에서 실행합니다. 모든 예제는 종료 코드 0으로 끝나며 패닉하는 예제는 없습니다. 파일을 쓰는 예제(02, 05~08)는 `std::env::temp_dir()` 아래에 `p1-ch9-<예제 번호>-<프로세스 ID>` 디렉터리를 만들고 끝날 때 지웁니다. 저장소 안에는 파일을 만들지 않습니다.

### 01_read_write_traits — 네 트레이트의 기본 동작

```sh
cargo run -p p1-ch9-io-fs-path --example 01_read_write_traits
```

```text
1. read는 요청보다 적게 줄 수 있고, 0이면 EOF입니다
    read(8바이트 버퍼) → 5바이트 "hello"
    read(8바이트 버퍼) → 5바이트 ", wor"
    read(8바이트 버퍼) → 3바이트 "ld!"
    read(8바이트 버퍼) → 0바이트 ""
2. read_exact / read_to_end / read_to_string은 read를 반복해 부릅니다
    read_exact(8) → "01234567", read 호출 3번
    read_exact(4) → 에러 UnexpectedEof: failed to fill whole buffer
    read_to_end → 9바이트 [234, 176, 128, 235, 130, 152, 235, 139, 164], read 호출 6번
    read_to_string → "앞: 가나다"
3. write도 일부만 쓸 수 있고, write_all이 끝까지 반복합니다
    write(12바이트) → Ok(4), 지금까지 "hell"
    write_all 뒤 → "hell / write_all", write 호출 4번
4. write!는 io::Write와 fmt::Write에 모두 쓰입니다
    Vec<u8>  ← [50, 43, 51, 61, 53, 10, 235, 129, 157]
    String::from_utf8 → "2+3=5\n끝"
    String   ← "2+3=5\n"
    [u8; 6]  ← "123456", 결과 Err(Error { kind: WriteZero, message: "failed to write whole buffer" })
5. Seek: Cursor로 읽기·쓰기 위치를 옮깁니다
    10바이트 쓴 뒤 위치 = 10
    Start(2)에서 3바이트 → "234", 위치 = 5
    Current(2) → 위치 = 7
    End(-2) → 위치 = 8, 끝까지 읽으면 "89"
    rewind 뒤 "AB"를 쓰면 → "AB23456789"
```

2번 구역의 `read_exact(8)`은 3바이트씩 내주는 리더에서 `read`를 세 번(3+3+2) 불렀습니다. `read_to_end`의 6번은 데이터가 든 다섯 번과 EOF를 확인한 한 번입니다. 4번 구역의 마지막 줄은 `io::Write`가 실패할 수 있다는 것을 보여 줍니다. 6바이트짜리 버퍼에 9자리 숫자를 쓰면 앞 6바이트만 들어가고 `WriteZero` 에러가 납니다.

### 02_bufread_lines — 줄 단위로 읽기

```sh
cargo run -p p1-ch9-io-fs-path --example 02_bufread_lines
cargo test -p p1-ch9-io-fs-path --example 02_bufread_lines
```

```text
1. lines()는 줄 끝의 \n과 \r\n을 떼어 냅니다
    "첫째 줄"
    "둘째 줄"
    "셋째 줄"
2. read_line()은 줄 끝 문자까지 덧붙이고, 0을 돌려주면 EOF입니다
    11바이트 "첫째 줄\n"
    12바이트 "둘째 줄\r\n"
    10바이트 "셋째 줄"
     0바이트 ""
3. split()과 read_until()은 다른 바이트로 나눕니다
    split(b';') → ["a", "bb", "", "ccc"]
    read_until(b'=') → 4바이트 "key="
4. UTF-8이 아닌 줄이 있으면 lines()는 그 줄에서 에러를 냅니다
    Ok("ok")
    Err(InvalidData: stream did not contain valid UTF-8)
    Ok("after")
    바이트로 읽고 from_utf8_lossy → ["ok", "��", "after"]
5. impl BufRead를 받는 함수는 어디서 읽든 똑같이 동작합니다
    &[u8]           → Some((2, "들판을 건너"))
    Cursor<Vec<u8>> → Some((2, "들판을 건너"))
    BufReader<File> → Some((2, "들판을 건너"))
    io::empty()     → None
```

4번 구역에서 에러가 난 뒤에도 `lines()`는 다음 줄 `"after"`를 계속 내줍니다. 테스트 명령은 `finds_first_longest_line`과 `reports_invalid_utf8` 두 테스트를 실행하고 `test result: ok. 2 passed`로 끝납니다. 두 테스트 모두 파일 없이 바이트 슬라이스만 씁니다.

### 03_buffering — 호출 횟수로 본 버퍼링

```sh
cargo run -p p1-ch9-io-fs-path --example 03_buffering
```

```text
1. 2000줄을 쓸 때 안쪽 write가 불린 횟수
    버퍼 없이  : 16890바이트, write 6000번
    BufWriter  : 16890바이트, write 3번
2. 10000바이트를 한 바이트씩 읽을 때 안쪽 read가 불린 횟수
    버퍼 없이  : 10000바이트, read 10001번 (마지막 1번은 EOF 확인)
    BufReader  : 10000바이트, read 3번
    BufReader  : 5000줄 lines(), read 4번
3. BufWriter는 flush해야 쓰기 에러를 알 수 있습니다
    write_all 뒤 BufWriter에 대기 중: 15바이트
    flush 없이 버림 → 결과 "hello, b" (에러를 알 방법이 없음)
    flush → 에러 WriteZero: failed to write the buffered data
4. LineWriter는 줄바꿈을 만날 때마다 안쪽으로 내보냅니다
    3줄 쓴 뒤      : write 3번
    줄바꿈 없이 쓴 뒤: write 3번
    flush 뒤       : write 4번
5. 표준 출력을 한 번 잠그고(lock) BufWriter로 감싸기
    (잠근 표준 출력) 1번째 줄
    (잠근 표준 출력) 2번째 줄
    (잠근 표준 출력) 3번째 줄
    (BufWriter + 잠근 표준 출력) 1번째 줄
    (BufWriter + 잠근 표준 출력) 2번째 줄
    (BufWriter + 잠근 표준 출력) 3번째 줄
    StdoutLock에 직접: write 9번, BufWriter를 거치면: write 1번
```

호출 횟수는 기본 버퍼 크기(현재 8 KiB)와 이 툴체인의 구현에 따른 값이라 툴체인이 바뀌면 달라질 수 있습니다. 16890바이트를 8 KiB 버퍼로 쓰면 버퍼가 두 번 차고 마지막 `flush`에서 한 번 더 써서 3번입니다. 래퍼 타입에서 횟수를 꺼낼 때는 감싼 값을 빌려 주는 `get_ref()`를 썼습니다.

### 04_stdin_stdout — 표준 입력과 표준 출력

파이프로 입력을 넣어 실행합니다.

```sh
printf '하나 둘\n\n셋 넷 다섯\n' | cargo run -q -p p1-ch9-io-fs-path --example 04_stdin_stdout
```

```text
   1: "하나 둘"
   2: ""
   3: "셋 넷 다섯"
[요약] 3줄 (빈 줄 1개), 단어 5개
```

마지막 줄은 표준 에러입니다. 결과만 파일로 보내면 요약은 화면에 남고 파일에는 번호 붙은 줄만 들어갑니다. 확인한 뒤 `out.txt`는 지웁니다.

```console
$ printf '하나 둘\n\n셋 넷 다섯\n' | cargo run -q -p p1-ch9-io-fs-path --example 04_stdin_stdout > out.txt
[요약] 3줄 (빈 줄 1개), 단어 5개
$ cat out.txt
   1: "하나 둘"
   2: ""
   3: "셋 넷 다섯"
$ rm out.txt
```

출력을 받는 쪽이 먼저 끝나면 `BrokenPipe`를 조용히 처리합니다.

```console
$ yes '가 나 다' | head -n 100000 | cargo run -q -p p1-ch9-io-fs-path --example 04_stdin_stdout | head -n 2
   1: "가 나 다"
   2: "가 나 다"
[안내] 출력을 받는 쪽이 먼저 끝나서 멈춥니다
```

입력이 비어 있으면(`< /dev/null`) `[요약] 0줄 (빈 줄 0개), 단어 0개`로 끝납니다. 파이프 없이 터미널에서 실행하면 예시 문장 세 줄(`"표준 입력이 터미널이라 예시 문장을 읽습니다"`, `""`, `"파이프로 입력을 넣어 보세요"`)을 읽고 `[요약] 3줄 (빈 줄 1개), 단어 10개`를 출력합니다. 프로그램은 시작할 때 `number_lines`를 `&[u8]`과 `Vec<u8>`으로 먼저 시험(`assert_eq!`)해 봅니다.

### 05_files — 파일 다루기

```sh
cargo run -p p1-ch9-io-fs-path --example 05_files
```

```text
1. 편의 함수: 파일 전체를 한 번에 쓰고 읽습니다
    read_to_string → "첫 줄\n둘째 줄\n"
    read           → 19바이트
2. File::create로 조금씩 쓰고, File::open + BufReader로 한 줄씩 읽습니다
    민수 82
    서연 95
    지훈 77
    합계 254
3. OpenOptions: 덧붙이기와 새로 만들기 전용
    [log] 시작
    [log] 작업 1 완료
    [log] 종료
    create_new → AlreadyExists: File exists (os error 17)
4. 열린 File의 읽기 위치는 하나뿐입니다
    첫 번째 read_to_string → 19바이트
    두 번째 read_to_string → 0바이트
    rewind 뒤              → 19바이트
5. 메타데이터
    scores.txt: 30바이트, 파일인가 true, 디렉터리인가 false
    예제 디렉터리: 디렉터리인가 true
    수정 시각을 얻었나 true, 읽기 전용인가 false
6. 복사, 이름 바꾸기, 삭제
    copy   scores.txt → scores.bak: 30바이트
    rename scores.txt → scores-2024.txt: 원래 이름 있나 false, 새 이름 있나 true
    remove_file 뒤 metadata → NotFound
```

3번 구역은 로그를 쓸 때마다 파일을 `append` 모드로 다시 열지만 내용이 쌓입니다. 수정 시각은 실행마다 다르므로 값 대신 얻을 수 있었는지만 출력했습니다.

### 06_io_errors — `io::Error`와 경로 담기

```sh
cargo run -p p1-ch9-io-fs-path --example 06_io_errors
```

```text
1. 실패를 일으켜 kind()와 raw_os_error()를 봅니다
    없는 파일 열기 → NotFound, OS 번호 2, "No such file or directory (os error 2)"
    있는 파일 create_new → AlreadyExists, OS 번호 17, "File exists (os error 17)"
    파일 아래 경로 열기 → NotADirectory, OS 번호 20, "Not a directory (os error 20)"
    비지 않은 디렉터리 삭제 → DirectoryNotEmpty, OS 번호 66, "Directory not empty (os error 66)"
    디렉터리를 파일로 읽기 → IsADirectory, OS 번호 21, "Is a directory (os error 21)"
    UTF-8 아닌 파일 읽기 → InvalidData, OS 번호 없음, "stream did not contain valid UTF-8"
    4바이트 파일에 read_exact(8) → UnexpectedEof, OS 번호 없음, "failed to fill whole buffer"
2. io::Error를 직접 만듭니다
    "8080\n" → 포트 8080
    "80x\n" → InvalidData: invalid digit found in string
    "" → UnexpectedEof: 포트 줄이 없습니다
    get_ref() → Some(ParseIntError { kind: InvalidDigit })
    raw_os_error() → None
    downcast::<ParseIntError>() → Ok(ParseIntError { kind: InvalidDigit })
    other(..)              → Other: 설정 형식이 잘못됐습니다
    from(ErrorKind)        → NotFound: entity not found
    from_raw_os_error(2)   → NotFound: No such file or directory (os error 2)
3. 어느 파일에서 실패했나요?
  (가) io::Error 그대로
    에러: No such file or directory (os error 2)
  (나) FileError로 감싸기
    에러: 파일을 읽지 못했습니다: <임시 디렉터리>/p1-ch9-06-<프로세스 ID>/app.conf
      원인: No such file or directory (os error 2)
    원인의 kind() = NotFound, 파일이 없는 경우인가 true
  (다) 메시지에 경로를 붙인 새 io::Error
    에러: <임시 디렉터리>/p1-ch9-06-<프로세스 ID>/sub: Is a directory (os error 21)
    kind() = IsADirectory, raw_os_error() = None
```

`<임시 디렉터리>`와 `<프로세스 ID>` 자리에는 실행한 환경의 실제 값이 나옵니다. 메시지와 번호는 macOS 기준이며 Linux에서는 일부 번호(예: 66 → 39)가 다릅니다. 1번 구역에서 `File::open(디렉터리)`가 아니라 `read_to_string`으로 시험한 이유는, 유닉스에서는 디렉터리를 여는 것까지는 성공하고 읽을 때 실패하기 때문입니다.

### 07_directories — 디렉터리 순회

```sh
cargo run -p p1-ch9-io-fs-path --example 07_directories
```

```text
1. 디렉터리 만들기
    create_dir_all(이미 있음)   → Ok(())
    create_dir(이미 있음)       → AlreadyExists
    create_dir(부모가 없음)     → NotFound
2. read_dir의 항목을 이름순으로 정렬해 봅니다
    Cargo.toml   파일
    README.md    파일
    src          디렉터리
    target       디렉터리
    tests        디렉터리
3. 재귀로 순회하며 트리를 출력합니다
    project/
      Cargo.toml (24바이트)
      README.md (7바이트)
      src/
        lib.rs (14바이트)
        main.rs (13바이트)
        util/
          mod.rs (14바이트)
          text.rs (15바이트)
      target/
      tests/
        basic.rs (19바이트)
4. .rs 파일만 골라 project 기준 상대 경로로 정리합니다
    src/lib.rs
    src/main.rs
    src/util/mod.rs
    src/util/text.rs
    tests/basic.rs
    전체 파일 7개 중 .rs 파일 5개, 합계 75바이트
5. 디렉터리 지우기
    remove_dir(내용 있음)  → DirectoryNotEmpty
    remove_dir(빈 target)  → Ok(())
    remove_dir_all 뒤 예제 디렉터리가 있나 false
```

정렬하지 않은 `read_dir`의 순서는 실행·파일 시스템마다 다를 수 있어서 출력하지 않았습니다. 4번 구역은 `collect_files`가 모은 절대 경로에서 `strip_prefix`로 `project` 부분을 떼어 냈고, 결과를 정렬했습니다. `"Cargo.toml"`이 `"README.md"`보다 앞인 것은 바이트 순서(대문자 `C` < `R`)로 정렬했기 때문입니다.

### 08_paths — 경로 다루기

```sh
cargo run -p p1-ch9-io-fs-path --example 08_paths
```

1·2·4·6번 구역은 2.13절의 표와 같은 내용이라 생략하고, 3·5·7번 구역만 옮깁니다. 전체 출력은 예제 파일의 `// 출력:` 주석에 있습니다.

```text
3. 경로 조립
    join("logs/today.log") → "/srv/app/logs/today.log"
    join("/etc/hosts")     → "/etc/hosts"
    push 두 번             → "/srv/app/config/app.toml"
    pop                    → "/srv/app/config"
    with_extension("html") → "out/report.html"
    with_file_name("a.md") → "out/a.md"
    ["usr","local","bin"] → "usr/local/bin"
5. 출력과 문자열 변환
    display() → 문서/회의 메모.txt
    {:?}      → "문서/회의 메모.txt"
    to_str()  → Some("문서/회의 메모.txt")
    to_string_lossy() → "문서/회의 메모.txt"
7. 경로 연산은 파일 시스템을 보지 않습니다
    없는 경로도 나눌 수 있음: extension=Some("txt"), parent=Some("no-such-dir")
    exists() = false, fs::exists() = Ok(false)
    canonicalize → 에러 NotFound
    sub/../data.txt: 구성 요소에 ".."가 남아 있나 true, exists() = true
    canonicalize 뒤: ".."가 남아 있나 false, 디렉터리 기준 상대 경로 = "data.txt"
    temp_dir() 쪽 경로와 canonicalize 결과가 같은가 false
```

5번 구역의 경로는 UTF-8이라 네 방법의 결과가 같습니다. UTF-8이 아닌 파일 이름(유닉스에서는 가능)이었다면 `to_str()`은 `None`, `display()`와 `to_string_lossy()`는 고칠 수 없는 바이트를 `�`로 바꿔 보여 줍니다. 7번 구역의 마지막 줄은 macOS 결과이고, 임시 디렉터리가 심볼릭 링크를 거치지 않는 환경에서는 `true`일 수 있습니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다. 고쳐서 컴파일에 성공했을 때 실행 파일이 챕터 디렉터리에 생기지 않도록 `--out-dir`로 임시 디렉터리를 지정합니다.

```sh
cd part1-standard-lib/ch9-io-fs-path
rustc --edition 2024 compile_fail/e0599_lines_on_file.rs --out-dir /tmp/rust-p1-ch9
```

에러 메시지 중간에 표준 라이브러리 소스 위치(`/rustc/0ef1bc38…/library/…`)가 나오는 경우가 있습니다. 경로의 해시 값은 툴체인마다 다르며, 아래 인용에서는 그 줄을 생략했습니다.

### 4.1 파일에 쓰는데 `fmt::Write`를 가져옴 — E0599

파일: [`compile_fail/e0599_writeln_file_with_fmt_write.rs`](compile_fail/e0599_writeln_file_with_fmt_write.rs)

```text
error[E0599]: cannot write into `File`
  --> compile_fail/e0599_writeln_file_with_fmt_write.rs:17:14
   |
17 |     writeln!(file, "점수: {}", 95)?;
   |              ^^^^
   |
note: must implement `io::Write`, `fmt::Write`, or have a `write_fmt` method
   = help: items from traits can only be used if the trait is in scope
help: trait `Write` which provides `write_fmt` is implemented but not in scope; perhaps you want to import it
   |
10 + use std::io::Write;
```

- **읽는 법**: `writeln!`의 첫 인자는 `write_fmt` 메서드를 가져야 하는데(`note:`), `File`에 그 메서드를 주는 트레이트(`std::io::Write`)가 스코프에 없다는 뜻입니다.
- **원인**: ch3에서 `String`에 쓰려고 `use std::fmt::Write;`를 쓰던 습관대로 가져왔습니다. `fmt::Write`는 문자열을 받는 곳의 트레이트라 `File`은 구현하지 않습니다(2.3절).
- **해결**: `use std::io::Write;`로 바꿉니다. 한 파일에서 둘 다 필요하면 한쪽을 `use std::fmt::Write as _;`로 가져옵니다(예제 01). `use std::io::Write`를 아예 빠뜨리고 `stdout().write_all(..)`을 부를 때도 같은 E0599(``no method named `write_all` found``)가 나고 해결도 같습니다.

### 4.2 `File`에 바로 `lines()` — E0599

파일: [`compile_fail/e0599_lines_on_file.rs`](compile_fail/e0599_lines_on_file.rs)

```text
error[E0599]: the method `lines` exists for struct `File`, but its trait bounds were not satisfied
  --> compile_fail/e0599_lines_on_file.rs:14:22
   |
14 |     for line in file.lines() {
   |                      ^^^^^ method cannot be called on `File` due to unsatisfied trait bounds
   |
   = note: the following trait bounds were not satisfied:
           `File: BufRead`
           which is required by `&mut File: BufRead`
```

- **읽는 법**: 4.1절과 달리 트레이트(`BufRead`)는 가져왔고 `lines`라는 메서드도 있지만, `File`이 그 트레이트를 구현하지 않아(`File: BufRead`가 만족되지 않음) 부를 수 없다는 뜻입니다.
- **원인**: 줄을 나누려면 버퍼가 필요한데 `File`에는 없습니다(2.4절).
- **해결**: `BufReader::new(file).lines()`로 감쌉니다. Part 0 ch20에서 `TcpStream`을 감싼 것과 같습니다.

### 4.3 `Path`를 `{}`로 출력 — E0277

파일: [`compile_fail/e0277_path_display.rs`](compile_fail/e0277_path_display.rs)

```text
error[E0277]: `Path` doesn't implement `std::fmt::Display`
  --> compile_fail/e0277_path_display.rs:14:24
   |
14 |     println!("파일: {}", path);
   |                     --   ^^^^ `Path` cannot be formatted with the default formatter; call `.display()` on it
   |                     |
   |                     required by this formatting parameter
   |
   = help: the trait `std::fmt::Display` is not implemented for `Path`
   = note: call `.display()` or `.to_string_lossy()` to safely print paths, as they may contain non-Unicode data
   = note: in format strings you may be able to use `{:?}` (or `{:#?}` for pretty-print) instead
   = note: required for `&Path` to implement `std::fmt::Display`

error[E0277]: `OsStr` doesn't implement `std::fmt::Display`
  --> compile_fail/e0277_path_display.rs:16:19
   |
16 |     println!("이름: {name}");
   |                     ^^^^^^ `OsStr` cannot be formatted with the default formatter
```

- **읽는 법**: 컴파일러가 이유(`may contain non-Unicode data`)와 해결책(`.display()`, `.to_string_lossy()`, `{:?}`)을 모두 알려 줍니다. 두 번째 에러는 `file_name()`이 돌려준 `&OsStr`도 같은 처지라는 뜻입니다.
- **원인**: 경로는 UTF-8이 아닐 수 있어서 `Display`가 없습니다(2.13절).
- **해결**: `path.display()`, `name.display()`로 출력합니다.

### 4.4 `BufReader`에 넘긴 `File`을 다시 씀 — E0382

파일: [`compile_fail/e0382_file_moved_into_bufreader.rs`](compile_fail/e0382_file_moved_into_bufreader.rs)

```text
error[E0382]: borrow of moved value: `file`
  --> compile_fail/e0382_file_moved_into_bufreader.rs:17:16
   |
14 |     let file = File::open("Cargo.toml")?;
   |         ---- move occurs because `file` has type `File`, which does not implement the `Copy` trait
15 |     let reader = BufReader::new(file);
   |                                 ---- value moved here
16 |     let count = reader.lines().count();
17 |     let size = file.metadata()?.len();
   |                ^^^^ value borrowed here after move
   |
   = note: you can use `File::try_clone` to duplicate a `File` instance
```

- **읽는 법**: 15번 줄에서 `file`이 `BufReader` 안으로 이동했으므로 17번 줄에서 쓸 수 없습니다. `note:`의 `try_clone`은 운영체제의 파일 핸들을 하나 더 만드는 방법입니다.
- **원인**: `BufReader::new`, `BufWriter::new`는 감쌀 값의 소유권을 가져갑니다. 감싼 뒤에도 원래 값이 필요한 경우가 많은데 그것을 잊었습니다.
- **해결**: 셋 중 하나를 고릅니다. (1) 감싼 값을 `reader.get_ref()`로 빌립니다(단, `lines()`는 `reader`를 소비하므로 그 전에). (2) `BufReader::new(&file)`처럼 빌린 파일을 감쌉니다. `&File`도 `Read`를 구현합니다. (3) 필요한 정보를 감싸기 전에 미리 읽어 둡니다.

### 4.5 리더를 `&dyn Read`로 받아 `read` — E0596

파일: [`compile_fail/e0596_read_through_shared_ref.rs`](compile_fail/e0596_read_through_shared_ref.rs)

```text
error[E0596]: cannot borrow `*reader` as mutable, as it is behind a `&` reference
  --> compile_fail/e0596_read_through_shared_ref.rs:15:17
   |
15 |         let n = reader.read(&mut buf)?;
   |                 ^^^^^^ `reader` is a `&` reference, so it cannot be borrowed as mutable
   |
help: consider changing this to be a mutable reference
   |
11 | fn count_bytes(reader: &mut dyn Read) -> io::Result<usize> {
   |                         +++
```

- **읽는 법**: `read`는 `&mut self`를 요구하는데 `reader`가 공유 참조라 가변으로 빌릴 수 없다는 뜻입니다. `help:`가 고칠 곳을 정확히 짚어 줍니다.
- **원인**: "읽기만 하니까 `&`로 충분하다"고 생각했지만, 읽으면 리더의 읽기 위치가 바뀝니다(2.2절). Part 0 ch2의 `read_line`이 *읽어 온 내용을 담을 문자열*에 `&mut`를 요구했다면, 이번에는 *리더 자신*이 `&mut`여야 하는 경우입니다.
- **해결**: `fn count_bytes(reader: &mut dyn Read)`로 바꾸고, 호출할 때도 `let mut data: &[u8] = b"hello"; count_bytes(&mut data)`처럼 넘깁니다.

### 4.6 그 밖의 실수

| 상황 | 결과 | 해결 |
| --- | --- | --- |
| `file.write(buf)?;`처럼 `write`/`read`의 반환값을 버림 | 일부만 써도 모름. clippy `unused_io_amount`가 기본으로 에러(deny) | `write_all`/`read_exact`를 씁니다(2.3절). |
| `BufWriter`를 `flush` 없이 버림 | 쓰기 에러가 조용히 사라짐 | 끝에서 `flush()?`를 부릅니다(2.7절, 예제 03). |
| 버퍼 없는 `File`에 `bytes()`나 작은 `write`를 반복 | 바이트·조각마다 시스템 호출. clippy `unbuffered_bytes` 경고 | `BufReader`/`BufWriter`로 감쌉니다. |
| `&[u8]`에 `.split(b';')` | E0277 ``expected an `FnMut(&u8)` closure, found `u8` `` (슬라이스의 고유 메서드가 골라짐) | `Cursor::new(..)`로 감싸거나 `BufRead::split(bytes, b';')`로 부릅니다(2.4절). |
| 줄바꿈 없는 `print!` 뒤에 바로 입력을 받음 | 안내 문구가 늦게 보일 수 있음 | `io::stdout().flush()?`(2.8절). |
| 출력이 많은 프로그램을 `\| head`로 실행 | `println!`이 `failed printing to stdout: Broken pipe`로 패닉 | `writeln!`으로 쓰고 `BrokenPipe`를 처리합니다(예제 04). |
| `read_dir`의 순서에 기댄 출력이나 테스트 | 환경·실행마다 결과가 다름 | 모아서 정렬합니다(2.10절). |
| `exists()`로 확인한 뒤 열거나 만들기 | 권한 에러가 `false`로 숨고, 그 사이 상태가 바뀔 수 있음(TOCTOU) | 바로 열고 에러를 처리하거나 `create_new`를 씁니다(2.9절). |
| `base.join(사용자_입력)`에 절대 경로가 들어옴 | 앞의 `base`가 무시됨. clippy `join_absolute_paths`(리터럴일 때) | 입력이 상대 경로인지(`is_relative()`) 확인합니다. |
| `format!("{}/{}", dir, name)`이나 `+`로 경로 잇기 | `Path`는 `Display`·`Add`가 없어 컴파일되지 않거나, 문자열로 바꾸며 구분자·UTF-8 문제가 생김 | `dir.join(name)`을 씁니다(2.13절). |

## 5. 연습 문제

### 연습 1. `tail` 만들기

입력의 마지막 N줄만 출력하는 함수 `fn tail(input: impl BufRead, n: usize, out: &mut impl Write) -> io::Result<()>`를 작성하세요. 먼저 바이트 슬라이스와 `Vec<u8>`으로 시험한 뒤, 표준 입력과 표준 출력으로 실행해 보세요.

`n = 2`로 만든 예제 파일(예: `examples/tail.rs`)을 실행하면 다음처럼 나와야 합니다.

```console
$ printf '1\n2\n3\n4\n5\n' | cargo run -q -p p1-ch9-io-fs-path --example tail
4
5
```

<details>
<summary>힌트</summary>

- 최근 N줄만 들고 있으면 되므로 `VecDeque<String>`(Part 1 ch4)에 `push_back`하고, 길이가 N을 넘으면 `pop_front`합니다.
- 테스트: `let mut out = Vec::new(); tail("a\nb\nc\n".as_bytes(), 2, &mut out)?; assert_eq!(out, b"b\nc\n");`
- `main`에서는 `tail(io::stdin().lock(), 2, &mut BufWriter::new(io::stdout().lock()))?`처럼 부릅니다. `BufWriter`를 임시값으로 넘기면 문장 끝에서 버려지며 flush가 시도되므로, 변수로 만들어 끝에서 `flush()?`를 부르는 편이 안전합니다.

</details>

### 연습 2. `io::copy` 직접 만들기

리더의 내용을 라이터로 모두 옮기고 옮긴 바이트 수를 돌려주는 `fn my_copy(src: &mut impl Read, dst: &mut impl Write) -> io::Result<u64>`를 작성하세요. 예제 01의 `Trickle`(조금씩 읽기)과 `Narrow`(조금씩 쓰기)로 시험하고, 결과를 표준 라이브러리의 `io::copy`와 비교하세요.

<details>
<summary>힌트</summary>

- `let mut buf = [0u8; 8192];`를 두고 `loop { let n = src.read(&mut buf)?; if n == 0 { break; } dst.write_all(&buf[..n])?; total += n as u64; }`의 모양입니다.
- `dst.write(&buf[..n])?`로 쓰면 `Narrow`에서 데이터가 빠집니다. clippy도 `unused_io_amount`로 막아 줍니다(2.3절).
- 문서에 따르면 `read`의 `ErrorKind::Interrupted`는 다시 시도해야 합니다. `Err(e) if e.kind() == io::ErrorKind::Interrupted => continue`를 넣어 보세요.

</details>

### 연습 3. 확장자별 용량 집계

디렉터리 하나를 받아 하위 디렉터리까지 모든 파일을 확장자별로 묶어 개수와 크기 합계를 출력하는 프로그램을 작성하세요. 확장자가 없는 파일은 `(없음)`으로 묶고, 결과는 확장자 이름순으로 출력합니다. 시험용 디렉터리는 예제 07처럼 `temp_dir()` 아래에 만들고 끝에서 지웁니다. 예제 07의 `project` 트리에 확장자 없는 파일 `LICENSE`(내용 `"MIT 2024"`, 8바이트)를 더하고 `"{ext:<6} {n}개 {size:>6}바이트"`로 출력하면 다음과 같습니다.

```text
(없음)   1개      8바이트
md     1개      7바이트
rs     5개     75바이트
toml   1개     24바이트
```

(`(없음)`은 네 글자라 너비 6을 채우려고 공백이 두 칸 붙습니다. 터미널에서는 한글이 두 칸을 차지해 열이 어긋나 보입니다.)

<details>
<summary>힌트</summary>

- 예제 07의 `collect_files`를 그대로 쓰고, 이름순 출력은 `BTreeMap<String, (usize, u64)>`(Part 1 ch4)의 `entry` API로 모읍니다.
- 확장자는 `path.extension().map(|e| e.to_string_lossy().into_owned()).unwrap_or_else(|| "(없음)".to_string())`입니다.
- 크기는 `fs::metadata(&path)?.len()`이고, 순회 중에 `DirEntry::metadata()`를 쓰면 경로로 다시 찾지 않아도 됩니다.

</details>

### 연습 4. 출력 경로 만들기

`src` 디렉터리 아래의 경로 목록을 받아, 같은 구조를 `out` 디렉터리 아래에 만들면서 확장자를 `.md`에서 `.html`로 바꾼 경로를 계산하세요. 파일 시스템은 건드리지 말고 경로 연산만 씁니다.

```text
src/index.md        → out/index.html
src/guide/intro.md  → out/guide/intro.html
src/img/logo.png    → out/img/logo.png
```

<details>
<summary>힌트</summary>

- `let rel = path.strip_prefix("src")?;`로 앞부분을 떼고, `Path::new("out").join(rel)`로 붙입니다.
- 확장자가 `md`인지는 `rel.extension().is_some_and(|e| e == "md")`로 확인하고, 맞으면 `with_extension("html")`을 씁니다.
- `strip_prefix`의 에러 타입은 `StripPrefixError`입니다. `src` 밖의 경로가 섞여 들어왔을 때 어떻게 할지(건너뛰기, 에러로 알리기)도 정해 보세요.

</details>

## 6. 참고 자료

- 표준 라이브러리 문서 (기준 문서)
  - [`std::io`](https://doc.rust-lang.org/std/io/index.html): 리더·라이터라는 이름, `BufReader`/`BufWriter`가 필요한 이유, 표준 입출력, `io::Result`, 그리고 "Platform-specific behavior" 절(문서에 적힌 시스템 호출은 참고용이지 약속이 아니라는 설명).
  - [`Read`](https://doc.rust-lang.org/std/io/trait.Read.html): `read`의 계약(요청보다 적게 읽어도 에러가 아님, 0이면 EOF, `Interrupted`)과 필수 메서드 안내.
  - [`BufRead`](https://doc.rust-lang.org/std/io/trait.BufRead.html): `read_line`, `lines`, `split`의 줄 끝 처리와 구현 타입 목록.
  - [`BufWriter`](https://doc.rust-lang.org/std/io/struct.BufWriter.html): drop 전에 `flush`해야 하는 이유, 버퍼링이 도움이 되지 않는 경우.
  - [`io::Error`](https://doc.rust-lang.org/std/io/struct.Error.html): `kind`, `raw_os_error`, `new`, `other`, `get_ref`, `into_inner`, `downcast`.
  - [`std::fs`](https://doc.rust-lang.org/std/fs/index.html): 함수 목록과 TOCTOU 절. [`File`](https://doc.rust-lang.org/std/fs/struct.File.html)과 [`OpenOptions`](https://doc.rust-lang.org/std/fs/struct.OpenOptions.html)에는 여는 방식별 예제와 플랫폼별 동작이 정리되어 있습니다.
  - [`std::path`](https://doc.rust-lang.org/std/path/index.html): `Path`/`PathBuf`가 `str`/`String`과 닮은 점, 경로 정규화 규칙(어떤 메서드가 반복 구분자와 `.`을 무시하는지). [`Path`](https://doc.rust-lang.org/std/path/struct.Path.html)의 `components`, `exists`, `try_exists` 설명과 [`PathBuf`](https://doc.rust-lang.org/std/path/struct.PathBuf.html)의 `push`(절대 경로를 넣으면 바뀜) 예제.
- Rust By Example: Std misc 장
  - [Path](https://doc.rust-lang.org/rust-by-example/std_misc/path.html): `Path`가 UTF-8 문자열이 아니라 `OsString`으로 저장되어 `&str`로의 변환이 실패할 수 있다는 설명과 `join`, `push`, `set_file_name`, `to_str` 예제.
  - [File I/O](https://doc.rust-lang.org/rust-by-example/std_misc/file.html)와 하위 절 [create](https://doc.rust-lang.org/rust-by-example/std_misc/file/create.html): `File::create`와 `write_all`, `OpenOptions` 언급. Part 0 ch12에서 소개한 open, read_lines 절과 이어집니다.
  - [Filesystem Operations](https://doc.rust-lang.org/rust-by-example/std_misc/fs.html): `create_dir`, `create_dir_all`, `read_dir`, `remove_file`, `remove_dir`로 `cat`·`echo`·`touch`를 흉내 내는 예제. 심볼릭 링크를 만드는 부분은 `std::os::unix` 같은 플랫폼 전용 모듈을 씁니다.
- 에러 설명: `rustc --explain E0599`처럼 에러 코드를 넣어 실행하면 터미널에서 설명과 예제를 볼 수 있습니다.

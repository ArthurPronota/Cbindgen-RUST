# Cbindgen в RUST

Проект: Rust-библиотека с C-совместимым API, заголовок для которой генерируется автоматически через **cbindgen** во время сборки. C-программа вызывает Rust-функцию через этот заголовок.

---

## 📁 Структура проекта

```text
o_070_cbinding/
├── Cargo.toml                 # настройки крейта: staticlib, зависимости
├── build.rs                   # скрипт сборки: запускает cbindgen
├── cbindgen.toml              # (опционально) настройки генерации .h
├── src/
│   └── lib.rs                 # Rust-код: Point + my_distance
├── include/
│   └── my_rust_lib.h          # ← генерируется cbindgen автоматически
├── main.c                     # C-программа, использующая Rust-библиотеку
└── target/
    └── x86_64-pc-windows-gnu/
        └── debug/
            └── libmy_lib.a    # ← статическая библиотека (создаёт cargo)
```

Два ключевых файла — **`src/lib.rs`** (Rust-источник) и **`include/my_rust_lib.h`** (генерируется из него). Их связывает **cbindgen**.

---

## 🎯 Роль cbindgen в проекте

```
src/lib.rs  ──►  cbindgen (build.rs)  ──►  include/my_rust_lib.h
   │                                              │
   └──► cargo build ──► libmy_lib.a               │
                              │                   │
                              └───────┬───────────┘
                                      ▼
                                   main.c
                                      │
                                      └──► gcc ──► my_app.exe
```

cbindgen **не собирает библиотеку** и **не компилирует C**. Он выполняет одну задачу: **читает Rust-код и пишет `.h`**, чтобы C-компилятор знал, какие функции и типы ему доступны.

---

## 📝 Исходный Rust-код

### `src/lib.rs`

```rust
// Раскладка структуры как в C — обязательно для передачи через FFI
#[repr(C)]
pub struct Point {
    x: i64,
    y: i64,
}

// Имя символа не мангится — C увидит ровно "my_distance"
#[unsafe(no_mangle)]
pub extern "C" fn my_distance(p: Point) -> f64 {
    ((p.x * p.x + p.y * p.y) as f64).sqrt()
}
```

Три атрибута — **всё, что нужно cbindgen**, чтобы понять: этот тип и эту функцию надо экспортировать в C.

| Атрибут | Что сообщает cbindgen |
|---------|----------------------|
| `#[repr(C)]` | Структура C-совместима → включить в `.h` как `typedef struct` |
| `#[no_mangle]` | Имя символа не мангится → в `.h` попадёт `my_distance` |
| `pub extern "C"` | Функция экспортируется, соглашение вызова C → прототип в `.h` |

---

## ⚙️ Настройка крейта

### `Cargo.toml`

```toml
[package]
name = "o_070_cbinding"
version = "0.1.0"
edition = "2024"

[dependencies]

[build-dependencies]
cbindgen = "0.29.4"          # ← нужен для build.rs

[lib]
name = "my_lib"              # ← имя библиотеки: libmy_lib.a
crate-type = ["staticlib"]   # ← статическая библиотека для C
```

Ключевые моменты:

- **`[build-dependencies] cbindgen`** — cbindgen доступен в `build.rs` на этапе сборки.
- **`[lib] name = "my_lib"`** → `cargo` создаст файл `libmy_lib.a`, а при линковке C используем `-lmy_lib`.
- **`crate-type = ["staticlib"]`** — вместо обычной Rust-библиотеки cargo соберёт системную `.a`.

### `build.rs`

```rust
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    cbindgen::Builder::new()
        .with_crate(".")                          // анализировать текущий крейт
        .with_language(cbindgen::Language::C)     // генерировать C, а не C++
        .generate()?                              // запустить генерацию
        .write_to_file("include/my_rust_lib.h");  // сохранить в include/

    Ok(())
}
```

`build.rs` запускается `cargo` **перед** компиляцией крейта. Это значит, что при каждом `cargo build`:
1. cbindgen читает `src/lib.rs`;
2. генерирует `include/my_rust_lib.h`;
3. затем cargo компилирует `libmy_lib.a`.

**Ручной запуск `cbindgen` больше не нужен** — заголовок всегда актуален.

---

## 📄 Сгенерированный заголовок

### `include/my_rust_lib.h` (создаётся cbindgen)

```c
#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct Point {
  int64_t x;
  int64_t y;
} Point;

double my_distance(struct Point p);
```

Что произошло при маппинге:

| Rust | C |
|------|---|
| `i64` | `int64_t` (+ `#include <stdint.h>`) |
| `f64` | `double` |
| `#[repr(C)] pub struct Point` | `typedef struct Point { ... } Point;` |
| `pub extern "C" fn my_distance` | `double my_distance(struct Point p);` |

Файл **не редактируется вручную** — при следующем `cargo build` он будет перезаписан.

---

## 🐧 Почему `--target x86_64-pc-windows-gnu`

На Windows у Rust по умолчанию таргет **`x86_64-pc-windows-msvc`**, а библиотеки MSVC несовместимы с MinGW/GCC. Чтобы собрать `.a`, совместимый с вашим `gcc.exe` из MSYS2:

```bash
rustup target add x86_64-pc-windows-gnu
```

Достаточно один раз. После этого:

```bash
cargo build --target x86_64-pc-windows-gnu
```

Результат — `target/x86_64-pc-windows-gnu/debug/libmy_lib.a` (GNU-формат, линкуется с `gcc`).

---

## 🧩 Что делает cargo при сборке

```bash
cargo build --target x86_64-pc-windows-gnu
```

Пошагово:

1. **`build.rs`** → cbindgen анализирует `src/lib.rs` → пишет `include/my_rust_lib.h`.
2. Компилируется `src/lib.rs` в объектный код под `x86_64-pc-windows-gnu`.
3. Собирается `libmy_lib.a` в `target/x86_64-pc-windows-gnu/debug/`.

Одна команда — и заголовок, и библиотека готовы.

---

## 📞 Получение списка системных библиотек

Rust-рантайм на Windows зависит от системных библиотек. Узнать их список:

```bash
cargo rustc --target x86_64-pc-windows-gnu --lib -- --print native-static-libs
```

Вывод:

```text
note: native-static-libs: -lkernel32 -lntdll -luserenv -lws2_32 -ldbghelp
```

Эти флаги нужно передать `gcc` при линковке — иначе будут `undefined reference` на символы рантайма Rust.

---

## 🖥️ C-программа

### `main.c`

```c
#include "include/my_rust_lib.h"   // ← заголовок, сгенерированный cbindgen
#include <stdio.h>

int main() {
    Point p = { .x = 3, .y = 4 };
    printf("Distance: %f\n", my_distance(p));
}
```

`main.c` **ничего не знает о Rust** — он подключает обычный C-заголовок и вызывает обычную C-функцию.

---

## 🛠️ Компиляция `main.c` и линковка

```bash
gcc main.c ^
    -Iinclude ^
    -Ltarget\x86_64-pc-windows-gnu\debug ^
    -lmy_lib ^
    -lkernel32 -lntdll -luserenv -lws2_32 -ldbghelp ^
    -o my_app.exe
```

### Разбор каждой группы флагов

| Флаг | Что делает | Связано с |
|------|-----------|-----------|
| `-Iinclude` | где искать `#include "..."` — там лежит `my_rust_lib.h` | **cbindgen** (сгенерировал `.h`) |
| `-Ltarget\x86_64-pc-windows-gnu\debug` | где искать библиотеки | **cargo** (собрал `.a` туда) |
| `-lmy_lib` | подключить `libmy_lib.a` (имя из `[lib] name`) | **Cargo.toml** |
| `-lkernel32 -lntdll -luserenv -lws2_32 -ldbghelp` | системные зависимости Rust-рантайма | вывод `--print native-static-libs` |
| `-o my_app.exe` | имя выходного файла | — |

### Порядок аргументов — важен

```
gcc <исходники>  -I<заголовки>  -L<пути>  -l<библиотеки>  -o <выход>
      1              2             3           4               5
```

`-lmy_lib` **после** `main.c` — иначе линковщик не поймёт, что символы `my_distance` нужны.

---

## 🔄 Полный цикл сборки

```bash
# 1. Один раз — установить GNU-таргет
rustup target add x86_64-pc-windows-gnu

# 2. Собрать Rust-библиотеку + сгенерировать .h (cbindgen через build.rs)
cargo build --target x86_64-pc-windows-gnu

# 3. Узнать список системных библиотек (один раз, они не меняются)
cargo rustc --target x86_64-pc-windows-gnu --lib -- --print native-static-libs

# 4. Скомпилировать C-программу с линковкой Rust-библиотеки
gcc main.c -Iinclude -Ltarget\x86_64-pc-windows-gnu\debug -lmy_lib ^
    -lkernel32 -lntdll -luserenv -lws2_32 -ldbghelp -o my_app.exe

# 5. Запустить
my_app.exe
```

Ожидаемый вывод:

```
Distance: 5.000000
```

---

## 🔍 Проверка результата

### Символы в библиотеке

```bash
nm target\x86_64-pc-windows-gnu\debug\libmy_lib.a | grep my_distance
# T my_distance     ← «T» = глобальный текстовый символ, имя не манглится
```

Если видите `_ZN...my_distance...` — забыт `#[no_mangle]`.

### Что именно сгенерировал cbindgen

```bash
type include\my_rust_lib.h
```

Должны увидеть `typedef struct Point` и прототип `double my_distance(struct Point p);`.

---

## ⚠️ Частые ошибки и их причины

| Ошибка | Причина | Связано с |
|--------|---------|-----------|
| `cannot find -lmy_lib` | нет `libmy_lib.a` в `-L` папке | сборка cargo |
| `my_rust_lib.h: No such file` | не был запущен `build.rs` или нет `-Iinclude` | cbindgen |
| `undefined reference to my_distance` | `-lmy_lib` до `main.c` | порядок gcc |
| `undefined reference to __chkstk` и т.п. | не переданы системные `-l...` | вывод `--print native-static-libs` |
| `ld returned 5` / формат `.lib` | собран MSVC-таргет, линкуется MinGW | нужен `--target ...-gnu` |
| Символ `_ZN...` | забыт `#[no_mangle]` | атрибут Rust |

---

## 🧠 Роли инструментов — сводка

| Инструмент | Что делает | Что создаёт |
|-----------|-----------|-------------|
| **cbindgen** | читает Rust-код, генерирует C-заголовок | `include/my_rust_lib.h` |
| **build.rs** | запускает cbindgen перед сборкой | (автоматизация cbindgen) |
| **cargo** | компилирует Rust в статическую библиотеку | `target/.../libmy_lib.a` |
| **gcc** | компилирует `main.c` и линкует всё вместе | `my_app.exe` |

Никакой инструмент не делает чужую работу: cbindgen — только заголовки, cargo — только библиотека, gcc — только C-часть и финальная линковка.

---

## ✅ Итог

Проект `o_070_cbinding` демонстрирует полный цикл **Rust → C**:

1. **cbindgen** через `build.rs` генерирует `include/my_rust_lib.h` из `src/lib.rs` — **автоматически при каждом `cargo build`**.
2. **cargo** собирает `libmy_lib.a` под GNU-таргет (совместим с MinGW GCC).
3. **gcc** компилирует `main.c`, подключая сгенерированный заголовок через `-Iinclude`, и линкует `libmy_lib.a` через `-L... -lmy_lib` вместе с системными библиотеками Rust-рантайма.
4. `main.c` видит обычный C-API (`Point`, `my_distance`) и не подозревает, что реализация — на Rust.

Ключевая ценность cbindgen: **единый источник истины — Rust-код**. Заголовок всегда соответствует библиотеке, ручной синхронизации `.h` и `.rs` не требуется.

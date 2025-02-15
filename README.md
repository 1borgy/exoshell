<div align="center">
  <h1>exoshell</h1>
  <img src="https://github.com/user-attachments/assets/3b8a78c0-7e1a-41fc-b819-0bfd8f9ff551" width="600">
  <div>
    <em>A console for the terminal, written in rust.</em>
  </div>
</div>

# Introduction

Exoshell uses [PyO3](https://github.com/PyO3/pyo3) and
[Maturin](https://github.com/PyO3/maturin) to build a python binary wheel.


# Example

Using exoshell from python:

```python
from exoshell import Console, Action
import colored


shell = Console()
shell.push_title("exoshell")

shell.start()

running = True
while running:
    match shell.update(1):
        case Action.Writeline(line):
            shell.print(
                colored.stylize(f">> {line}", colored.Fore.YELLOW, colored.Style.BOLD)
            )
            shell.print("\n")
            shell.print(
                colored.stylize(
                    f"<< echo: {line!r}", colored.Fore.YELLOW, colored.Style.BOLD
                )
            )
            shell.print("\n")

        case Action.Write(c):
            shell.print(f"{c}")

        case Action.Quit():
            running = False

        case None:
            ...

shell.stop()
```

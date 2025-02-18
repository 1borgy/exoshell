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


console = Console("exoshell", ("exoshell", "demo"))
console.start()

running = True
while running:
    match console.update(1):
        case Action.Writeline(line):
            console.print(
                colored.stylize(f">> {line}", colored.Fore.YELLOW, colored.Style.BOLD)
            )
            console.print("\n")
            console.print(f"echo: {line!r}")
            console.print("\n")

        case Action.Write(c):
            console.print(f"{c}")

        case Action.Quit():
            running = False

        case None:
            ...

console.stop()
```

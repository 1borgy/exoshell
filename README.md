<div align="center">
  <h1>exoshell</h1>
  <img src="https://github.com/user-attachments/assets/905729df-4c9f-4016-8f04-903e6096763b" width="600">
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
from exoshell import Action
from exoshell import Console

console = Console("exoshell", ("exoshell", "demo"))
console.start()

running = True
while running:
    match console.update(1):
        case Action.Writeline(line):
            console.print(f">> {line}\n")
            console.print(f"echo: {line!r}\n")

        case Action.Write(c):
            console.print(f"{c}")

        case Action.Quit():
            running = False

        case None:
            ...

console.stop()
```

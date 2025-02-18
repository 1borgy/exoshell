from collections.abc import Sequence

class Action:
    class Writeline:
        __match_args__ = ("_0",)
        _0: str
        def __init__(self, _0: str) -> None: ...

    class Write:
        __match_args__ = ("_0",)
        _0: str
        def __init__(self, _0: str) -> None: ...

    class Quit:
        __match_args__ = ()

class Console:
    def __init__(self, name: str, titles: Sequence[str]) -> None: ...
    def start(self) -> None: ...
    def stop(self) -> None: ...
    def update(
        self, timeout: int
    ) -> Action.Writeline | Action.Write | Action.Quit | None: ...
    def print(self, value: str) -> None: ...

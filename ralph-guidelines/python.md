---
tag: python
keywords: python, pytest, pep8, typing, poetry, pip, venv, async
---

# Wytyczne: Python

## Styl

- PEP 8 (4 spacje, `snake_case` dla funkcji/zmiennych, `PascalCase` dla klas, `UPPER_SNAKE` dla stałych)
- Type hints dla publicznych funkcji: `def foo(x: int) -> str:`
- f-string zamiast `.format()` i `%`
- `pathlib.Path` zamiast `os.path`

## Struktura projektu

- `src/<package>/` + `tests/`
- `pyproject.toml` (poetry/uv) zamiast `setup.py`
- `__init__.py` eksportuje publiczne API paczki
- Virtualenv zawsze — nigdy instalacja do systemowego Pythona

## Testy

- pytest (nie unittest)
- fixture > `setUp`/`tearDown`
- `@pytest.mark.parametrize` dla wielu wariantów
- Mock: `unittest.mock.patch` lub `pytest-mock`
- Nazwa testu: `test_<funkcja>_<warunek>_<oczekiwanie>`

## Importy

- Absolutne importy (`from myapp.models import User`)
- Kolejność: stdlib → third-party → local (pusty wiersz między grupami)
- `isort` lub `ruff` dla auto-sortowania

## Async

- `async def` tylko gdy realnie potrzebne I/O concurrency
- Nie mieszaj sync/async bez powodu — wybierz jeden styl dla modułu
- `asyncio.gather` dla równoległości, nie `create_task` + `await` w pętli

## Czego unikać

- `import *`
- Mutable default arguments (`def f(x=[]):`)
- Gołe `except:` bez typu wyjątku
- Logika w `if __name__ == "__main__":` poza prostymi skryptami
- `global` — preferuj klasę lub parametr

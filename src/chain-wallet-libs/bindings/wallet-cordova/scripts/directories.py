from pathlib import Path

script_directory = Path(__file__).parent
plugin_directory = script_directory.parent
tests_directory = plugin_directory / "tests"
repository_directory = script_directory.parent.parent.parent
rust_build_directory = repository_directory / "target"

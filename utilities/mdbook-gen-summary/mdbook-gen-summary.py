#!/usr/bin/env python3
# Simple program to create an index for rust doc documentation, to be embedded inside an md-book
# We do not use any non-standard packages, so this script should "just work" if

import argparse
import sys
import os
import subprocess
import json

from pathlib import Path
from string import punctuation


def is_dir(dir):
    """Check if the directory is a directory."""
    real_dir = Path(dir)
    if real_dir.exists() and real_dir.is_dir():
        return real_dir
    raise argparse.ArgumentTypeError(f"{dir} is not a directory.")


def is_md_file(filename):
    """Does the path exist and file ends in .md"""
    real_filename = Path(filename).relative_to(".")
    is_dir(real_filename.parent)
    if real_filename.suffix != ".md":
        raise argparse.ArgumentTypeError(f"{filename} is not a `.md` file.")
    return real_filename


def is_cargo_workspace(dir):
    """Check if the directory is a cargo workspace."""
    real_dir = is_dir(dir)
    if (real_dir / "Cargo.toml").exists():
        return real_dir
    raise argparse.ArgumentTypeError(dir)


def read_workspace_metadata(workspace):
    """Read metadata from the cargo workspace."""
    raw_meta = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=workspace,
        capture_output=True,
    )
    if raw_meta.returncode != 0:
        raise IOError(f"Cargo returned {raw_meta}")
    return json.loads(raw_meta.stdout)


def get_rustdoc_html(rustdoc, relative_to_md):
    """Get generated rustdoc html index files."""

    docs = {}

    for path in rustdoc.iterdir():
        if path.is_dir():
            if (path / "index.html").exists():
                docs[path.name] = relative_to_md / path.name / "index.html"

    return docs


def index_packages(meta):
    """Get top level crates (crates which no other crate uses), and all packages by name."""
    tl_packages = {}
    all_packages = {}
    for package in meta["packages"]:
        all_packages[package["name"]] = package
        tl_packages[package["name"]] = True

        for dependency in package["dependencies"]:
            tl_packages.pop(dependency["name"], None)
    tl_packages = tl_packages.keys()
    return (sorted(tl_packages), all_packages)

def package_description(package):
    """Get clean package description."""
    description = ""
    if "description" in package and package["description"] is not None:
        description = f" : {package['description']}".replace("\n", " ")
    return description   

def output_dep_package(package, all_packages, offset, html):
    description = package_description(package)

    docs_path = ""
    name_path = package["name"].replace("-", "_")
    if name_path in html:
        docs_path = html[name_path]

    output_string = f"{' '*offset}- [{package['name']}]({docs_path}){description}".strip().strip(punctuation)
    output_string += "\n"
    
    if "dependencies" in package:
        for dep in package["dependencies"]:
            if dep["name"] in all_packages and dep["kind"] is None:
                real_dep = all_packages[dep["name"]]
                if real_dep["source"] is None:
                    output_string += output_dep_package(real_dep, all_packages, offset + 2, html)
                    
    return output_string


def generate_mdbook_rustdoc(args):
    """Generate the mdbook page for rust docs."""
    meta = read_workspace_metadata(args.workspace)
    html = get_rustdoc_html(args.rustdoc, args.md_relative_path)
    tl_packages, all_packages = index_packages(meta)

    md_file = "# Rust API Documentation and Packages\n\n"
    md_file += "<!-- markdownlint-disable line-length -->\n\n"

    for package in tl_packages:
        real_package = all_packages[package]
        md_file += f"## {real_package['name']} {package_description(real_package)}".strip().strip(punctuation)
        md_file += "\n\n"
        md_file += output_dep_package(all_packages[package], all_packages, 0, html)
        md_file += "\n"
        
    # End with 1 trailing newline.
    md_file = md_file.strip() + "\n"
        
    args.page.write_text(md_file)


def main() -> int:
    """Parse CLI arguments."""
    parser = argparse.ArgumentParser(
        description="Creates Mdbook Index Page for Rust Doc Pages."
    )
    parser.add_argument(
        "--page",
        help="Destination mdbook index file to create",
        required=True,
        type=is_md_file,
    )
    parser.add_argument(
        "--workspace",
        help="Workspace directory of the rust project",
        type=is_cargo_workspace,
        default=".",
    )
    parser.add_argument(
        "--rustdoc",
        help="Source built rust-doc pages",
        required=True,
        type=is_dir,
    )
    parser.add_argument(
        "--md-relative-path",
        help="Path doc subdirs are, relative to the markdown page",
        required=True,
        type=Path,
    )

    args = parser.parse_args()
    generate_mdbook_rustdoc(args)
    return 0


if __name__ == "__main__":
    sys.exit(main())

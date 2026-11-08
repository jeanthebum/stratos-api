"""Run Horizon's official native.py check, with a Windows stage-copy adapter.

Requires an existing horizon-mods checkout and Horizon's Rust/linker dependencies.
The upstream compiler invocation and -D warnings are kept unchanged. On Linux/WSL,
rsync is used directly. No alternate revision or packaging action is accepted.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import types

ROOT = Path(__file__).resolve().parents[1]


def sync_stage(source, destination, target):
    """Checksum-sync two disposable directories, preserving unchanged mtimes."""
    source, destination, target = map(Path, (source, destination, target))
    source, destination, target = source.resolve(), destination.resolve(), target.resolve()
    if source != target / 'stage.new' or destination != target / 'stage/stratos_api':
        raise ValueError('refusing a stage copy outside the expected scratch paths')
    if any(p.is_symlink() or getattr(p.lstat(), 'st_file_attributes', 0) & stat.FILE_ATTRIBUTE_REPARSE_POINT for tree in (source, destination)
           for p in [tree, *tree.rglob('*')]):
        raise ValueError('stage copy does not accept symlinks or junctions')
    source_files = {p.relative_to(source): p for p in source.rglob('*') if p.is_file()}
    for p in sorted(destination.rglob('*'), key=lambda p: len(p.parts), reverse=True):
        rel = p.relative_to(destination)
        if p.is_file() and rel not in source_files:
            p.unlink()
        elif p.is_dir() and not (source / rel).is_dir():
            p.rmdir()
    for rel, src in source_files.items():
        dst = destination / rel
        dst.parent.mkdir(parents=True, exist_ok=True)
        if not dst.exists() or hashlib.sha256(src.read_bytes()).digest() != hashlib.sha256(dst.read_bytes()).digest():
            shutil.copy2(src, dst)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--registry', type=Path, required=True, help='horizon-rust/horizon-mods checkout')
    args = parser.parse_args()
    driver = args.registry.resolve() / 'native/tools/native.py'
    spec = importlib.util.spec_from_file_location('horizon_native_check', driver)
    upstream = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(upstream)
    manifest = json.loads((ROOT / 'horizon-native.json').read_text())
    compat = json.loads((ROOT / 'compatibility.json').read_text())
    if (manifest['horizon'], manifest['mixins']) != (compat['horizon_revision'], compat['horizon_mixins_revision']):
        sys.exit('compatibility.json and release pins disagree')
    # Cargo gives CARGO_ENCODED_RUSTFLAGS precedence over RUSTFLAGS. Preserve any
    # required platform linker flags, but ensure they cannot bypass -D warnings.
    if 'CARGO_ENCODED_RUSTFLAGS' in os.environ:
        os.environ['CARGO_ENCODED_RUSTFLAGS'] += '\x1f-D\x1fwarnings'
    if os.name == 'nt':
        original_run = subprocess.run
        def run(command, **kwargs):
            if command[:4] == ['rsync', '-a', '--checksum', '--delete']:
                sync_stage(command[4], command[5], upstream.TARGET)
                return subprocess.CompletedProcess(command, 0)
            return original_run(command, **kwargs)
        upstream.subprocess = types.SimpleNamespace(run=run, DEVNULL=subprocess.DEVNULL)
        print('Windows: adapting only rsync stage copying; official compiler checks remain enabled.', flush=True)
    sys.argv = [str(driver), 'check', str(ROOT)]
    upstream.main()


if __name__ == '__main__':
    main()

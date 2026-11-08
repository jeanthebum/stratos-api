"""Verify the source payload and create reproducible native-mod and SDK archives."""
from pathlib import Path
import hashlib, json, tomllib, zipfile

ROOT = Path(__file__).resolve().parents[1]

def normalized(data): return data.replace(b'\r\n', b'\n')
def digest(data): return hashlib.sha256(data).hexdigest()

def archive(path, files):
    with zipfile.ZipFile(path, 'w', compression=zipfile.ZIP_DEFLATED, compresslevel=9) as out:
        for file in sorted(files):
            info = zipfile.ZipInfo(file.relative_to(ROOT).as_posix(), (2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            out.writestr(info, normalized(file.read_bytes()))

def main():
    manifest = json.loads((ROOT / 'horizon-native.json').read_text())
    mod = ROOT / 'mixins' / manifest['id']
    mixin = tomllib.loads((mod / 'horizon-mixin.toml').read_text())
    core = tomllib.loads((mod / 'core/Cargo.toml').read_text())
    assert manifest['version'] == mixin['mod']['version'] == core['package']['version']
    assert manifest['horizon'] == mixin['compatibility']['horizon_revision']
    index = json.loads((mod / 'core/data/engine-patches.json').read_text())
    assert index['schema'] == 1 and index['files']
    seen = set()
    for entry in index['files']:
        name = entry['path']
        assert name not in seen and not Path(name).is_absolute() and '..' not in Path(name).parts
        seen.add(name)
        data = (mod / 'core/data/engine' / name).read_bytes()
        assert digest(normalized(data)) == entry['after_sha256'], name
    def eligible(file):
        return file.is_file() and not any(p in {'.git','target','dist','releases','__pycache__'} for p in file.relative_to(ROOT).parts) and not (file.name == 'Cargo.lock' and file.parent != ROOT) and file.suffix != '.pyc'
    files = [f for f in mod.rglob('*') if eligible(f)]
    assert len(files) < 511
    dist = ROOT / 'dist'; dist.mkdir(exist_ok=True)
    native = dist / f"StratosAPI-{manifest['version']}.hnmod"
    archive(native, [ROOT / 'horizon-native.json', *files])
    sdk = dist / f"StratosAPI-{manifest['version']}-source.zip"
    archive(sdk, [f for f in ROOT.rglob('*') if eligible(f)])
    sums = {p.name: digest(p.read_bytes()) for p in [native, sdk]}
    (dist / 'SHA256SUMS').write_text(''.join(f'{h}  {n}\n' for n,h in sums.items()), encoding='utf8')
    print(json.dumps({'files':len(files)+1,'sha256':sums},indent=2))

if __name__ == '__main__': main()

#[path = "../src/patcher.rs"]
mod patcher;
use std::{fs, path::PathBuf, sync::atomic::{AtomicUsize, Ordering}};
use sha2::{Digest, Sha256};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture { root: PathBuf, package: PathBuf }
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("stratos-overlay-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir_all(&root).unwrap();
        let package = root.join("native-crates/stratos_api/core");
        fs::create_dir_all(package.join("data/engine")).unwrap();
        fs::write(root.join(".horizon-mixins-stage.json"), "{}").unwrap();
        fs::write(root.join("native-post-build.json"), "[]").unwrap();
        Self {root, package}
    }
    fn files(&self, entries: &[(&str, Option<&str>, &str)]) {
        let mut files = vec![];
        for (path, before, after) in entries {
            let source = self.package.join("data/engine").join(path);
            fs::create_dir_all(source.parent().unwrap()).unwrap();
            fs::write(source, after).unwrap();
            let target = self.root.join(path);
            fs::create_dir_all(target.parent().unwrap()).unwrap();
            if let Some(text) = before {fs::write(target, text).unwrap();}
            let hash = |s: &str| format!("{:x}", Sha256::digest(s.replace("\r\n", "\n").as_bytes()));
            files.push(serde_json::json!({"path":path,"before_sha256":before.map(hash),"after_sha256":hash(after)}));
        }
        fs::write(self.package.join("data/engine-patches.json"), serde_json::to_vec(&serde_json::json!({"schema":1,"files":files})).unwrap()).unwrap();
    }
}
impl Drop for Fixture { fn drop(&mut self) { fs::remove_dir_all(&self.root).unwrap(); } }
const A: &str = "crates/horizon-shader-api/src/lib.rs";
const B: &str = "crates/horizon-client-render/src/gfx.rs";
#[test]
fn applies_pristine_and_new_files_and_is_idempotent_across_line_endings() {
    let f = Fixture::new(); f.files(&[(A, Some("old\r\n"), "new\n"), (B, None, "added\n")]);
    assert_eq!(patcher::stage_root(&f.package), Some(f.root.clone()));
    let r = patcher::apply(&f.root, &f.package).unwrap();
    assert_eq!((r.written,r.watched.len()), (2,2));
    assert_eq!(fs::read_to_string(f.root.join(B)).unwrap(), "added\n");
    assert_eq!(patcher::apply(&f.root,&f.package).unwrap().written, 0);
    // A fresh staging pass restores pristine source; the next build reapplies it.
    fs::write(f.root.join(A), "old\n").unwrap();
    assert_eq!(patcher::apply(&f.root,&f.package).unwrap().written, 1);
}
#[test]
fn another_mods_conflict_prevents_all_writes() {
    let f = Fixture::new(); f.files(&[(A,Some("old"),"new"),(B,Some("old"),"new")]);
    fs::write(f.root.join(B), "another mod").unwrap();
    assert!(patcher::apply(&f.root,&f.package).err().unwrap().contains("source conflict"));
    assert_eq!(fs::read_to_string(f.root.join(A)).unwrap(), "old");
}
#[test]
fn refuses_unmarked_workspace_and_corrupted_payload() {
    let f = Fixture::new(); f.files(&[(A,Some("old"),"new")]);
    fs::remove_file(f.root.join(".horizon-mixins-stage.json")).unwrap();
    assert!(patcher::apply(&f.root,&f.package).is_err());
    fs::write(f.root.join(".horizon-mixins-stage.json"), "{}").unwrap();
    fs::write(f.package.join("data/engine").join(A), "tampered").unwrap();
    assert!(patcher::apply(&f.root,&f.package).err().unwrap().contains("checksum"));
    assert_eq!(fs::read_to_string(f.root.join(A)).unwrap(), "old");
    assert!(patcher::stage_root(&f.root).is_none());
}
#[test]
fn refuses_traversal_and_duplicate_targets() {
    let f = Fixture::new(); f.files(&[(A,Some("old"),"new")]);
    let index = f.package.join("data/engine-patches.json");
    let original = fs::read_to_string(&index).unwrap();
    fs::write(&index, original.replace(A, "crates/horizon-shader-api/../../escape.rs")).unwrap();
    assert!(patcher::apply(&f.root,&f.package).err().unwrap().contains("unsafe"));
    let mut json: serde_json::Value = serde_json::from_str(&original).unwrap();
    let entry=json["files"][0].clone(); json["files"].as_array_mut().unwrap().push(entry);
    fs::write(index,serde_json::to_vec(&json).unwrap()).unwrap();
    assert!(patcher::apply(&f.root,&f.package).err().unwrap().contains("duplicate"));
}

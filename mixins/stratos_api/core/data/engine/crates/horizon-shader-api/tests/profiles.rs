use horizon_shader_api::Manifest;
fn source(extra: &str) -> String {
    format!(
        r#"schema = 1
id = "profile_test"
name = "Profiles"
version = "1"
api = "0.2"
[[post]]
file = "post.wgsl"
[[options]]
id = "profile"
name = "Profile"
type = "choice"
values = ["Low", "High", "Custom"]
default = "High"
[[options]]
id = "quality"
name = "Quality"
type = "choice"
values = ["Fast", "Fine"]
default = "Fine"
[[options]]
id = "enabled"
name = "Enabled"
type = "bool"
default = true
[[options]]
id = "amount"
name = "Amount"
type = "float"
default = 0.5
min = 0.0
max = 1.0
[[options]]
id = "exposure"
name = "Exposure"
type = "float"
default = 1.0
min = 0.0
max = 2.0
{extra}
"#
    )
}
const PROFILES: &str = r#"
[profiles]
option = "profile"
custom = "Custom"
[[profiles.presets]]
value = "Low"
values = {quality = "Fast", enabled = false, amount = 0.0}
[[profiles.presets]]
value = "High"
values = {quality = "Fine", enabled = true, amount = 0.5}
"#;
#[test]
fn profiles_switch_every_budget_setting_and_preserve_exposure() {
    let m = Manifest::parse(&source(PROFILES)).unwrap();
    let p = m.profiles.unwrap();
    let mut v = vec![0.0, 1.0, 1.0, 0.5, 1.4];
    p.apply(&mut v);
    assert_eq!(v, vec![0.0, 0.0, 0.0, 0.0, 1.4]);
    v[0] = 1.0;
    p.changed(0, &mut v);
    assert_eq!(v, vec![1.0, 1.0, 1.0, 0.5, 1.4]);
}
#[test]
fn manual_budget_edits_become_custom_without_overwriting_the_edit() {
    let p = Manifest::parse(&source(PROFILES))
        .unwrap()
        .profiles
        .unwrap();
    let mut v = vec![1.0, 1.0, 1.0, 0.5, 1.0];
    v[4] = 1.5;
    p.changed(4, &mut v);
    assert_eq!(v[0], 1.0);
    v[1] = 0.0;
    p.changed(1, &mut v);
    assert_eq!(v[0], 2.0);
    assert_eq!(v[1], 0.0);
    p.apply(&mut v);
    assert_eq!(v[1], 0.0);
}
#[test]
fn old_saved_settings_are_preserved_as_custom() {
    let p = Manifest::parse(&source(PROFILES))
        .unwrap()
        .profiles
        .unwrap();
    let mut v = vec![1.0, 0.0, 1.0, 0.7, 1.2];
    p.reconcile(&mut v);
    assert_eq!(v, vec![2.0, 0.0, 1.0, 0.7, 1.2]);
}
#[test]
fn profiles_are_only_available_on_the_extended_api() {
    assert!(Manifest::parse(&source(PROFILES).replace("api = \"0.2\"", "api = \"0.1\"")).is_err());
    assert!(Manifest::parse(&source("").replace("api = \"0.2\"", "api = \"0.1\"")).is_ok());
}
#[test]
fn rejects_unknown_options_and_recursive_selector_changes() {
    for bad in [
        PROFILES.replace("amount = 0.0", "absent = 0.0"),
        PROFILES.replace("amount = 0.0", "profile = \"High\""),
    ] {
        assert!(Manifest::parse(&source(&bad)).is_err());
    }
}
#[test]
fn rejects_invalid_types_ranges_and_nonfinite_values() {
    for value in ["true", "2.0", "-0.0000001", "nan", "inf", "\"oops\""] {
        assert!(
            Manifest::parse(&source(
                &PROFILES.replace("amount = 0.0", &format!("amount = {value}"))
            ))
            .is_err()
        );
    }
    assert!(Manifest::parse(&source(&PROFILES.replace("enabled = false", "enabled = 0"))).is_err());
    assert!(
        Manifest::parse(&source(
            &PROFILES.replace("quality = \"Fast\"", "quality = \"Missing\"")
        ))
        .is_err()
    );
}
#[test]
fn rejects_incomplete_duplicate_and_custom_presets() {
    for bad in [
        PROFILES.replace("value = \"Low\"", "value = \"High\""),
        PROFILES.replace("value = \"Low\"", "value = \"Custom\""),
        PROFILES.replace(", amount = 0.0", ""),
    ] {
        assert!(Manifest::parse(&source(&bad)).is_err());
    }
}

use horizon_shader_api::{Manifest, stratos};
#[test]
fn public_abi_is_frozen() {
    use horizon_shader_api::{Api,Frame,package::sha256_hex};
    let stages: Vec<_> = Api::V0_2.stages().iter().map(|s| serde_json::json!({
        "stage":format!("{s:?}"), "prelude_sha256":sha256_hex(s.prelude(Api::V0_2).as_bytes()),
        "entries_sha256":sha256_hex(s.entries(Api::V0_2).as_bytes())
    })).collect();
    let fields: Vec<_> = Frame::WGSL.fields.iter().zip(Frame::OFFSETS).map(|(f,o)|serde_json::json!({"name":f.name,"type":f.ty,"offset":o})).collect();
    let actual=serde_json::json!({"api":"0.1.0","frame_bytes":std::mem::size_of::<Frame>(),"fields":fields,"stages":stages});
    assert_eq!(std::mem::size_of::<Frame>(),stratos_api_core::FRAME_BYTES);
    let frozen: serde_json::Value=serde_json::from_str(stratos_api_core::ABI_SNAPSHOT).unwrap();
    assert_eq!(actual,frozen,"StratosAPI 0.1.0 cannot silently change its ABI");
}
fn source(extra: &str) -> String { format!(r#"schema=1
id="someone_elses_shader"
name="Independent shader"
version="53.9.2"
api="0.2"
[weather]
file="rain.wgsl"
[[options]]
id="drops"
name="Drops"
type="float"
min=0.0
max=2.0
default=1.0
[[options]]
id="wet"
name="Humidity"
type="float"
min=0.0
max=2.0
default=1.0
[[options]]
id="active"
name="Enabled"
type="bool"
default=true
{extra}
"#) }
const CONTRACT: &str = r#"[stratos]
api=">=0.1.0, <0.2.0"
[stratos.weather]
liquid_droplets=true
leaf_drips_option="drops"
humidity_option="wet"
enabled_option="active"
"#;
#[test]
fn pack_versions_and_names_do_not_define_the_api() {
    let m=Manifest::parse(&source(CONTRACT)).unwrap();
    assert_eq!(m.stratos.as_ref().unwrap().negotiated,"0.1.0");
    let s=stratos::weather_settings(&m,[("drops",1.5),("wet",0.7),("active",1.0)].into_iter());
    assert!(s.liquid); assert_eq!(s.drips,1.5); assert_eq!(s.humidity,0.7);
    assert_eq!(stratos::weather_settings(&m,[("drops",1.5),("active",0.0)].into_iter()).drips,0.0);
}
#[test]
fn incompatible_contract_is_rejected_before_reading_shader_files() {
    for range in ["=0.1.1",">=0.2.0","nope",""] {
        let error=Manifest::parse(&source(&CONTRACT.replace(">=0.1.0, <0.2.0",range))).unwrap_err().to_string();
        assert!(error.contains("stratos") || error.contains("StratosAPI"),"{error}");
    }
}
#[test]
fn contract_rejects_invalid_weather_controls_and_api_line() {
    for (from,to) in [("leaf_drips_option=\"drops\"","leaf_drips_option=\"missing\""),("humidity_option=\"wet\"","humidity_option=\"active\""),("enabled_option=\"active\"","enabled_option=\"drops\""),("liquid_droplets=true","liquid_droplets=false")] {
        assert!(Manifest::parse(&source(&CONTRACT.replace(from,to))).is_err());
    }
    assert!(Manifest::parse(&source(CONTRACT).replace("api=\"0.2\"","api=\"0.1\"")).is_err());
    assert!(Manifest::parse(&source(CONTRACT).replace("[weather]\nfile=\"rain.wgsl\"","[[post]]\nfile=\"post.wgsl\"")).is_err());
}
#[test]
fn undeclared_extension_does_not_enable_custom_particles() {
    for contract in ["", "[stratos]\napi=\"=0.1.0\""] {
        let m=Manifest::parse(&source(contract)).unwrap();
        assert!(!stratos::weather_settings(&m,[("drops",2.0)].into_iter()).liquid);
    }
}

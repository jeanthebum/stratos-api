// Listed through Horizon's documented native-mod hook.
fn stratos_list(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    mods.push(crate::game::native_hooks::NativeMod {
        id: "stratos_api".into(),
        title: "StratosAPI 0.1.1".into(),
        description: String::new(),
        keys: Vec::new(),
    });
}

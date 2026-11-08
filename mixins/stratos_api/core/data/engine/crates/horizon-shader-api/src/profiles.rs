//! Validated option bundles selected by an ordinary choice button (API 0.2).
use crate::{OptionKind, PackOption};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq)]
pub struct OptionProfiles {
    pub selector: usize,
    pub custom: f64,
    pub presets: Vec<OptionPreset>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct OptionPreset {
    pub choice: f64,
    pub values: Vec<(usize, f64)>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawProfiles {
    option: String,
    custom: String,
    presets: Vec<RawPreset>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPreset {
    value: String,
    values: BTreeMap<String, toml::Value>,
}

fn constant(kind: &OptionKind, value: &toml::Value) -> Option<f64> {
    match kind {
        OptionKind::Bool { .. } => value.as_bool().map(|b| f64::from(u8::from(b))),
        OptionKind::Choice { values, .. } => {
            let name = value.as_str()?;
            values.iter().position(|v| v == name).map(|i| i as f64)
        }
        OptionKind::Int { min, max, .. } => {
            let n = value.as_integer()?;
            (n >= i64::from(*min) && n <= i64::from(*max)).then_some(n as f64)
        }
        OptionKind::Float { min, max, .. } => {
            let n = value
                .as_float()
                .or_else(|| value.as_integer().map(|i| i as f64))?;
            let v = n as f32;
            (n.is_finite() && v.is_finite() && v >= *min && v <= *max).then_some(f64::from(v))
        }
    }
}

impl OptionProfiles {
    pub(crate) fn check(raw: RawProfiles, options: &[PackOption]) -> Result<Self, String> {
        let selector = options
            .iter()
            .position(|o| o.id == raw.option)
            .ok_or_else(|| format!("profiles.option: no option {:?}", raw.option))?;
        let OptionKind::Choice {
            values: choices, ..
        } = &options[selector].kind
        else {
            return Err("profiles.option must name a choice".into());
        };
        let custom = choices
            .iter()
            .position(|v| *v == raw.custom)
            .ok_or_else(|| "profiles.custom must name a selector choice".to_owned())?
            as f64;
        if raw.presets.is_empty() || raw.presets.len() > 32 {
            return Err("profiles: expected 1..32 presets".into());
        }
        let mut seen = BTreeSet::new();
        let mut presets = Vec::new();
        let mut expected_keys = None;
        for preset in raw.presets {
            let choice = choices
                .iter()
                .position(|v| *v == preset.value)
                .ok_or_else(|| format!("unknown profile choice {:?}", preset.value))?;
            if choice as f64 == custom || !seen.insert(choice) {
                return Err("profile choices must be unique and cannot be Custom".into());
            }
            if preset.values.is_empty() {
                return Err("profile needs at least one setting".into());
            }
            let mut values = Vec::new();
            for (id, value) in preset.values {
                let i = options
                    .iter()
                    .position(|o| o.id == id)
                    .ok_or_else(|| format!("profile setting {id:?} is not an option"))?;
                if i == selector {
                    return Err("profile cannot set its own selector".into());
                }
                let number = constant(&options[i].kind, &value)
                    .ok_or_else(|| format!("profile {:?}: invalid value for {id}", preset.value))?;
                values.push((i, number));
            }
            let keys: BTreeSet<_> = values.iter().map(|(i, _)| *i).collect();
            if expected_keys
                .as_ref()
                .is_some_and(|expected| *expected != keys)
            {
                return Err(
                    "all profiles must set the same options, so switching is deterministic".into(),
                );
            }
            expected_keys = Some(keys);
            presets.push(OptionPreset {
                choice: choice as f64,
                values,
            });
        }
        if presets.len() + 1 != choices.len() {
            return Err("each selector choice needs a preset, except Custom".into());
        }
        Ok(Self {
            selector,
            custom,
            presets,
        })
    }

    /// Choosing a bundle updates every included value in one settings operation.
    pub fn apply(&self, values: &mut [f64]) {
        let Some(&choice) = values.get(self.selector) else {
            return;
        };
        if let Some(preset) = self.presets.iter().find(|p| p.choice == choice) {
            for &(i, v) in &preset.values {
                if let Some(at) = values.get_mut(i) {
                    *at = v;
                }
            }
        }
    }

    /// Reconcile manual edits or existing settings without silently overwriting them.
    pub fn reconcile(&self, values: &mut [f64]) {
        let Some(&choice) = values.get(self.selector) else {
            return;
        };
        if let Some(preset) = self.presets.iter().find(|p| p.choice == choice)
            && preset
                .values
                .iter()
                .any(|&(i, v)| values.get(i).is_none_or(|n| (*n - v).abs() > 1e-6))
        {
            values[self.selector] = self.custom;
        }
    }
    pub fn changed(&self, option: usize, values: &mut [f64]) {
        if option == self.selector {
            self.apply(values);
        } else {
            self.reconcile(values);
        }
    }
}

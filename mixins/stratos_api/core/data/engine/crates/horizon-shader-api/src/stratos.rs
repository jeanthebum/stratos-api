//! StratosAPI contracts: independent of a shader's name, ID or release number.
use crate::{Api, Manifest, OptionKind, PackOption};
use serde::Deserialize;
pub use stratos_api_core::{FRAME_BYTES, IMPLEMENTATION_VERSION, SUPPORTED_APIS, negotiate, resolve};

#[derive(Clone, Debug, PartialEq)]
pub struct Contract {
    pub requirement: String,
    pub negotiated: String,
    pub weather: Option<Weather>,
}
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Weather {
    #[serde(default)]
    pub liquid_droplets: bool,
    pub leaf_drips_option: Option<String>,
    pub humidity_option: Option<String>,
    pub enabled_option: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Raw {
    api: String,
    weather: Option<Weather>,
}
impl Raw {
    pub(crate) fn check(self, api: Api, options: &[PackOption], has_weather: bool) -> Result<Contract, String> {
        if api != Api::V0_2 { return Err("[stratos] requires Horizon shader api = \"0.2\"".into()); }
        let negotiated = resolve(&self.api)?;
        if let Some(weather) = &self.weather {
            if !has_weather { return Err("stratos.weather requires a [weather] stage".into()); }
            if weather.leaf_drips_option.is_some() && !weather.liquid_droplets {
                return Err("leaf drips require stratos.weather.liquid_droplets = true".into());
            }
            for (label, id, boolean) in [
                ("leaf_drips_option", &weather.leaf_drips_option, false),
                ("humidity_option", &weather.humidity_option, false),
                ("enabled_option", &weather.enabled_option, true),
            ] {
                let Some(id) = id else { continue };
                let option = options.iter().find(|o| o.id == *id)
                    .ok_or_else(|| format!("stratos.weather.{label}: unknown option {id:?}"))?;
                let valid = if boolean { matches!(option.kind, OptionKind::Bool { .. }) }
                    else { matches!(option.kind, OptionKind::Float { .. } | OptionKind::Int { .. }) };
                if !valid { return Err(format!("stratos.weather.{label}: {id:?} must be {}", if boolean { "bool" } else { "numeric" })); }
            }
        }
        Ok(Contract { requirement: self.api, negotiated, weather: self.weather })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeatherSettings { pub liquid: bool, pub drips: f32, pub humidity: f32 }

/// Third-party packs choose their own control IDs. Only pre-Stratos Atmos gets
/// the narrow legacy mapping; it does not define the public shader contract.
pub fn weather_settings<'a>(manifest: &Manifest, values: impl Iterator<Item=(&'a str, f64)>) -> WeatherSettings {
    let legacy;
    let weather = if let Some(contract) = &manifest.stratos {
        contract.weather.as_ref()
    } else if manifest.api == Api::V0_2 && manifest.id == "afterglow_hdr" && manifest.weather.is_some() {
        legacy = Weather { liquid_droplets: manifest.options.iter().any(|o| o.id == "canopy_drips"),
            leaf_drips_option: Some("canopy_drips".into()), humidity_option: Some("jungle_moisture".into()), enabled_option: Some("biome_atmosphere".into()) };
        Some(&legacy)
    } else { None };
    let Some(weather) = weather else { return WeatherSettings { liquid: false, drips: 0.0, humidity: 1.0 } };
    let (mut enabled, mut drips, mut humidity) = (true, 0.0, 1.0);
    for (name, value) in values.filter(|(_, v)| v.is_finite()) {
        if weather.enabled_option.as_deref() == Some(name) { enabled = value > 0.5; }
        if weather.leaf_drips_option.as_deref() == Some(name) { drips = value.clamp(0.0, 2.0) as f32; }
        if weather.humidity_option.as_deref() == Some(name) { humidity = value.clamp(0.0, 2.0) as f32; }
    }
    WeatherSettings { liquid: weather.liquid_droplets, drips: if enabled && weather.liquid_droplets { drips } else { 0.0 }, humidity }
}

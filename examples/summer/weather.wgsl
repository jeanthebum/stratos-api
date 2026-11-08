// Original minimal water shading; no Minecraft rain sprite supplies its shape.
override leaf_beads: f32 = 1.0;
override humidity: f32 = 1.0;
override wet_effects: bool = true;
fn weather(input: WeatherInput) -> vec4<f32> {
    if (input.snow) { return input.color; }
    let kind = weather_kind();
    if (!wet_effects && kind >= 2u) { return vec4<f32>(0.0); }
    var p = weather_uv() * 2.0 - 1.0;
    if (kind == 0u) { p = (fract(weather_uv() * vec2<f32>(7.0, 3.0)) - 0.5) * vec2<f32>(22.0, 10.0); }
    let radius = length(p);
    let edge = max(fwidth(radius), 0.02);
    var cover = 1.0 - smoothstep(1.0 - edge, 1.0 + edge, radius);
    if (kind == 4u) { cover = (1.0 - smoothstep(0.07, 0.07 + edge, abs(radius - 0.75))) * (1.0 - weather_progress()); }
    let z = sqrt(max(0.0, 1.0 - dot(p, p)));
    let fresnel = 0.02 + 0.98 * pow(1.0 - z, 5.0);
    let size = vec2<i32>(textureDimensions(weather_scene_color));
    let pixel = clamp(vec2<i32>(weather_screen().xy + p * 1.5), vec2<i32>(0), size - 1);
    let behind = textureLoad(weather_scene_color, pixel, 0).rgb;
    let highlight = pow(max(0.0, dot(normalize(vec3<f32>(p, z)), normalize(vec3<f32>(-0.4, 0.6, 1.0)))), 60.0);
    let water = mix(behind, vec3<f32>(0.35, 0.55, 0.85), fresnel * 0.6) + vec3<f32>(1.0, 0.82, 0.55) * highlight;
    return vec4<f32>(water, clamp(cover * weather_opacity() * 0.75, 0.0, 0.9));
}

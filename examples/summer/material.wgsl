fn material(input: MaterialInput) -> Surface {
    let day = smoothstep(-0.10, 0.16, frame.sun_direction.y);
    let diffuse = max(dot(input.normal, frame.light_direction), 0.0);
    // Environment weights describe the camera's blended surroundings, not each pixel.
    let humid = clamp(frame.biome_jungle + frame.biome_swamp, 0.0, 1.0);
    let ambient = mix(vec3<f32>(0.19, 0.32, 0.49), vec3<f32>(0.18, 0.35, 0.32), humid * 0.3);
    let outdoor = pow(input.light.y, 2.0);
    let light = (ambient * 0.50 + vec3<f32>(1.0, 0.81, 0.53) * diffuse * 2.2) * outdoor * day
        + input.lightmap * (0.12 + pow(input.light.x, 2.0));
    return Surface(vec4<f32>(input.albedo.rgb * input.tint.rgb * light, input.albedo.a * input.tint.a));
}

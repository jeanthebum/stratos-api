// Linear-light sky. Keep this example small enough to replace piece by piece.
fn sky(input: SkyInput) -> vec4<f32> {
    let day = smoothstep(-0.12, 0.12, frame.sun_direction.y);
    let elevation = pow(clamp(input.direction.y, 0.0, 1.0), 0.45);
    let daylight = mix(vec3<f32>(0.55, 0.76, 0.92), vec3<f32>(0.055, 0.27, 0.64), elevation);
    let angle = dot(input.direction, frame.sun_direction);
    let disk = smoothstep(0.99996, 0.99997, angle);
    let halo = pow(max(angle, 0.0), 300.0) * 0.15;
    let color = mix(vec3<f32>(0.002, 0.004, 0.012), daylight, day);
    return vec4<f32>(color + vec3<f32>(1.0, 0.81, 0.53) * (disk * 8.0 + halo) * day, 1.0);
}

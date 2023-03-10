pub fn axis_from_digital(neg: bool, pos: bool) -> f32 {
    match (neg, pos) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    }
}

use bevy::prelude::*;
use roots::find_roots_quadratic;
use roots::Roots;

/// a and d are the start and end points.
/// b and c are the handles.
/// t is the percent. Clamped from 0.0 to 1.0.
#[allow(dead_code)]
pub fn bezier_vec2(a: Vec2, b: Vec2, c: Vec2, d: Vec2, t: f32) -> Vec2 {
    // default behaviour is to clamp
    let mut t = t;
    if t < 0.0 {
        t = 0.0;
    }
    if t > 1.0 {
        t = 1.0;
    }

    bezier_vec2_unclamped(a, b, c, d, t)
}

/// a and d are the start and end points.
/// b and c are the handles.
/// t is the percent.
#[allow(dead_code)]
pub fn bezier_vec2_unclamped(a: Vec2, b: Vec2, c: Vec2, d: Vec2, t: f32) -> Vec2 {
    let result = (1.0 - t).powi(3) * a
        + 3.0 * (1.0 - t).powi(2) * b
        + 3.0 * t.powi(2) * (1.0 - t) * c
        + t.powi(3) * d;

    result
}

/// a and d are the start and end values.
/// b and c are the handles.
/// t is the percent. Clamped from 0.0 to 1.0
#[allow(dead_code)]
pub fn bezier_f32(a: f32, b: f32, c: f32, d: f32, t: f32) -> f32 {
    let mut t = t;
    if t < 0.0 {
        t = 0.0;
    }
    if t > 1.0 {
        t = 1.0;
    }

    bezier_f32_unclamped(a, b, c, d, t)
}

/// a and d are the start and end values.
/// b and c are the handles.
/// t is the percent.
#[allow(dead_code)]
pub fn bezier_f32_unclamped(a: f32, b: f32, c: f32, d: f32, t: f32) -> f32 {
    let result = (1.0 - t).powi(3) * a
        + 3.0 * (1.0 - t).powi(2) * b
        + 3.0 * t.powi(2) * (1.0 - t) * c
        + t.powi(3) * d;

    result
}

pub struct Interception {
    pub intercept_pos: Vec2,
    pub heading: Vec2,
    pub time: f32,
}

pub fn get_intercept(
    predator_pos: Vec2,
    predator_speed: f32,
    prey_pos: Vec2,
    prey_dir: Vec2,
    prey_speed: f32,
) -> Option<Interception> {
    if predator_pos == prey_pos {
        return Some(Interception {
            intercept_pos: predator_pos,
            heading: Vec2::ZERO,
            time: 0.0,
        });
    }
    if predator_speed <= 0.0 {
        return None;
    }

    let vec_from_prey = predator_pos - prey_pos;
    let dist_to_prey_sq = vec_from_prey.length_squared();
    let dist_to_prey = vec_from_prey.length();

    if prey_speed == 0.0 {
        return Some(Interception {
            intercept_pos: prey_pos,
            heading: vec_from_prey.normalize_or_zero(),
            time: dist_to_prey / predator_speed,
        });
    } else {
        let a = predator_speed * predator_speed - prey_speed * prey_speed;
        let b = 2.0 * vec_from_prey.dot(prey_dir * prey_speed);
        let c = -dist_to_prey_sq;

        // quad solver
        // let roots = find_root_intervals(function, init, min_interval_width, min_image_width, max_recursions)
        let roots = find_roots_quadratic(a, b, c);
        // let roots = find_root_intervals_to(function, init, min_interval_width, min_image_width, max_recursions, results, candidates)
        match roots {
            Roots::No(_) => {
                return None;
            }
            Roots::One(one) => {
                // one mode can be if both solutions match
                // is this possible?
                if one[0] < 0.0 {
                    return None;
                }
                let time_to_intercept = one[0];

                let intercept_pos =
                    predator_pos + prey_pos + prey_dir * prey_speed * time_to_intercept;
                let heading = intercept_pos - predator_pos;
                return Some(Interception {
                    intercept_pos,
                    heading,
                    time: time_to_intercept,
                });
            }
            Roots::Two(two) => {
                if two[0] < 0.0 && two[1] < 0.0 {
                    // both negative. Intercept is in the past
                    return None;
                }
                let time_to_intercept = if two[0] > 0.0 && two[1] > 0.0 {
                    // both positive, take the smaller one
                    // it equates with a shorter time, sooner intersection
                    // two[0].min(two[1])
                    // first is already less. solver orders them already
                    two[0]
                } else {
                    // return larger. Smaller one is negative
                    // two[0].max(two[1])
                    two[1]
                };

                let intercept_pos =
                    prey_pos + prey_dir * prey_speed * time_to_intercept;
                let heading = intercept_pos - predator_pos;
                return Some(Interception {
                    intercept_pos,
                    heading,
                    time: time_to_intercept,
                });
            }
            Roots::Three(_) => {
                // not possible for quadratic
                return None;
            }
            Roots::Four(_) => {
                // not possible for quadratic
                return None;
            }
        }
    }
}

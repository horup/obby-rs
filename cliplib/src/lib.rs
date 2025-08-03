
use glam::Vec2;
use parry2d::{na::{Isometry2, Vector2}, query::ShapeCastOptions};

pub enum ClipBehavior {
    Clip,
    NoClip,
    Ignore
}
pub trait Body {
    fn center(&self) -> Vec2;
    fn half_extent(&self) -> f32;
    fn clip_behavior(&self) -> ClipBehavior {
        ClipBehavior::Clip
    }
}

pub enum ClipMoveResult<'a, T: Body> {
    Unhindered {
        new_pos: Vec2,
    },
    Clipped {
        other_body: &'a T,
        new_pos: Vec2,
        normal: Vec2
    },
}

fn shape<T: Body>(body: &T) -> parry2d::shape::Cuboid {
    parry2d::shape::Cuboid::new([body.half_extent(), body.half_extent()].into())
}

fn isometric2(pos: &Vec2) -> Isometry2<f32> {
    Isometry2::new(Vector2::new(pos.x, pos.y), 0.0)
}


fn vector2(pos: &Vec2) -> Vector2<f32> {
    Vector2::new(pos.x, pos.y)
}

pub fn clip_move<'a, 'b, T, F, I>(body: &'a T, vel: Vec2, other_bodies: F) -> ClipMoveResult<'b, T>
where
    T: Body,
    F: Fn() -> I,
    I: Iterator<Item = &'b T>,
{
    let mut res = ClipMoveResult::Unhindered {
        new_pos: body.center() + vel
    };
    let mut new_body_pos = body.center();
    for other_body in other_bodies() {
        let body_shape = shape(body);
        let other_body_shape = shape(other_body);

        let r = parry2d::query::cast_shapes(&isometric2(&new_body_pos), &vector2(&vel), &body_shape, &isometric2(&other_body.center()), &vector2(&Vec2::new(0.0, 0.0)), &other_body_shape, ShapeCastOptions {
            ..Default::default()
        });
        let Ok(r) = r else { 
            continue;
        };
        let Some(r) = r else {
            continue;
        };
        if r.time_of_impact <= 1.0 {
            let vv = vel * r.time_of_impact;
            let a = vel.normalize_or_zero() * 0.005;
            let n = Vec2::new(r.normal1.x, r.normal1.y);
            new_body_pos = new_body_pos + vv -a;
            res = ClipMoveResult::Clipped {
                other_body,
                new_pos: new_body_pos,
                normal:-n
            };
        }
        //dbg!(r.time_of_impact);
        /*let r = parry2d::query::contact(
            &isometric2(&new_body_pos),
            &body_shape,
            &isometric2(&other_body.center()),
            &other_body_shape,
            5.0,
        );
        let Ok(r) = r else {
            continue;
        };
        let Some(r) = r else {
            continue;
        };
        if r.dist < 0.0 {
            let alpha = 0.0;//0.05;
            let n = Vec2::new(r.normal1.x, r.normal1.y);
            new_body_pos = new_body_pos + n * (r.dist - alpha);
            res = ClipMoveResult::Clipped {
                other_body: other_body,
                new_pos: new_body_pos,
                normal:-n
            };
        }*/
    }

    res
}

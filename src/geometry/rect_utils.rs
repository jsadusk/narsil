use crate::types::*;

pub trait CenterInt {
    fn center(&self) -> Point;
}

impl CenterInt for Rect {
    fn center(&self) -> Point {
        (
            (self.max().x + self.min().x) / 2,
            (self.max().y + self.min().y) / 2,
        )
            .into()
    }
}

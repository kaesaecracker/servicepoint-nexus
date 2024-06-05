#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rect {
    pub top_left: Point,
    pub bottom_right: Point,
}

pub fn point_in_rect(point: Point, rect: Rect) -> bool {
    point.x >= rect.top_left.x
        && point.x <= rect.bottom_right.x
        && point.y >= rect.top_left.y
        && point.y <= rect.bottom_right.y
}

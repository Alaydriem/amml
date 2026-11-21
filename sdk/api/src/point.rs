use amml_abi::AmmlPoint;

/// A point in pixels, in client or back buffer coordinates; 0,0 is the top left.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl From<AmmlPoint> for Point {
    fn from(point: AmmlPoint) -> Self {
        Self { x: point.x, y: point.y }
    }
}

impl From<Point> for AmmlPoint {
    fn from(point: Point) -> Self {
        Self { x: point.x, y: point.y }
    }
}

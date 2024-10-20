

pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn into_tuple(self) -> (f32, f32, f32, f32) {
        (self.r, self.g, self.b, self.a)
    }

    pub fn into_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn into_array3(self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }

    pub fn into_tuple3(self) -> (f32, f32, f32) {
        (self.r, self.g, self.b)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }
    }
}

impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        self.r == other.r && self.g == other.g && self.b == other.b && self.a == other.a
    }
}

impl Eq for Color {}

impl std::fmt::Debug for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Color {{ r: {}, g: {}, b: {}, a: {} }}", self.r, self.g, self.b, self.a)
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Color {{ r: {}, g: {}, b: {}, a: {} }}", self.r, self.g, self.b, self.a)
    }
}

impl From<Color> for [f32; 4] {
    fn from(color: Color) -> Self {
        [color.r, color.g, color.b, color.a]
    }
}

impl From<[f32; 4]> for Color {
    fn from(color: [f32; 4]) -> Self {
        Self {
            r: color[0],
            g: color[1],
            b: color[2],
            a: color[3],
        }
    }
}

impl From<Color> for [f32; 3] {
    fn from(color: Color) -> Self {
        [color.r, color.g, color.b]
    }
}

impl From<[f32; 3]> for Color {
    fn from(color: [f32; 3]) -> Self {
        Self {
            r: color[0],
            g: color[1],
            b: color[2],
            a: 1.0,
        }
    }
}

impl From<Color> for (f32, f32, f32, f32) {
    fn from(color: Color) -> Self {
        (color.r, color.g, color.b, color.a)
    }
}

impl From<(f32, f32, f32, f32)> for Color {
    fn from(color: (f32, f32, f32, f32)) -> Self {
        Self {
            r: color.0,
            g: color.1,
            b: color.2,
            a: color.3,
        }
    }
}

impl From<Color> for (f32, f32, f32) {
    fn from(color: Color) -> Self {
        (color.r, color.g, color.b)
    }
}

impl From<(f32, f32, f32)> for Color {
    fn from(color: (f32, f32, f32)) -> Self {
        Self {
            r: color.0,
            g: color.1,
            b: color.2,
            a: 1.0,
        }
    }
}

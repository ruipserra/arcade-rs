use sdl2::rect::Rect as SdlRect;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rectangle {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rectangle {
    /// Generates an SDL rectangle representing `self`.
    /// Panics if it couldn't be created.
    pub fn to_sdl(&self) -> Option<SdlRect> {
        // The rectangle can't have negative width or height
        assert!(self.w >= 0.0 && self.h >= 0.0);

        SdlRect::new(self.x as i32, self.y as i32, self.w as u32, self.h as u32)
            .unwrap()
    }

    /// Returns an Some(Rectangle) if `self` is contained in `parent`,
    /// otherwise returns None.
    pub fn move_inside(self, parent: Rectangle) -> Option<Rectangle> {
        // `self` must be smaller thant `rectangle`
        if self.w > parent.w || self.h > parent.h {
            return None;
        }

        Some(Rectangle {
            w: self.w,
            h: self.h,
            x: if self.x < parent.x { parent.x }
               else if self.x + self.w > parent.x + parent.w { parent.x + parent.w - self.w }
               else { self.x },
            y: if self.y < parent.y { parent.y }
               else if self.y + self.h > parent.y + parent.h { parent.y + parent.h - self.h }
               else { self.y },
        })
    }

    pub fn contains(&self, rect: Rectangle) -> bool {
        let xmin = rect.x;
        let xmax = rect.x + rect.w;
        let ymin = rect.y;
        let ymax = rect.y + rect.h;

        xmin >= self.x && xmin <= self.x + self.w &&
        xmax >= self.x && xmax <= self.x + self.w &&
        ymin >= self.y && ymin <= self.y + self.h &&
        ymax >= self.y && ymax <= self.y + self.h
    }

    pub fn overlaps(&self, other: Rectangle) -> bool {
        self.x < other.x + other.w &&
        self.x + self.w > other.x &&
        self.y < other.y + other.h &&
        self.y + self.h > other.y
    }
}

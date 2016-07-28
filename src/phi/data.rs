
use sdl2::rect::Rect as SdlRect;
use sdl2::rect::Point as SdlPoint;


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rectangle {
	pub x: f64,
	pub y: f64,
	pub w: f64,
	pub h: f64,
}

impl Rectangle {

	pub fn intersection(rect_a: &Rectangle, rect_b: &Rectangle) -> Option<Rectangle> {
		let (xl, xr) = (rect_a.x.max(rect_b.x), (rect_a.x + rect_a.w).min(rect_b.x + rect_b.w));
		let (yt, yb) = (rect_a.y.max(rect_b.y), (rect_a.y + rect_a.h).min(rect_b.y + rect_b.h));
		let (wi, hi) = (xr - xl, yb - yt);

		if wi > 0.0 && hi > 0.0 {
			return Some(Rectangle {
				w: wi,
				h: hi,
				x: xl,
				y: yt
			});
		}
		None
	}


	/// Generate a rectangle with the provided size, with its top-left corner
	/// at (0, 0).
	pub fn with_size(w: f64, h: f64) -> Rectangle {
		Rectangle {
			w: w,
			h: h,
			x: 0.0,
			y: 0.0,
		}
	}


	pub fn location(&self) -> (f64, f64) {
		(self.x, self.y)
	}
	/// Centers
	pub fn center_at(&self, center: (f64, f64)) -> Rectangle {
		Rectangle {
			x: center.0 - self.w / 2.0,
			y: center.1 - self.h / 2.0,
			..*self
		}
	}

	/// Return the center of the rectangle.
	pub fn center(&self) -> (f64, f64) {
		let x = self.x + self.w / 2.0;
		let y = self.y + self.h / 2.0;
		(x, y)
	}


	/// Generates an SDL-compatible Rect equivalent to `self`.
	/// Panics if it could not be created, for example if a
	/// coordinate of a corner overflows an `i32`.
	pub fn to_sdl(&self) -> Option<SdlRect> {
		// Reject negative width and height
		assert!(self.w >= 0.0 && self.h >= 0.0);

		// SdlRect::new : `(i32, i32, u32, u32) -> Result<Option<SdlRect>>`
		Some(SdlRect::new(self.x as i32, self.y as i32, self.w as u32, self.h as u32))
	}

	/// Return a (perhaps moved) rectangle which is contained by a `parent`
	/// rectangle. If it can indeed be moved to fit, return `Some(result)`;
	/// otherwise, return `None`.
	pub fn move_inside(&self, parent: Rectangle) -> Option<Rectangle> {
		// It must be smaller than the parent rectangle to fit in it.
		if self.w <= parent.w && self.h <= parent.h {
			return 		Some(Rectangle {
				w: self.w,
				h: self.h,
				x: if self.x < parent.x { parent.x }
				else if self.x + self.w >= parent.x + parent.w { parent.x + parent.w - self.w }
				else { self.x },
				y: if self.y < parent.y { parent.y }
				else if self.y + self.h >= parent.y + parent.h { parent.y + parent.h - self.h }
				else { self.y },
			})
		}
		None
	}

	pub fn contains(&self, rect: Rectangle) -> bool {
		let xmin = rect.x;
		let xmax = xmin + rect.w;
		let ymin = rect.y;
		let ymax = ymin + rect.h;

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


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat4X4 {
	pub a00: f64, pub a01: f64, pub a02: f64, pub a03: f64,
	pub a10: f64, pub a11: f64, pub a12: f64, pub a13: f64,
	pub a20: f64, pub a21: f64, pub a22: f64, pub a23: f64,
	pub a30: f64, pub a31: f64, pub a32: f64, pub a33: f64,	
}

impl Mat4X4 {

	pub fn identity() -> Mat4X4 {
		Mat4X4 {
			a00: 1.0, a01: 0.0, a02: 0.0, a03: 0.0,
			a10: 0.0, a11: 1.0, a12: 0.0, a13: 0.0,
			a20: 0.0, a21: 0.0, a22: 1.0, a23: 0.0,
			a30: 0.0, a31: 0.0, a32: 0.0, a33: 1.0,
		}
	}

	pub fn projection(fdist: f64) -> Mat4X4 {
		Mat4X4 {
			a00: 1.0, a01: 0.0, a02: 0.0, a03: 0.0,
			a10: 0.0, a11: 1.0, a12: 0.0, a13: 0.0,
			a20: 0.0, a21: 0.0, a22: -1.0, a23: 0.0,
			a30: 0.0, a31: 0.0, a32: -1.0 / fdist, a33: 0.0,
		}
	}
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point3 {
	pub x: f64,
	pub y: f64,
	pub z: f64,
}

impl Point3 {

	pub fn new() -> Point3 {
		Point3 {
			x: 0.0,
			y: 0.0,
			z: 0.0,
		}
	}

	pub fn projected(&self, fdist: f64) -> Point3 {
		return Point3 {
			x: self.x * fdist / -self.z,
			y: self.y * fdist / -self.z,
			z: fdist
		}
	}

	pub fn remapped4(&self, mat: &Mat4X4) -> Point3 {
		let w = mat.a30 + mat.a31 + mat.a32 + mat.a33;

		Point3 {
			x: (self.x * mat.a00 + self.y * mat.a01 + self.z * mat.a02 + mat.a03) / w,
			y: (self.x * mat.a10 + self.y * mat.a11 + self.z * mat.a12 + mat.a13) / w,
			z: (self.x * mat.a20 + self.y * mat.a21 + self.z * mat.a22 + mat.a23) / w,
		}
	}


	pub fn to_sdl(&self) -> SdlPoint {
		SdlPoint::new((self.x / self.z) as i32, (self.y / self.z) as i32)
	}
}
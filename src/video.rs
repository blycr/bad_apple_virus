//! Play the existing 64x48 rectangles directly to keep the window count low.
const SOURCE_WIDTH: usize = 64;
const SOURCE_HEIGHT: usize = 48;
pub const FPS: f64 = 30.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

pub struct Video {
    pub width: i32,
    pub height: i32,
    frames: Vec<Vec<Rect>>,
}

impl Video {
    pub fn from_legacy(bytes: &[u8]) -> Result<Self, &'static str> {
        let (records, remainder) = bytes.as_chunks::<4>();
        if !remainder.is_empty() {
            return Err("incomplete box record");
        }
        let mut frames = Vec::new();
        let mut frame = Vec::new();
        let mut terminated = true;
        for c in records {
            if *c == [0, 0, 0, 0] {
                frames.push(std::mem::take(&mut frame));
                terminated = true;
            } else {
                let (x, y, w, h) = (c[0] as usize, c[1] as usize, c[2] as usize, c[3] as usize);
                if w == 0 || h == 0 || x + w > SOURCE_WIDTH || y + h > SOURCE_HEIGHT {
                    return Err("box outside the source grid");
                }
                frame.push(Rect {
                    x: x as u16,
                    y: y as u16,
                    w: w as u16,
                    h: h as u16,
                });
                terminated = false;
            }
        }
        if !terminated || frames.is_empty() {
            return Err("missing frame separator or empty video");
        }
        Ok(Self {
            width: SOURCE_WIDTH as i32,
            height: SOURCE_HEIGHT as i32,
            frames,
        })
    }

    pub fn max_windows(&self) -> usize {
        self.frames.iter().map(Vec::len).max().unwrap_or(0)
    }

    pub fn frame(&self, index: usize) -> Option<&[Rect]> {
        self.frames.get(index).map(Vec::as_slice)
    }
}

pub struct Viewport {
    left: i32,
    top: i32,
    width: i32,
    height: i32,
    grid_width: i32,
    grid_height: i32,
}

impl Viewport {
    pub fn fit(screen_width: i32, screen_height: i32, grid_width: i32, grid_height: i32) -> Self {
        let (width, height) = if screen_width as i64 * grid_height as i64
            >= screen_height as i64 * grid_width as i64
        {
            (screen_height * grid_width / grid_height, screen_height)
        } else {
            (screen_width, screen_width * grid_height / grid_width)
        };
        Self {
            left: (screen_width - width) / 2,
            top: (screen_height - height) / 2,
            width,
            height,
            grid_width,
            grid_height,
        }
    }

    pub fn map(&self, r: &Rect) -> (i32, i32, i32, i32) {
        // Round shared edges, never each rectangle's width independently.
        // This guarantees adjoining blocks meet on fractional DPI/scales.
        let x0 = (r.x as i32 * self.width + self.grid_width / 2) / self.grid_width;
        let x1 = ((r.x as i32 + r.w as i32) * self.width + self.grid_width / 2) / self.grid_width;
        let y0 = (r.y as i32 * self.height + self.grid_height / 2) / self.grid_height;
        let y1 =
            ((r.y as i32 + r.h as i32) * self.height + self.grid_height / 2) / self.grid_height;
        (self.left + x0, self.top + y0, x1 - x0, y1 - y0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_video_keeps_original_geometry_and_window_budget() {
        let bytes = include_bytes!("../assets/boxes.bin");
        let video = Video::from_legacy(bytes).unwrap();
        assert_eq!((video.width, video.height), (64, 48));
        assert_eq!(video.frames.len(), 6572);
        assert_eq!(video.max_windows(), 155);
        let mut index = 0;
        let mut raw_frame = Vec::new();
        for c in bytes.as_chunks::<4>().0 {
            if *c == [0, 0, 0, 0] {
                assert_eq!(video.frame(index).unwrap(), raw_frame.as_slice());
                raw_frame.clear();
                index += 1;
            } else {
                raw_frame.push(Rect {
                    x: c[0] as u16,
                    y: c[1] as u16,
                    w: c[2] as u16,
                    h: c[3] as u16,
                });
            }
        }
    }

    #[test]
    fn invalid_boxes_are_rejected_and_frame_count_stays_unchanged() {
        assert!(Video::from_legacy(&[0, 0, 0, 0, 1]).is_err());
        assert!(Video::from_legacy(&[63, 0, 2, 1, 0, 0, 0, 0]).is_err());
        assert!(Video::from_legacy(&[0, 0, 1, 1]).is_err());
        let video = Video::from_legacy(&[0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0]).unwrap();
        assert!(video.frame(0).unwrap().is_empty());
        assert!(!video.frame(1).unwrap().is_empty());
        assert!(video.frame(2).is_none());
    }

    #[test]
    fn fractional_scaling_has_shared_edges_and_stays_in_bounds() {
        for (sw, sh) in [(1920, 1080), (1366, 768), (1536, 864), (801, 601)] {
            let vp = Viewport::fit(sw, sh, 64, 48);
            for x in 0..63 {
                let a = vp.map(&Rect {
                    x,
                    y: 0,
                    w: 1,
                    h: 48,
                });
                let b = vp.map(&Rect {
                    x: x + 1,
                    y: 0,
                    w: 1,
                    h: 48,
                });
                assert_eq!(a.0 + a.2, b.0);
                assert!(a.0 >= 0 && a.1 >= 0 && a.0 + a.2 <= sw && a.1 + a.3 <= sh);
            }
        }
    }
}

use super::context::ChartContext;

use crate::coord::{AsRangedCoord, RangedCoord, Shift};
use crate::drawing::backend::DrawingBackend;
use crate::drawing::{DrawingArea, DrawingAreaErrorKind};
use crate::style::TextStyle;

/// The enum used to specify the position of label area.
/// This is used when we configure the label area size with the API `set_label_area_size`
pub enum LabelAreaPosition {
    Top = 0,
    Bottom = 1,
    Left = 2,
    Right = 3,
}

/// The helper object to create a chart context, which is used for the high-level figure drawing.
/// With the hlep of this object, we can convert a basic drawing area into a chart context, which
/// allows the high-level chartting API beening used on the drawing area.
pub struct ChartBuilder<'a, 'b, DB: DrawingBackend> {
    label_area_size: [i32; 4], // [upper, lower, left, right]
    label_area_inset: [bool; 4],
    root_area: &'a DrawingArea<DB, Shift>,
    title: Option<(String, TextStyle<'b>)>,
    margin: [u32; 4],
}

impl<'a, 'b, DB: DrawingBackend> ChartBuilder<'a, 'b, DB> {
    /// Create a chart builder on the given drawing area
    /// - `root`: The root drawing area
    /// - Returns: The chart builder object
    pub fn on(root: &'a DrawingArea<DB, Shift>) -> Self {
        Self {
            label_area_size: [0; 4],
            label_area_inset: [false; 4],
            root_area: root,
            title: None,
            margin: [0; 4],
        }
    }

    /// Set the margin size of the chart (applied for top, bottom, left and right at the same time)
    /// - `size`: The size of the chart margin.
    pub fn margin(&mut self, size: u32) -> &mut Self {
        self.margin = [size, size, size, size];
        self
    }

    /// Set the top margin of current chart
    /// - `size`: The size of the top margin.
    pub fn margin_top(&mut self, size: u32) -> &mut Self {
        self.margin[0] = size;
        self
    }

    /// Set the bottom margin of current chart
    /// - `size`: The size of the bottom margin.
    pub fn margin_bottom(&mut self, size: u32) -> &mut Self {
        self.margin[1] = size;
        self
    }

    /// Set the left margin of current chart
    /// - `size`: The size of the left margin.
    pub fn margin_left(&mut self, size: u32) -> &mut Self {
        self.margin[2] = size;
        self
    }

    /// Set the right margin of current chart
    /// - `size`: The size of the right margin.
    pub fn margin_right(&mut self, size: u32) -> &mut Self {
        self.margin[3] = size;
        self
    }

    /// Set the size of X label area
    /// - `size`: The height of the x label area, if x is 0, the chart doesn't have the X label area
    pub fn x_label_area_size(&mut self, size: i32) -> &mut Self {
        self.label_area_size[1] = size;
        self
    }

    pub fn inset_x_labels(&mut self) -> &mut Self {
        self.label_area_inset[1] = true;
        self
    }

    /// Set the size of the Y label area
    /// - `size`: The width of the Y label area. If size is 0, the chart doesn't have Y label area
    pub fn y_label_area_size(&mut self, size: i32) -> &mut Self {
        self.label_area_size[2] = size;
        self
    }

    pub fn inset_y_labels(&mut self) -> &mut Self {
        self.label_area_inset[2] = true;
        self
    }

    /// Set the size of X label area on the top of the chart
    /// - `size`: The height of the x label area, if x is 0, the chart doesn't have the X label area
    pub fn top_x_label_area_size(&mut self, size: i32) -> &mut Self {
        self.label_area_size[0] = size;
        self
    }

    pub fn inset_top_x_labels(&mut self) -> &mut Self {
        self.label_area_inset[0] = true;
        self
    }

    /// Set the size of the Y label area on the right side
    /// - `size`: The width of the Y label area. If size is 0, the chart doesn't have Y label area
    pub fn right_y_label_area_size(&mut self, size: i32) -> &mut Self {
        self.label_area_size[3] = size;
        self
    }

    pub fn inset_right_y_labels(&mut self) -> &mut Self {
        self.label_area_inset[3] = true;
        self
    }

    /// Set a label area size
    /// - `pos`: THe position where the label area locted
    /// - `size`: The size of the label area size
    pub fn set_label_area_size(&mut self, pos: LabelAreaPosition, size: i32) -> &mut Self {
        self.label_area_size[pos as usize] = size;
        self
    }

    /// Set the caption of the chart
    /// - `caption`: The caption of the chart
    /// - `style`: The text style
    /// - Note: If the caption is set, the margin option will be ignored
    pub fn caption<S: AsRef<str>, Style: Into<TextStyle<'b>>>(
        &mut self,
        caption: S,
        style: Style,
    ) -> &mut Self {
        self.title = Some((caption.as_ref().to_string(), style.into()));
        self
    }

    /// Build the chart with a 2D Cartesian coordinate system. The function will returns a chart
    /// context, where data series can be rendered on.
    /// - `x_spec`: The specification of X axis
    /// - `y_spec`: The specification of Y axis
    /// - Returns: A chart context
    #[allow(clippy::type_complexity)]
    pub fn build_ranged<X: AsRangedCoord, Y: AsRangedCoord>(
        &mut self,
        x_spec: X,
        y_spec: Y,
    ) -> Result<
        ChartContext<'a, DB, RangedCoord<X::CoordDescType, Y::CoordDescType>>,
        DrawingAreaErrorKind<DB::ErrorType>,
    > {
        let mut label_areas = [None, None, None, None];

        let mut drawing_area = DrawingArea::clone(self.root_area);

        if *self.margin.iter().max().unwrap_or(&0) > 0 {
            drawing_area = drawing_area.margin(
                self.margin[0] as i32,
                self.margin[1] as i32,
                self.margin[2] as i32,
                self.margin[3] as i32,
            );
        }

        if let Some((ref title, ref style)) = self.title {
            drawing_area = drawing_area.titled(title, style.clone())?;
        }

        let (w, h) = drawing_area.dim_in_pixel();

        let mut actual_drawing_area_pos = [0, h as i32, 0, w as i32];

        for (idx, (dx, dy)) in (0..4).map(|idx| (idx, [(0, -1), (0, 1), (-1, 0), (1, 0)][idx])) {
            //let size = if self.label_area_size[idx] <= 0 { 0 } else { self.label_area_size[idx] };
            let size = self.label_area_size[idx];
            let split_point = if !self.label_area_inset[idx] {
                if dx + dy < 0 {
                    size
                } else {
                    -size
                }
            } else {
                0
            };
            actual_drawing_area_pos[idx] += split_point;
        }

        let mut splitted: Vec<_> = drawing_area
            .split_by_breakpoints(
                &actual_drawing_area_pos[2..4],
                &actual_drawing_area_pos[0..2],
            )
            .into_iter()
            .map(Some)
            .collect();

        for (src_idx, dst_idx) in [1, 7, 3, 5].iter().zip(0..4) {
            let (h, w) = splitted[*src_idx].as_ref().unwrap().dim_in_pixel();
            if h > 0 && w > 0 {
                std::mem::swap(&mut label_areas[dst_idx], &mut splitted[*src_idx]);
            }
        }

        for (id, (_, size)) in self
            .label_area_inset
            .iter()
            .zip(self.label_area_size.iter())
            .enumerate()
            .filter(|(_, (inset, size))| **inset && **size != 0)
        {
            let area = splitted[4].as_ref().unwrap();
            let (w, h) = area.dim_in_pixel();
            let mut new_area = match id {
                0 => area.clone().alter_new((None, None), (None, Some(*size))),
                2 => area.clone().alter_new((None, None), (Some(*size), None)),
                1 => area.clone().alter_new(
                    (None, Some(h as i32 - *size)),
                    (None, Some(h as i32 - *size)),
                ),
                3 => area
                    .clone()
                    .alter_new((Some(w as i32 - *size), None), (None, None)),
                _ => unreachable!(),
            }
            .make_inset();
            std::mem::swap(&mut label_areas[id], &mut Some(new_area));
        }

        std::mem::swap(&mut drawing_area, &mut splitted[4].as_mut().unwrap());

        let mut pixel_range = drawing_area.get_pixel_range();
        pixel_range.1 = pixel_range.1.end..pixel_range.1.start;

        let mut x_label_area = [None, None];
        let mut y_label_area = [None, None];

        std::mem::swap(&mut x_label_area[0], &mut label_areas[0]);
        std::mem::swap(&mut x_label_area[1], &mut label_areas[1]);
        std::mem::swap(&mut y_label_area[0], &mut label_areas[2]);
        std::mem::swap(&mut y_label_area[1], &mut label_areas[3]);

        Ok(ChartContext {
            x_label_area,
            y_label_area,
            drawing_area: drawing_area.apply_coord_spec(RangedCoord::new(
                x_spec,
                y_spec,
                pixel_range,
            )),
            series_anno: vec![],
        })
    }
}

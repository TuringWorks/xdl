//! GUI-based plotting window using FLTK

use anyhow::Result;
use fltk::{button::Button, draw, enums::*, frame::Frame, prelude::*, window::Window};
use std::cell::RefCell;
use std::rc::Rc;

pub struct PlotWindow {
    window: Window,
}

struct PlotFrame {
    #[allow(dead_code)]
    frame: Frame,
    #[allow(dead_code)]
    x_data: Vec<f64>,
    #[allow(dead_code)]
    y_data: Vec<f64>,
    #[allow(dead_code)]
    title: String,
    #[allow(dead_code)]
    xtitle: String,
    #[allow(dead_code)]
    ytitle: String,
}

impl PlotFrame {
    #[allow(dead_code)]
    fn new(x: i32, y: i32, w: i32, h: i32, x_data: Vec<f64>, y_data: Vec<f64>) -> Self {
        Self::new_with_formula(x, y, w, h, x_data, y_data, "")
    }

    fn new_with_formula(
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        x_data: Vec<f64>,
        y_data: Vec<f64>,
        formula: &str,
    ) -> Self {
        Self::new_with_labels(x, y, w, h, x_data, y_data, formula, "X", "Y")
    }

    fn new_with_labels(
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        x_data: Vec<f64>,
        y_data: Vec<f64>,
        title: &str,
        xtitle: &str,
        ytitle: &str,
    ) -> Self {
        let mut frame = Frame::new(x, y, w, h, "");
        frame.set_frame(FrameType::DownBox);
        frame.set_color(Color::White);

        let plot_data = Rc::new(RefCell::new((
            x_data.clone(),
            y_data.clone(),
            title.to_string(),
            xtitle.to_string(),
            ytitle.to_string(),
        )));
        let plot_data_draw = plot_data.clone();

        frame.draw(move |f| {
            let data = plot_data_draw.borrow();
            Self::draw_plot_with_labels(f, &data.0, &data.1, &data.2, &data.3, &data.4);
        });

        Self {
            frame,
            x_data,
            y_data,
            title: title.to_string(),
            xtitle: xtitle.to_string(),
            ytitle: ytitle.to_string(),
        }
    }

    #[allow(dead_code)]
    fn draw_plot(frame: &Frame, x_data: &[f64], y_data: &[f64]) {
        Self::draw_plot_with_formula(frame, x_data, y_data, "")
    }

    fn draw_plot_with_formula(frame: &Frame, x_data: &[f64], y_data: &[f64], formula: &str) {
        Self::draw_plot_with_labels(frame, x_data, y_data, formula, "X", "Y")
    }

    fn draw_plot_with_labels(
        frame: &Frame,
        x_data: &[f64],
        y_data: &[f64],
        title: &str,
        xtitle: &str,
        ytitle: &str,
    ) {
        if x_data.is_empty() || y_data.is_empty() {
            return;
        }

        // Get frame dimensions
        let (fx, fy, fw, fh) = (frame.x(), frame.y(), frame.w(), frame.h());
        let margin = 40;
        let plot_x = fx + margin;
        let plot_y = fy + margin;
        let plot_w = fw - 2 * margin;
        let plot_h = fh - 2 * margin;

        // Find data ranges
        let x_min = x_data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let x_max = x_data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let y_min = y_data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let y_max = y_data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        // Draw axes
        draw::set_draw_color(Color::Black);
        draw::set_line_style(draw::LineStyle::Solid, 1);

        // X-axis
        draw::draw_line(plot_x, plot_y + plot_h, plot_x + plot_w, plot_y + plot_h);
        // Y-axis
        draw::draw_line(plot_x, plot_y, plot_x, plot_y + plot_h);

        // Draw plot data
        draw::set_draw_color(Color::Blue);
        draw::set_line_style(draw::LineStyle::Solid, 2);

        let mut prev_screen_x = None;
        let mut prev_screen_y = None;

        for (i, (&x, &y)) in x_data.iter().zip(y_data.iter()).enumerate() {
            // Convert data coordinates to screen coordinates
            let screen_x = plot_x + ((x - x_min) / (x_max - x_min) * plot_w as f64) as i32;
            let screen_y = plot_y + plot_h - ((y - y_min) / (y_max - y_min) * plot_h as f64) as i32;

            if let (Some(px), Some(py)) = (prev_screen_x, prev_screen_y) {
                draw::draw_line(px, py, screen_x, screen_y);
            } else if i == 0 {
                // Draw first point
                draw::draw_point(screen_x, screen_y);
            }

            prev_screen_x = Some(screen_x);
            prev_screen_y = Some(screen_y);
        }

        // Draw labels
        draw::set_draw_color(Color::Black);
        draw::set_font(Font::Helvetica, 12);

        // Title
        let display_title = if title.is_empty() { "XDL Plot" } else { title };
        let (title_w, title_h) = draw::measure(display_title, false);
        draw::draw_text2(
            display_title,
            fx + (fw - title_w) / 2,
            fy + 20,
            title_w,
            title_h,
            Align::Center,
        );

        // Axis labels
        let x_label = if xtitle.is_empty() {
            format!("X: {:.2} to {:.2}", x_min, x_max)
        } else {
            xtitle.to_string()
        };
        let y_label = if ytitle.is_empty() {
            format!("Y: {:.2} to {:.2}", y_min, y_max)
        } else {
            ytitle.to_string()
        };

        draw::draw_text2(
            &x_label,
            plot_x,
            plot_y + plot_h + 15,
            plot_w,
            15,
            Align::Left,
        );

        // Rotate and draw Y label (simplified - just draw at side)
        draw::draw_text2(&y_label, fx + 5, plot_y, 30, plot_h, Align::Left);
    }
}

impl PlotWindow {
    pub fn with_labels(
        x_data: Vec<f64>,
        y_data: Vec<f64>,
        title: &str,
        xtitle: &str,
        ytitle: &str,
        formula: &str,
    ) -> Result<Self> {
        let mut window = Window::new(200, 200, 700, 500, title);
        window.set_color(Color::from_rgb(240, 240, 240));

        // Create the plot frame that will handle drawing
        let plot_title = if !formula.is_empty() { formula } else { title };
        let _plot_frame = PlotFrame::new_with_labels(
            10,
            10,
            680,
            420,
            x_data.clone(),
            y_data.clone(),
            plot_title,
            xtitle,
            ytitle,
        );

        // Info button at bottom
        let mut info_btn = Button::new(300, 450, 100, 30, "Plot Info");
        info_btn.set_color(Color::from_rgb(70, 130, 180));
        info_btn.set_label_color(Color::White);

        let data_info =
            format!(
            "Plot data: {} points\nX range: {:.3} to {:.3}\nY range: {:.3} to {:.3}\n\nFormula: {}",
            x_data.len(),
            x_data.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
            x_data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
            y_data.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
            y_data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
            if formula.is_empty() { "Dynamic data" } else { formula }
        );

        info_btn.set_callback(move |_| {
            fltk::dialog::message_default(&data_info);
        });

        window.end();
        window.make_resizable(true);

        // Set up close callback to prevent issues when window closes
        window.set_callback(move |win| {
            if fltk::app::event() == fltk::enums::Event::Close {
                win.hide();
            }
        });

        Ok(Self { window })
    }

    pub fn show(&mut self) {
        self.window.show();
    }
}

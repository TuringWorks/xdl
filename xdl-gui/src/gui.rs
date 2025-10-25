//! Main GUI implementation using FLTK

use anyhow::Result;
use fltk::draw;
use fltk::{
    app,
    button::Button,
    dialog,
    enums::{Color, Font, FrameType},
    group::{Flex, FlexType, Pack},
    menu::{MenuBar, MenuFlag},
    prelude::*,
    table::{Table, TableContext},
    text::{TextBuffer, TextDisplay, TextEditor},
    window::Window,
};
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use tracing::{error, info};

use crate::image_window::ImageWindow;
use crate::plot_window::PlotWindow;

// Global queue for pending plot windows to show after execution
static PENDING_PLOT_WINDOWS: Lazy<Mutex<Vec<PlotWindow>>> = Lazy::new(|| Mutex::new(Vec::new()));

// Structure to hold execution results from worker thread
struct ExecutionResult {
    output_text: String,
    variables: HashMap<String, String>,
}
use xdl_interpreter::Interpreter;
use xdl_stdlib::{register_gui_image_callback, register_gui_plot_callback};

// Variable data structure for table display
#[derive(Clone)]
struct VarData {
    name: String,
    value: String,
    var_type: String,
    size: String,
}

// Custom table widget for variables
struct VariableTable {
    table: Table,
    data: Rc<RefCell<Vec<VarData>>>,
}

impl VariableTable {
    fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        let mut table = Table::default().with_pos(x, y).with_size(w, h);
        table.set_rows(0);
        table.set_row_height_all(22);
        table.set_row_header(false);
        table.set_cols(4);
        table.set_col_header(true);
        table.set_col_header_height(25);

        // Set individual column widths for better layout
        table.set_col_width(0, w / 5); // Name column
        table.set_col_width(1, (w * 2) / 5); // Value column (wider)
        table.set_col_width(2, w / 6); // Type column
        table.set_col_width(3, w / 6); // Size column

        table.set_col_resize(true);
        table.end();

        let data: Rc<RefCell<Vec<VarData>>> = Rc::new(RefCell::new(Vec::new()));
        let data_clone = data.clone();

        table.draw_cell(move |_t, ctx, row, col, x, y, w, h| {
            match ctx {
                TableContext::StartPage => {
                    draw::set_font(Font::Helvetica, 10);
                }
                TableContext::ColHeader => {
                    draw::push_clip(x, y, w, h);
                    draw::draw_box(
                        FrameType::ThinUpBox,
                        x,
                        y,
                        w,
                        h,
                        Color::from_rgb(230, 230, 230),
                    );
                    draw::set_draw_color(Color::Black);
                    draw::set_font(Font::HelveticaBold, 11);
                    let label = match col {
                        0 => "Name",
                        1 => "Value",
                        2 => "Type",
                        3 => "Size",
                        _ => "",
                    };
                    draw::draw_text2(label, x, y, w, h, fltk::enums::Align::Center);
                    draw::pop_clip();
                }
                TableContext::Cell => {
                    if let Ok(data) = data_clone.try_borrow() {
                        if row >= 0 && (row as usize) < data.len() {
                            draw::push_clip(x, y, w, h);

                            // Alternating row colors
                            let bg_color = if row % 2 == 0 {
                                Color::White
                            } else {
                                Color::from_rgb(248, 248, 252)
                            };
                            draw::draw_box(FrameType::FlatBox, x, y, w, h, bg_color);

                            // Draw text
                            draw::set_draw_color(Color::Black);
                            draw::set_font(Font::Courier, 10);

                            let var = &data[row as usize];
                            let text = match col {
                                0 => &var.name,
                                1 => &var.value,
                                2 => &var.var_type,
                                3 => &var.size,
                                _ => "",
                            };

                            // Draw text with padding
                            draw::draw_text2(
                                text,
                                x + 4,
                                y,
                                w - 8,
                                h,
                                fltk::enums::Align::Left | fltk::enums::Align::Inside,
                            );

                            // Draw cell border
                            draw::set_draw_color(Color::from_rgb(220, 220, 220));
                            draw::draw_rect(x, y, w, h);

                            draw::pop_clip();
                        }
                    }
                }
                _ => {}
            }
        });

        Self { table, data }
    }

    fn update_data(&mut self, vars: &HashMap<String, String>) {
        let mut data = Vec::new();

        // Sort variable names for consistent display
        let mut sorted_vars: Vec<(&String, &String)> = vars.iter().collect();
        sorted_vars.sort_by_key(|(name, _)| name.as_str());

        for (name, value) in sorted_vars {
            let var_type = if value.contains("array") {
                "Double Array".to_string()
            } else if value.parse::<f64>().is_ok() {
                "Double".to_string()
            } else {
                "Computed".to_string()
            };

            let display_value = if value.contains("array[1x") {
                if let Some(start) = value.find("[1x") {
                    if let Some(end) = value[start..].find(']') {
                        format!("<{}>", &value[start + 1..start + end])
                    } else {
                        "<array>".to_string()
                    }
                } else {
                    "<array>".to_string()
                }
            } else if value.len() > 20 {
                format!("{:.17}...", value)
            } else {
                value.clone()
            };

            let size = if value.contains("1x") {
                if let Some(start) = value.find("1x") {
                    if let Some(end) = value[start..].find(']') {
                        value[start..start + end].to_string()
                    } else {
                        "1x?".to_string()
                    }
                } else {
                    "1x1".to_string()
                }
            } else {
                "1x1".to_string()
            };

            data.push(VarData {
                name: name.clone(),
                value: display_value,
                var_type,
                size,
            });
        }

        if let Ok(mut d) = self.data.try_borrow_mut() {
            *d = data;
        }

        let new_rows = vars.len() as i32;
        if new_rows != self.table.rows() {
            self.table.set_rows(new_rows);
        }

        // Force complete redraw
        self.table.redraw();
    }
}

pub struct XdlGui {
    window: Window,
    #[allow(dead_code)]
    interpreter: Rc<RefCell<Interpreter>>,
    #[allow(dead_code)]
    command_buffer: TextBuffer,
    #[allow(dead_code)]
    output_buffer: TextBuffer,
    #[allow(dead_code)]
    variables: Rc<RefCell<HashMap<String, String>>>, // variable name -> value representation
    #[allow(dead_code)]
    variable_table: Rc<RefCell<VariableTable>>,
    #[allow(dead_code)]
    executing: Rc<RefCell<bool>>, // Guard to prevent concurrent executions
}

impl XdlGui {
    pub fn new() -> Result<Self> {
        // Set environment variable to indicate GUI mode (prevents blocking 3D windows)
        std::env::set_var("XDL_GUI_MODE", "1");

        // Create main window with proportions suitable for a scientific computing environment
        let mut window = Window::new(
            100,
            100,
            1200,
            800,
            "XDL GUI - Scientific Computing Environment",
        );
        window.set_color(Color::from_rgb(236, 233, 216));

        // Initialize interpreter
        let interpreter = Rc::new(RefCell::new(Interpreter::new()));

        // Create command buffer early so it can be used in menu callbacks
        let mut command_buffer = TextBuffer::default();
        command_buffer.set_text("// XDL Command Window - Load a .xdl file or type commands below\n// Example:\n// t0 = 0\n// y0 = 10\n// d = 0.85\n// plot(t,y)\n");

        let mut output_buffer = TextBuffer::default();
        output_buffer.set_text("Command Output Window\n\nReady to execute XDL commands.\n");

        // Create comprehensive menu bar
        let mut menu = MenuBar::new(0, 0, 1200, 25, "");

        // File Menu
        let mut cmd_buffer_menu_new = command_buffer.clone();
        let mut out_buffer_menu_new = output_buffer.clone();
        menu.add(
            "&File/&New",
            fltk::enums::Shortcut::Ctrl | 'n',
            MenuFlag::Normal,
            move |_| {
                cmd_buffer_menu_new
                    .set_text("// New XDL Script\n// Enter your XDL commands here\n\n");
                out_buffer_menu_new.set_text("New script created. Start typing your XDL code.\n");
                info!("New script created");
            },
        );

        // File Open callback with access to command buffer
        let mut cmd_buffer_for_menu = command_buffer.clone();
        let mut out_buffer_for_menu = output_buffer.clone();
        menu.add(
            "&File/&Open...",
            fltk::enums::Shortcut::Ctrl | 'o',
            MenuFlag::Normal,
            move |_| {
                if let Some(path) =
                    dialog::file_chooser("Open XDL or MATLAB File", "*.{xdl,m}", ".", false)
                {
                    info!("Menu: Opening file: {}", path);

                    match std::fs::read_to_string(&path) {
                        Ok(content) => {
                            let filename = std::path::Path::new(&path)
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy();

                            // Check if this is a MATLAB .m file
                            let file_path = std::path::Path::new(&path);
                            let display_content =
                                if file_path.extension().and_then(|s| s.to_str()) == Some("m") {
                                    info!("Detected MATLAB .m file, transpiling to XDL");
                                    match xdl_matlab::transpile_matlab_to_xdl(&content) {
                                        Ok(xdl_code) => xdl_code,
                                        Err(e) => {
                                            let error_msg = format!(
                                                "Error transpiling MATLAB file {}: {}",
                                                path, e
                                            );
                                            out_buffer_for_menu.set_text(&error_msg);
                                            error!("Failed to transpile MATLAB file: {}", e);
                                            return;
                                        }
                                    }
                                } else {
                                    content
                                };

                            cmd_buffer_for_menu.set_text(&format!(
                                "// File: {}\n{}\n\n// Click Execute to run this code",
                                filename, display_content
                            ));
                            out_buffer_for_menu.set_text(&format!(
                                "Loaded file: {}\nClick Execute button to run the code.\n",
                                path
                            ));
                        }
                        Err(e) => {
                            let error_msg = format!("Error reading file {}: {}", path, e);
                            out_buffer_for_menu.set_text(&error_msg);
                            error!("Failed to read file: {}", e);
                        }
                    }
                }
            },
        );
        // File Save
        let cmd_buffer_save = command_buffer.clone();
        let mut out_buffer_save = output_buffer.clone();
        menu.add(
            "&File/&Save",
            fltk::enums::Shortcut::Ctrl | 's',
            MenuFlag::Normal,
            move |_| {
                if let Some(path) = dialog::file_chooser("Save XDL File", "*.xdl", ".", true) {
                    let content = cmd_buffer_save.text();
                    match std::fs::write(&path, content) {
                        Ok(_) => {
                            out_buffer_save
                                .set_text(&format!("File saved successfully: {}\n", path));
                            info!("File saved: {}", path);
                        }
                        Err(e) => {
                            out_buffer_save.set_text(&format!("Error saving file: {}\n", e));
                            error!("Failed to save file: {}", e);
                        }
                    }
                } else {
                    out_buffer_save.set_text("Save operation cancelled.\n");
                }
            },
        );

        // File Save As
        let cmd_buffer_save_as = command_buffer.clone();
        let mut out_buffer_save_as = output_buffer.clone();
        menu.add(
            "&File/Save &As...",
            fltk::enums::Shortcut::Ctrl | fltk::enums::Shortcut::Shift | 's',
            MenuFlag::Normal,
            move |_| {
                if let Some(path) = dialog::file_chooser("Save XDL File As", "*.xdl", ".", true) {
                    let content = cmd_buffer_save_as.text();
                    match std::fs::write(&path, content) {
                        Ok(_) => {
                            out_buffer_save_as.set_text(&format!("File saved as: {}\n", path));
                            info!("File saved as: {}", path);
                        }
                        Err(e) => {
                            out_buffer_save_as.set_text(&format!("Error saving file: {}\n", e));
                            error!("Failed to save file as: {}", e);
                        }
                    }
                } else {
                    out_buffer_save_as.set_text("Save As operation cancelled.\n");
                }
            },
        );

        menu.add(
            "&File/-",
            fltk::enums::Shortcut::None,
            MenuFlag::MenuDivider,
            |_| {},
        );

        // File Recent Files (placeholder)
        menu.add(
            "&File/&Recent Files",
            fltk::enums::Shortcut::None,
            MenuFlag::Normal,
            |_| {
                dialog::message_default("Recent files functionality not yet implemented.\nUse File > Open to browse for XDL files.");
            },
        );

        menu.add(
            "&File/-",
            fltk::enums::Shortcut::None,
            MenuFlag::MenuDivider,
            |_| {},
        );

        menu.add(
            "&File/&Quit",
            fltk::enums::Shortcut::Ctrl | 'q',
            MenuFlag::Normal,
            |_| {
                info!("Application quit requested");
                app::quit();
            },
        );
        // Edit menu items
        menu.add(
            "&Edit/&Undo",
            fltk::enums::Shortcut::Ctrl | 'z',
            MenuFlag::Normal,
            |_| {
                info!("Undo functionality not yet implemented");
                dialog::message_default("Undo functionality is not yet available.");
            },
        );

        menu.add(
            "&Edit/&Redo",
            fltk::enums::Shortcut::Ctrl | 'y',
            MenuFlag::Normal,
            |_| {
                info!("Redo functionality not yet implemented");
                dialog::message_default("Redo functionality is not yet available.");
            },
        );

        menu.add(
            "&Edit/-",
            fltk::enums::Shortcut::None,
            MenuFlag::MenuDivider,
            |_| {},
        );

        menu.add(
            "&Edit/Cu&t",
            fltk::enums::Shortcut::Ctrl | 'x',
            MenuFlag::Normal,
            |_| {
                info!("Cut functionality - use standard Ctrl+X in text fields");
                dialog::message_default("Use Ctrl+X to cut text in the command window.");
            },
        );

        menu.add(
            "&Edit/&Copy",
            fltk::enums::Shortcut::Ctrl | 'c',
            MenuFlag::Normal,
            |_| {
                info!("Copy functionality - use standard Ctrl+C in text fields");
                dialog::message_default("Use Ctrl+C to copy text in the command window.");
            },
        );

        menu.add(
            "&Edit/&Paste",
            fltk::enums::Shortcut::Ctrl | 'v',
            MenuFlag::Normal,
            |_| {
                info!("Paste functionality - use standard Ctrl+V in text fields");
                dialog::message_default("Use Ctrl+V to paste text in the command window.");
            },
        );

        menu.add(
            "&Edit/-",
            fltk::enums::Shortcut::None,
            MenuFlag::MenuDivider,
            |_| {},
        );

        menu.add(
            "&Edit/Select &All",
            fltk::enums::Shortcut::Ctrl | 'a',
            MenuFlag::Normal,
            |_| {
                info!("Select All functionality - use standard Ctrl+A in text fields");
                dialog::message_default("Use Ctrl+A to select all text in the command window.");
            },
        );

        menu.add(
            "&Edit/&Find",
            fltk::enums::Shortcut::Ctrl | 'f',
            MenuFlag::Normal,
            |_| {
                info!("Find functionality not yet implemented");
                dialog::message_default("Find functionality is not yet available.");
            },
        );

        menu.add(
            "&Edit/Find && &Replace",
            fltk::enums::Shortcut::Ctrl | 'h',
            MenuFlag::Normal,
            |_| {
                info!("Find and Replace functionality not yet implemented");
                dialog::message_default("Find and Replace functionality is not yet available.");
            },
        );

        // View Menu (basic items without variable references)
        let mut output_buffer_clear = output_buffer.clone();
        menu.add(
            "&View/&Clear Output",
            fltk::enums::Shortcut::None,
            MenuFlag::Normal,
            move |_| {
                output_buffer_clear.set_text("Output cleared.\n");
                info!("Output window cleared");
            },
        );

        menu.add(
            "&View/&Refresh",
            fltk::enums::Shortcut::None, // Use None instead of F5 for now
            MenuFlag::Normal,
            |_| {
                info!("Refresh requested - variables and display updated");
                dialog::message_default("Interface refreshed.");
            },
        );

        // Control Menu
        menu.add(
            "&Control/&Execute Script",
            fltk::enums::Shortcut::Ctrl | 'e',
            MenuFlag::Normal,
            |_| {
                info!("Execute menu item clicked - use Execute button in control panel");
                dialog::message_default(
                    "Use the Execute button in the control panel to run your XDL script.",
                );
            },
        );

        menu.add(
            "&Control/&Stop Execution",
            fltk::enums::Shortcut::Ctrl | 'k',
            MenuFlag::Normal,
            |_| {
                info!("Stop execution requested");
                dialog::message_default("Stop execution functionality not yet implemented.");
            },
        );

        menu.add(
            "&Control/-",
            fltk::enums::Shortcut::None,
            MenuFlag::MenuDivider,
            |_| {},
        );

        let mut cmd_buf_clear = command_buffer.clone();
        menu.add(
            "&Control/&Clear Command Window",
            fltk::enums::Shortcut::None,
            MenuFlag::Normal,
            move |_| {
                cmd_buf_clear.set_text("// XDL Command Window\n// Enter your commands here\n\n");
                info!("Command window cleared");
            },
        );

        // Tools Menu
        menu.add(
            "&Tools/&Variable Browser",
            fltk::enums::Shortcut::None,
            MenuFlag::Normal,
            |_| {
                info!("Variable browser is already visible in right panel");
                dialog::message_default("The Variable Browser is visible in the right panel.\nIt updates automatically when you execute XDL scripts.");
            },
        );

        menu.add(
            "&Tools/&Plot Window",
            fltk::enums::Shortcut::None,
            MenuFlag::Normal,
            |_| {
                info!("Plot window info requested");
                dialog::message_default("Plot windows open automatically when you use plot commands in your XDL scripts.\n\nExample: plot, x, y");
            },
        );

        menu.add(
            "&Tools/-",
            fltk::enums::Shortcut::None,
            MenuFlag::MenuDivider,
            |_| {},
        );

        menu.add(
            "&Tools/&Options",
            fltk::enums::Shortcut::None,
            MenuFlag::Normal,
            |_| {
                info!("Options dialog requested");
                dialog::message_default("Options and preferences dialog is not yet implemented.");
            },
        );

        // Applications Menu
        menu.add(
            "&Applications/&Text Editor",
            fltk::enums::Shortcut::None,
            MenuFlag::Normal,
            |_| {
                info!("External text editor not implemented");
                dialog::message_default("Use the built-in command window for editing XDL scripts.\nExternal editor integration is not yet available.");
            },
        );

        menu.add(
            "&Applications/&Calculator",
            fltk::enums::Shortcut::None,
            MenuFlag::Normal,
            |_| {
                info!("Calculator functionality requested");
                dialog::message_default("Built-in calculator is not yet available.\nYou can perform calculations directly in the XDL command window.");
            },
        );

        // Help Menu
        menu.add(
            "&Help/&Help Topics",
            fltk::enums::Shortcut::None, // Use None instead of F1 for now
            MenuFlag::Normal,
            |_| {
                let help_text = "XDL GUI Help Topics:\n\n\
                    Getting Started:\n\
                    • Use File > Open to load .xdl or .m (MATLAB) files\n\
                    • Type commands directly in the command window\n\
                    • Click Execute to run your scripts\n\n\
                    Features:\n\
                    • Variable tracking in the right panel\n\
                    • Automatic plot window generation\n\
                    • Script editing with syntax awareness\n\
                    • MATLAB .m file support (auto-transpiles to XDL)\n\n\
                    Keyboard Shortcuts:\n\
                    • Ctrl+N: New script\n\
                    • Ctrl+O: Open file\n\
                    • Ctrl+S: Save file\n\
                    • Ctrl+E: Execute script\n\
                    • F1: This help\n\
                    • F5: Refresh";
                dialog::message_default(help_text);
            },
        );

        menu.add(
            "&Help/&Keyboard Shortcuts",
            fltk::enums::Shortcut::None,
            MenuFlag::Normal,
            |_| {
                let shortcuts_text = "Keyboard Shortcuts:\n\n\
                    File Operations:\n\
                    • Ctrl+N: New script\n\
                    • Ctrl+O: Open file\n\
                    • Ctrl+S: Save file\n\
                    • Ctrl+Shift+S: Save As\n\
                    • Ctrl+Q: Quit\n\n\
                    Editing:\n\
                    • Ctrl+Z: Undo (in text fields)\n\
                    • Ctrl+Y: Redo (in text fields)\n\
                    • Ctrl+X: Cut\n\
                    • Ctrl+C: Copy\n\
                    • Ctrl+V: Paste\n\
                    • Ctrl+A: Select All\n\n\
                    Execution:\n\
                    • Ctrl+E: Execute script\n\
                    • Ctrl+K: Stop execution\n\n\
                    Other:\n\
                    • F1: Help\n\
                    • F5: Refresh";
                dialog::message_default(shortcuts_text);
            },
        );

        menu.add(
            "&Help/-",
            fltk::enums::Shortcut::None,
            MenuFlag::MenuDivider,
            |_| {},
        );

        menu.add(
            "&Help/&About XDL GUI",
            fltk::enums::Shortcut::None,
            MenuFlag::Normal,
            |_| {
                let about_text = "XDL GUI v1.0\n\n\
                    A Scientific Computing Environment\n\
                    for XDL (eXtended Data Language)\n\n\
                    Features:\n\
                    • Interactive script editing\n\
                    • Variable tracking and visualization\n\
                    • Integrated plotting capabilities\n\
                    Built with Rust and FLTK\n\
                    Copyright © 2024";
                dialog::message_default(about_text);
            },
        );

        // Main two-panel layout (center and right)
        let mut main_flex = Flex::new(0, 25, 1200, 775, "");
        main_flex.set_type(FlexType::Row);
        main_flex.set_spacing(2);

        // Center panel - Split into command input (top) and output (bottom)
        let mut center_flex = Flex::default();
        center_flex.set_type(FlexType::Column);

        // Top: Command input window (using Flex instead of Pack)
        let mut command_flex = Flex::default();
        command_flex.set_type(FlexType::Column);
        command_flex.set_label("XDL Commands");
        command_flex.set_frame(FrameType::DownBox);
        command_flex.set_color(Color::White);

        // Command editor (uses the early-created buffer)
        let mut command_editor = TextEditor::default();
        command_editor.set_buffer(command_buffer.clone());
        command_editor.set_text_size(12);
        command_editor.set_text_font(fltk::enums::Font::Courier);
        command_editor.set_color(Color::White);

        // Execute button and controls
        let mut control_pack = Pack::default().with_size(0, 30);
        control_pack.set_type(fltk::group::PackType::Horizontal);

        let mut execute_btn = Button::default().with_size(80, 25).with_label("Execute");
        execute_btn.set_color(Color::from_rgb(70, 130, 180));
        execute_btn.set_label_color(Color::White);

        let mut cancel_btn = Button::default().with_size(70, 25).with_label("Cancel");
        cancel_btn.set_color(Color::from_rgb(220, 80, 80));
        cancel_btn.set_label_color(Color::White);
        cancel_btn.deactivate(); // Initially disabled

        let mut clear_btn = Button::default().with_size(60, 25).with_label("Clear");
        clear_btn.set_color(Color::from_rgb(160, 160, 160));

        control_pack.end();
        command_flex.fixed(&control_pack, 30);
        command_flex.end();

        // Bottom: Command output/results window
        let mut output_display = TextDisplay::default();
        output_display.set_label("Command Results");
        output_display.set_frame(FrameType::DownBox);
        output_display.set_color(Color::White);

        // Use the early-created output buffer
        output_display.set_buffer(output_buffer.clone());
        output_display.set_text_size(10);
        output_display.set_text_font(fltk::enums::Font::Courier);

        // Set equal sizes for command and output windows
        center_flex.end();

        // Right panel - Variable Table (dynamic)
        let mut var_table = VariableTable::new(0, 0, 300, 775);
        var_table.table.set_label("Variable Browser");
        var_table.table.set_frame(FrameType::DownBox);

        // Create variable storage
        let variables = Rc::new(RefCell::new(HashMap::new()));

        // Create execution guard to prevent concurrent/repeated executions
        let executing = Rc::new(RefCell::new(false));

        // Create cancellation flag shared between main thread and worker thread
        let cancel_flag = Arc::new(AtomicBool::new(false));

        // Set panel sizes for two-panel layout
        main_flex.fixed(&var_table.table, 300); // Give space to variable table

        // Store references for callbacks after layout is done
        let var_table_ref = Rc::new(RefCell::new(var_table));

        // Add Clear Variables menu item now that variables are available
        let vars_clone_menu = Rc::clone(&variables);
        let var_table_clone_menu = Rc::clone(&var_table_ref);
        menu.add(
            "&View/Clear &Variables",
            fltk::enums::Shortcut::None,
            MenuFlag::Normal,
            move |_| {
                if let Ok(mut vars) = vars_clone_menu.try_borrow_mut() {
                    vars.clear();
                    Self::update_variable_table(&var_table_clone_menu, &vars_clone_menu);
                    info!("Variables cleared");
                }
            },
        );

        window.end();

        // Set up execute button callback with async execution
        let cmd_buffer_clone_exec = command_buffer.clone();
        let mut out_buffer_clone_exec = output_buffer.clone();
        let vars_clone = Rc::clone(&variables);
        let var_table_clone = Rc::clone(&var_table_ref);
        let executing_clone = Rc::clone(&executing);
        let execute_btn_clone = execute_btn.clone();
        let mut cancel_btn_for_exec = cancel_btn.clone();
        let cancel_flag_for_exec = Arc::clone(&cancel_flag);

        execute_btn.set_callback(move |btn| {
            info!("=== EXECUTE BUTTON CLICKED ===");

            // Clear any pending plot windows from previous executions
            if let Ok(mut plots) = PENDING_PLOT_WINDOWS.lock() {
                let old_count = plots.len();
                plots.clear();
                if old_count > 0 {
                    info!(
                        "Cleared {} old plot windows from previous execution",
                        old_count
                    );
                }
            }

            // Check if already executing - prevent concurrent executions
            if let Ok(mut exec_guard) = executing_clone.try_borrow_mut() {
                if *exec_guard {
                    info!("Already executing - ignoring duplicate execution request");
                    return;
                }
                *exec_guard = true;
            } else {
                info!("Cannot acquire execution lock - skipping execution");
                return;
            }

            // Reset cancel flag
            cancel_flag_for_exec.store(false, Ordering::Relaxed);

            // Update UI to show execution in progress
            btn.set_label("Executing...");
            btn.deactivate();
            cancel_btn_for_exec.activate(); // Enable cancel button
            out_buffer_clone_exec.set_text("Executing script...\n");
            app::awake();

            let code = cmd_buffer_clone_exec.text();
            let filtered_code = code
                .lines()
                .filter(|line| !line.trim().starts_with("//") && !line.trim().is_empty())
                .collect::<Vec<_>>()
                .join("\n");

            if !filtered_code.trim().is_empty() {
                // Simple strategy: try to execute as XDL first
                // If it has MATLAB patterns, try transpiling
                let code_to_execute = if Self::looks_like_matlab(&filtered_code) {
                    // Try MATLAB transpilation
                    match xdl_matlab::transpile_matlab_to_xdl(&filtered_code) {
                        Ok(xdl_code) => {
                            info!("Transpiled MATLAB code to XDL");
                            xdl_code
                        }
                        Err(_) => {
                            // Transpilation failed, might already be XDL
                            info!("MATLAB transpilation failed, trying as XDL");
                            filtered_code.clone()
                        }
                    }
                } else {
                    // Doesn't look like MATLAB, use as-is
                    filtered_code.clone()
                };

                // Create channel for receiving results from worker thread
                let (tx, rx) = mpsc::channel();
                let mut out_buffer_for_thread = out_buffer_clone_exec.clone();
                let vars_for_thread = Rc::clone(&vars_clone);
                let var_table_for_thread = Rc::clone(&var_table_clone);
                let executing_for_thread = Rc::clone(&executing_clone);
                let mut btn_for_thread = execute_btn_clone.clone();
                let mut cancel_btn_for_result = cancel_btn_for_exec.clone();
                let cancel_flag_for_thread = Arc::clone(&cancel_flag_for_exec);
                let cancel_flag_for_timeout = Arc::clone(&cancel_flag_for_exec);

                // Spawn worker thread to execute XDL code
                thread::spawn(move || {
                    let result = Self::execute_xdl_code_async(
                        &code_to_execute,
                        "Editor",
                        cancel_flag_for_thread,
                    );
                    tx.send(result).ok();
                    // Wake up the main thread to check for results
                    app::awake();
                });

                // Set up periodic check for results using timeout
                app::add_timeout3(0.1, move |handle| {
                    if let Ok(result) = rx.try_recv() {
                        // Check if cancellation happened - if so, don't update UI
                        if cancel_flag_for_timeout.load(Ordering::Relaxed) {
                            info!("Thread completed after cancellation - ignoring results");
                            return; // Don't update UI, user already saw cancellation message
                        }

                        // Update output buffer
                        out_buffer_for_thread.set_text(&result.output_text);

                        // Update variables
                        if let Ok(mut vars) = vars_for_thread.try_borrow_mut() {
                            *vars = result.variables;
                        }

                        // Update variable table
                        Self::update_variable_table(&var_table_for_thread, &vars_for_thread);

                        // Release execution lock (may already be released by cancel)
                        if let Ok(mut exec_guard) = executing_for_thread.try_borrow_mut() {
                            if *exec_guard {
                                *exec_guard = false;
                                info!("=== EXECUTION COMPLETED ===");
                            }
                        }

                        // Restore button state
                        btn_for_thread.set_label("Execute");
                        btn_for_thread.activate();
                        cancel_btn_for_result.deactivate();

                        // Show all queued plot windows
                        if let Ok(mut plots) = PENDING_PLOT_WINDOWS.lock() {
                            info!("Showing {} queued plot windows...", plots.len());
                            for mut plot_win in plots.drain(..) {
                                plot_win.show();
                            }
                            info!("All plot windows shown");
                        }
                        // Callback completes - no need to repeat
                    } else {
                        // Keep polling - schedule next check
                        app::repeat_timeout3(0.1, handle);
                    }
                });
            } else {
                // No code to execute, release lock and restore button
                if let Ok(mut exec_guard) = executing_clone.try_borrow_mut() {
                    *exec_guard = false;
                }
                btn.set_label("Execute");
                btn.activate();
                cancel_btn_for_exec.deactivate();
            }
        });

        // Set up cancel button callback
        let mut out_buffer_for_cancel = output_buffer.clone();
        let cancel_flag_for_cancel = Arc::clone(&cancel_flag);
        let executing_for_cancel = Rc::clone(&executing);
        let mut execute_btn_for_cancel = execute_btn.clone();

        cancel_btn.set_callback(move |btn| {
            info!("=== CANCEL BUTTON CLICKED ===");

            // Set the cancellation flag (thread will check this and stop early)
            cancel_flag_for_cancel.store(true, Ordering::Relaxed);

            // Immediately reset UI state - don't wait for thread to finish
            out_buffer_for_cancel.set_text("✗ Execution cancelled by user\n");

            // Restore button states immediately
            execute_btn_for_cancel.set_label("Execute");
            execute_btn_for_cancel.activate();
            btn.deactivate();

            // Release execution lock immediately
            if let Ok(mut exec_guard) = executing_for_cancel.try_borrow_mut() {
                *exec_guard = false;
                info!("Execution lock released after cancellation");
            }

            info!("=== CANCELLATION COMPLETE (UI restored) ===");
        });

        // Set up clear button callback
        let mut cmd_buffer_clone_clear = command_buffer.clone();
        let mut out_buffer_clone_clear = output_buffer.clone();

        clear_btn.set_callback(move |_| {
            cmd_buffer_clone_clear.set_text("// XDL Command Window - Load a .xdl file or type commands below\n// Example:\n// t0 = 0\n// y0 = 10\n// d = 0.85\n// plot(t,y)\n");
            out_buffer_clone_clear.set_text("Output cleared.\n");
        });

        // Register plot callback (done once per GUI instance)
        // IMPORTANT: Don't show windows immediately - queue them to show after execution
        register_gui_plot_callback(move |x_data, y_data, title, xtitle, ytitle| {
            info!("Plot callback: Creating plot window for '{}'", title);

            match PlotWindow::with_labels(x_data, y_data, &title, &xtitle, &ytitle, "") {
                Ok(plot_win) => {
                    // Queue the window to be shown later using global queue
                    if let Ok(mut plots) = PENDING_PLOT_WINDOWS.lock() {
                        plots.push(plot_win);
                        info!("Plot window queued (total queued: {})", plots.len());
                    }
                }
                Err(e) => eprintln!("Plot error: {}", e),
            }
        });

        // Register image display callback for 3D plots
        register_gui_image_callback(move |image_path, title| {
            match ImageWindow::new(&image_path, &title) {
                Ok(mut img_win) => img_win.show(),
                Err(e) => eprintln!("Image display error: {}", e),
            }
        });

        let gui = Self {
            window,
            interpreter,
            command_buffer,
            output_buffer,
            variables,
            variable_table: var_table_ref,
            executing,
        };

        Ok(gui)
    }

    // Helper methods for file handling and XDL execution

    #[allow(dead_code)]
    fn handle_file_selection(
        selected_text: &str,
        current_dir: &std::path::Path,
        cmd_buffer: &mut TextBuffer,
        out_buffer: &mut TextBuffer,
        _interpreter: &Rc<RefCell<Interpreter>>,
    ) {
        // Parse the selected item (remove emoji and get filename)
        let filename = Self::extract_filename_from_browser_text(selected_text);

        if filename == ".." {
            // Handle parent directory navigation
            info!("Navigate to parent directory");
            return;
        }

        if filename.ends_with(".xdl") || filename.ends_with(".pro") {
            // Handle XDL file selection and execution
            let file_path = current_dir.join(&filename);
            info!("Selected XDL file: {}", file_path.display());

            match std::fs::read_to_string(&file_path) {
                Ok(content) => {
                    // Display file content in command window (editor)
                    cmd_buffer.set_text(&format!(
                        "// File: {}\n{}\n\n// Click Execute to run this code",
                        filename, content
                    ));

                    // Show file loaded message in output
                    out_buffer.set_text(&format!(
                        "Loaded file: {}\nClick Execute button to run the code.\n",
                        filename
                    ));
                }
                Err(e) => {
                    let error_msg = format!("Error reading file {}: {}", filename, e);
                    out_buffer.set_text(&error_msg);
                    error!("Failed to read file: {}", e);
                }
            }
        } else {
            // Handle other file types (show info)
            let file_path = current_dir.join(&filename);
            if file_path.is_file() {
                if let Ok(metadata) = std::fs::metadata(&file_path) {
                    let size = metadata.len();
                    let info_text = format!(
                        "File: {}\nSize: {} bytes\nType: {}\n",
                        filename,
                        size,
                        Self::get_file_type(&filename)
                    );
                    out_buffer.set_text(&info_text);
                }
            }
        }
    }

    #[allow(dead_code)]
    fn extract_filename_from_browser_text(browser_text: &str) -> String {
        // Remove emoji and extract filename
        browser_text
            .split_whitespace()
            .skip(1) // Skip the emoji
            .collect::<Vec<&str>>()
            .join(" ")
    }

    #[allow(dead_code)]
    fn get_file_type(filename: &str) -> &'static str {
        match std::path::Path::new(filename)
            .extension()
            .and_then(|s| s.to_str())
        {
            Some("xdl") | Some("pro") => "XDL Script",
            Some("dat") => "Data File",
            Some("txt") => "Text File",
            Some("png") | Some("jpg") | Some("gif") => "Image File",
            Some("pdf") => "PDF Document",
            _ => "Unknown",
        }
    }

    fn looks_like_matlab(code: &str) -> bool {
        // Check for explicit MATLAB hint in comments
        // Supports: % MATLAB, // MATLAB, % matlab, // matlab
        if code.lines().any(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("% MATLAB")
                || trimmed.starts_with("// MATLAB")
                || trimmed.starts_with("%MATLAB")
                || trimmed.starts_with("//MATLAB")
                || trimmed.to_uppercase().starts_with("% MATLAB")
                || trimmed.to_uppercase().starts_with("// MATLAB")
        }) {
            return true;
        }

        // If code has XDL-style comments (starting with ;), it's already XDL
        if code.lines().any(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with(';') && !trimmed.starts_with(";;") // ; is XDL comment
        }) {
            return false;
        }

        // If code contains XDL keywords like FINDGEN, PLOT3D, it's XDL
        if code.contains("FINDGEN")
            || code.contains("PLOT3D")
            || code.contains("RANDOMU")
            || code.contains("N_ELEMENTS")
        {
            return false;
        }

        // Heuristic to detect MATLAB code
        // Look for common MATLAB patterns:
        // - MATLAB-specific functions
        // - Element-wise operators
        // - MATLAB-style comments starting with %
        code.contains("linspace")
            || code.contains("complex")
            || code.contains("axis")
            || code.contains("tiledlayout")
            || code.contains("nexttile")
            || code.contains("comet3")
            || code.contains("meshgrid")
            || code.contains("surf")
            || code.contains(" .*")
            || code.contains(".* ")
            || code.contains(" ./")
            || code.contains("./ ")
            || code.contains(" .^")
            || code.contains(".^ ")
            || code.lines().any(|line| line.trim_start().starts_with('%'))
    }

    fn execute_xdl_code_async(
        xdl_code: &str,
        filename: &str,
        cancel_flag: Arc<AtomicBool>,
    ) -> ExecutionResult {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static EXEC_COUNT: AtomicUsize = AtomicUsize::new(0);
        let exec_num = EXEC_COUNT.fetch_add(1, Ordering::SeqCst) + 1;

        info!(
            ">>> execute_xdl_code_async called (execution #{})",
            exec_num
        );

        let mut results = Vec::new();
        results.push(format!("=== Executing {} ===", filename));

        // Create new interpreter with output capture
        use std::cell::RefCell;
        use std::rc::Rc;
        use xdl_parser::parse_program;
        use xdl_parser::tokenize;

        results.push("✓ Executing with XDL interpreter".to_string());
        results.push("".to_string());

        // Create output capture buffer
        let capture_buffer: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(Vec::new()));
        let buffer_clone = Rc::clone(&capture_buffer);

        // Create interpreter with custom output
        let mut custom_interp = xdl_interpreter::Interpreter::with_output(buffer_clone);

        let mut variables = HashMap::new();

        // Parse and execute
        info!("Tokenizing XDL code...");

        // Check for cancellation before starting
        if cancel_flag.load(Ordering::Relaxed) {
            results.push("✗ Execution cancelled by user".to_string());
            results.push("=== Execution cancelled ===".to_string());
            return ExecutionResult {
                output_text: results.join("\n"),
                variables: HashMap::new(),
            };
        }

        match tokenize(xdl_code) {
            Ok(tokens) => {
                info!("Parsing {} tokens...", tokens.len());
                match parse_program(&tokens) {
                    Ok(program) => {
                        info!(
                            "Executing program with {} statements...",
                            program.statements.len()
                        );

                        // Check cancellation before execution
                        if cancel_flag.load(Ordering::Relaxed) {
                            results.push("✗ Execution cancelled by user".to_string());
                            results.push("=== Execution cancelled ===".to_string());
                            return ExecutionResult {
                                output_text: results.join("\n"),
                                variables: HashMap::new(),
                            };
                        }

                        match custom_interp.execute_program(&program) {
                            Ok(_) => {
                                info!("Program execution completed successfully");
                                // Get captured output
                                if let Ok(buf) = capture_buffer.try_borrow() {
                                    let output_str = String::from_utf8_lossy(&buf);
                                    if !output_str.is_empty() {
                                        results.push("Output:".to_string());
                                        for line in output_str.lines() {
                                            results.push(format!("  {}", line));
                                        }
                                        results.push("".to_string());
                                    }
                                }

                                // Extract variables from interpreter
                                let interp_vars = custom_interp.get_variables();
                                for (name, value) in interp_vars {
                                    variables.insert(name, value.to_string_repr());
                                }

                                results.push("✓ Execution completed successfully".to_string());
                            }
                            Err(e) => {
                                results.push(format!("✗ Execution error: {}", e));
                                error!("Execution error: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        results.push(format!("✗ Parse error: {}", e));
                        error!("Parse error: {}", e);
                    }
                }
            }
            Err(e) => {
                results.push(format!("✗ Tokenization error: {}", e));
                error!("Tokenization error: {}", e);
            }
        }

        results.push("=== Execution completed ===".to_string());
        let output_text = results.join("\n");

        info!(
            "<<< execute_xdl_code_async completed (execution #{})",
            exec_num
        );
        info!("Executed XDL file: {}", filename);

        ExecutionResult {
            output_text,
            variables,
        }
    }

    #[allow(dead_code)]
    fn show_execution_trace(
        xdl_code: &str,
        variables: &Rc<RefCell<HashMap<String, String>>>,
        results: &mut Vec<String>,
    ) {
        // Show what was executed with variable tracking
        let statements: Vec<&str> = xdl_code
            .lines()
            .filter(|line| {
                !line.trim().is_empty()
                    && !line.trim().starts_with(';')
                    && !line.trim().starts_with("//")
            })
            .collect();

        results.push("Execution trace:".to_string());
        for (i, statement) in statements.iter().enumerate() {
            let trimmed = statement.trim();

            // Track variables
            if trimmed.contains("=")
                && !trimmed.contains("==")
                && !trimmed.contains("!=")
                && !trimmed.contains("<=")
                && !trimmed.contains(">=")
            {
                let parts: Vec<&str> = trimmed.split('=').collect();
                if parts.len() == 2 {
                    let var_name = parts[0].trim();
                    let expression = parts[1].trim();

                    // Track in variables map
                    if let Ok(mut vars) = variables.try_borrow_mut() {
                        if let Ok(val) = expression.parse::<f64>() {
                            vars.insert(var_name.to_string(), val.to_string());
                        } else {
                            vars.insert(var_name.to_string(), "computed".to_string());
                        }
                    }
                }
            }

            // Show concise trace
            if trimmed.starts_with("if ")
                || trimmed.starts_with("for ")
                || trimmed.starts_with("while ")
                || trimmed.starts_with("print")
            {
                results.push(format!("  [{}] {}", i + 1, trimmed));
            } else if trimmed.contains("=") {
                let parts: Vec<&str> = trimmed.split('=').collect();
                if parts.len() == 2 {
                    results.push(format!(
                        "  [{}] {} = {}",
                        i + 1,
                        parts[0].trim(),
                        parts[1].trim()
                    ));
                }
            }
        }
    }

    #[allow(dead_code)]
    fn execute_fallback_simulation(
        xdl_code: &str,
        variables: &Rc<RefCell<HashMap<String, String>>>,
        results: &mut Vec<String>,
    ) {
        // Split XDL code into individual statements
        let statements: Vec<&str> = xdl_code
            .lines()
            .filter(|line| {
                !line.trim().is_empty()
                    && !line.trim().starts_with(';')
                    && !line.trim().starts_with("//")
            })
            .collect();

        for (i, statement) in statements.iter().enumerate() {
            let statement = statement.trim();
            if statement.is_empty() {
                continue;
            }

            results.push(format!("[{}] > {}", i + 1, statement));

            match Self::simulate_xdl_execution_with_vars(statement, variables) {
                Ok(result) => {
                    if !result.is_empty() {
                        results.push(format!("    => {}", result));
                    }
                }
                Err(e) => {
                    results.push(format!("    ERROR: {}", e));
                }
            }
        }
    }

    #[allow(dead_code)]
    fn simulate_xdl_execution(statement: &str) -> Result<String, String> {
        let trimmed = statement.trim();

        // Handle control flow keywords
        if trimmed.starts_with("if ") || trimmed == "if" {
            return Ok("[Control: IF statement]".to_string());
        } else if trimmed.starts_with("then") || trimmed == "then" {
            return Ok("".to_string()); // Don't show THEN
        } else if trimmed.starts_with("else") || trimmed == "else" {
            return Ok("[Control: ELSE branch]".to_string());
        } else if trimmed.starts_with("endif") || trimmed == "endif" {
            return Ok("".to_string()); // Don't show ENDIF
        } else if trimmed.starts_with("for ") {
            return Ok(format!("[Control: FOR loop - {}]", trimmed));
        } else if trimmed.starts_with("endfor") || trimmed == "endfor" {
            return Ok("[Control: FOR loop end]".to_string());
        } else if trimmed.starts_with("while ") {
            return Ok(format!("[Control: WHILE loop - {}]", trimmed));
        } else if trimmed.starts_with("endwhile") || trimmed == "endwhile" {
            return Ok("[Control: WHILE loop end]".to_string());
        } else if trimmed == "break" {
            return Ok("[Control: BREAK]".to_string());
        } else if trimmed == "continue" {
            return Ok("[Control: CONTINUE]".to_string());
        }

        if statement.contains("=")
            && !statement.contains("==")
            && !statement.contains("!=")
            && !statement.contains("<=")
            && !statement.contains(">=")
        {
            // Variable assignment
            let parts: Vec<&str> = statement.split('=').collect();
            if parts.len() == 2 {
                let var_name = parts[0].trim();
                let expression = parts[1].trim();

                // Simple expression evaluation
                if let Ok(value) = expression.parse::<f64>() {
                    return Ok(format!("{} = {}", var_name, value));
                } else if expression.starts_with("findgen") {
                    if let Some(n) = Self::extract_number_from_function(expression, "findgen") {
                        return Ok(format!("{} = array[0..{}]", var_name, n - 1));
                    }
                } else if expression.contains("sin") || expression.contains("cos") {
                    return Ok(format!("{} = computed function result", var_name));
                } else if expression.contains("+")
                    || expression.contains("-")
                    || expression.contains("*")
                    || expression.contains("/")
                {
                    // Arithmetic expression
                    return Ok(format!("{} = (computed: {})", var_name, expression));
                }

                return Ok(format!("{} = {}", var_name, expression));
            }
        } else if statement.starts_with("plot") {
            return Ok("📊 Plot generated".to_string());
        } else if statement.starts_with("print") {
            let content = statement
                .trim_start_matches("print")
                .trim()
                .trim_matches(',')
                .trim();
            if content.is_empty() {
                return Ok("\n".to_string());
            }
            // Parse print arguments
            let args: Vec<&str> = content.split(',').map(|s| s.trim()).collect();
            let mut output_parts = Vec::new();
            for arg in args {
                if arg.starts_with('"') && arg.ends_with('"') {
                    // String literal
                    output_parts.push(arg.trim_matches('"').to_string());
                } else {
                    // Variable or expression - show as-is for now
                    output_parts.push(format!("[{}]", arg));
                }
            }
            return Ok(format!("📄 {}", output_parts.join(" ")));
        }

        // Comparison expressions - don't show as separate statements
        if trimmed.contains(" eq ")
            || trimmed.contains(" ne ")
            || trimmed.contains(" lt ")
            || trimmed.contains(" gt ")
            || trimmed.contains(" le ")
            || trimmed.contains(" ge ")
            || trimmed.contains(" and ")
            || trimmed.contains(" or ")
        {
            return Ok("".to_string());
        }

        // Default: show the statement was executed
        if !trimmed.is_empty() {
            Ok(format!("✓ {}", trimmed))
        } else {
            Ok("".to_string())
        }
    }

    #[allow(dead_code)]
    fn simulate_xdl_execution_with_vars(
        statement: &str,
        variables: &Rc<RefCell<HashMap<String, String>>>,
    ) -> Result<String, String> {
        let result = Self::simulate_xdl_execution(statement);

        // Track variables
        if statement.contains("=") && !statement.contains("==") {
            let parts: Vec<&str> = statement.split('=').collect();
            if parts.len() == 2 {
                let var_name = parts[0].trim().to_string();
                let expression = parts[1].trim();

                // Determine variable type and value
                let var_value = if expression.starts_with("findgen") {
                    if let Some(n) = Self::extract_number_from_function(expression, "findgen") {
                        format!("array[1x{}]", n)
                    } else {
                        "array".to_string()
                    }
                } else if expression.contains("sin") || expression.contains("cos") {
                    "computed array".to_string()
                } else if let Ok(val) = expression.parse::<f64>() {
                    val.to_string()
                } else {
                    "computed".to_string()
                };

                if let Ok(mut vars) = variables.try_borrow_mut() {
                    vars.insert(var_name, var_value);
                }
            }
        }

        result
    }

    fn update_variable_table(
        var_table: &Rc<RefCell<VariableTable>>,
        variables: &Rc<RefCell<HashMap<String, String>>>,
    ) {
        if let (Ok(mut table), Ok(vars)) = (var_table.try_borrow_mut(), variables.try_borrow()) {
            table.update_data(&vars);
        }
    }

    #[allow(dead_code)]
    fn truncate_string(s: &str, max_len: usize) -> String {
        if s.len() > max_len {
            format!("{:.width$}...", s, width = max_len.saturating_sub(3))
        } else {
            s.to_string()
        }
    }

    #[allow(dead_code)]
    fn extract_number_from_function(expression: &str, function_name: &str) -> Option<i32> {
        if let Some(start) = expression.find(&format!("{}(", function_name)) {
            let start_idx = start + function_name.len() + 1;
            if let Some(end) = expression[start_idx..].find(')') {
                let number_str = &expression[start_idx..start_idx + end];
                return number_str.trim().parse().ok();
            }
        }
        None
    }

    #[allow(dead_code)]
    fn extract_plot_data_with_formula(
        plot_statement: &str,
        all_statements: &[&str],
    ) -> (Vec<f64>, Vec<f64>, String) {
        // Parse the plot command to extract variable names
        let plot_args = Self::parse_plot_arguments(plot_statement);

        // Generate data based on the script context
        match plot_args.len() {
            1 => {
                // plot, y_data -> use indices as x_data
                let y_var = &plot_args[0];
                let (x_data, y_data) = Self::generate_data_for_variable(y_var, all_statements);
                let formula = Self::generate_formula_for_variable(y_var, all_statements);
                (x_data, y_data, formula)
            }
            2 => {
                // plot, x_data, y_data
                let x_var = &plot_args[0];
                let y_var = &plot_args[1];
                let (_, x_data) = Self::generate_data_for_variable(x_var, all_statements);
                let (_, y_data) = Self::generate_data_for_variable(y_var, all_statements);
                let formula = format!("plot({}, {})", x_var, y_var);
                (x_data, y_data, formula)
            }
            _ => {
                // Default fallback
                let x_data: Vec<f64> = (0..100).map(|i| i as f64).collect();
                let y_data: Vec<f64> = (0..100).map(|i| (i as f64 * 0.1).sin()).collect();
                (x_data, y_data, "Default plot".to_string())
            }
        }
    }

    #[allow(dead_code)]
    fn parse_plot_arguments(plot_statement: &str) -> Vec<String> {
        // Extract arguments from plot command: "plot, var1, var2" -> ["var1", "var2"]
        let after_plot = plot_statement
            .trim()
            .strip_prefix("plot")
            .unwrap_or("")
            .trim()
            .strip_prefix(",")
            .unwrap_or("")
            .trim();

        if after_plot.is_empty() {
            return Vec::new();
        }

        after_plot
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    #[allow(dead_code)]
    fn generate_data_for_variable(var_name: &str, all_statements: &[&str]) -> (Vec<f64>, Vec<f64>) {
        // Analyze the script to understand how the variable was created
        for statement in all_statements {
            let statement = statement.trim();

            if statement.starts_with(&format!("{} =", var_name)) {
                // Found variable definition
                let rhs = statement.split('=').nth(1).unwrap_or("").trim();

                if rhs.contains("findgen") {
                    // Handle findgen-based data
                    if let Some(n) = Self::extract_number_from_function(rhs, "findgen") {
                        let base_data: Vec<f64> = (0..n).map(|i| i as f64).collect();

                        if rhs.contains("*") && rhs.contains("0.2") {
                            // scaled_data = data * 0.2
                            let scaled: Vec<f64> = base_data.iter().map(|&x| x * 0.2).collect();
                            return Self::create_indexed_data(scaled);
                        } else {
                            return Self::create_indexed_data(base_data);
                        }
                    }
                } else if rhs.contains("sin(") {
                    // Handle sin function
                    if rhs.contains("scaled_data") {
                        // func_data = sin(scaled_data) - need to trace scaled_data
                        let scaled_data: Vec<f64> = (0..100).map(|i| i as f64 * 0.2).collect();
                        let sin_data: Vec<f64> = scaled_data.iter().map(|&x| x.sin()).collect();
                        return (scaled_data, sin_data);
                    } else {
                        // Generic sin function
                        let x_data: Vec<f64> = (0..200).map(|i| i as f64 / 35.0).collect();
                        let y_data: Vec<f64> =
                            x_data.iter().map(|&x| (x.powf(2.5)).sin()).collect();
                        return (x_data, y_data);
                    }
                } else if rhs.contains("+") && rhs.contains("10.0") {
                    // shifted_data = scaled_data + 10.0
                    let base: Vec<f64> = (0..100).map(|i| i as f64 * 0.2 + 10.0).collect();
                    return Self::create_indexed_data(base);
                }
            }
        }

        // Default fallback based on variable name patterns
        match var_name {
            "data" => {
                let data: Vec<f64> = (0..100).map(|i| i as f64).collect();
                Self::create_indexed_data(data)
            }
            "scaled_data" => {
                let data: Vec<f64> = (0..100).map(|i| i as f64 * 0.2).collect();
                Self::create_indexed_data(data)
            }
            "func_data" => {
                let x_data: Vec<f64> = (0..100).map(|i| i as f64 * 0.2).collect();
                let y_data: Vec<f64> = x_data.iter().map(|&x| x.sin()).collect();
                (x_data, y_data)
            }
            "shifted_data" => {
                let data: Vec<f64> = (0..100).map(|i| i as f64 * 0.2 + 10.0).collect();
                Self::create_indexed_data(data)
            }
            _ => {
                // Generic fallback
                let x_data: Vec<f64> = (0..50).map(|i| i as f64).collect();
                let y_data: Vec<f64> = (0..50).map(|i| (i as f64 * 0.1).sin()).collect();
                (x_data, y_data)
            }
        }
    }

    #[allow(dead_code)]
    fn create_indexed_data(values: Vec<f64>) -> (Vec<f64>, Vec<f64>) {
        let indices: Vec<f64> = (0..values.len()).map(|i| i as f64).collect();
        (indices, values)
    }

    #[allow(dead_code)]
    fn generate_formula_for_variable(var_name: &str, all_statements: &[&str]) -> String {
        // Look for the variable definition to extract the formula
        for statement in all_statements {
            let statement = statement.trim();

            if statement.starts_with(&format!("{} =", var_name)) {
                // Found variable definition, extract the right-hand side
                let rhs = statement.split('=').nth(1).unwrap_or("").trim();

                // Create a more readable formula description
                if rhs.contains("findgen") {
                    if let Some(n) = Self::extract_number_from_function(rhs, "findgen") {
                        if rhs.contains("*") && rhs.contains("0.2") {
                            return format!("{} = findgen({}) * 0.2", var_name, n);
                        } else {
                            return format!("{} = findgen({})", var_name, n);
                        }
                    }
                } else if rhs.contains("sin(") {
                    if rhs.contains("scaled_data") {
                        return format!("{} = sin(scaled_data)", var_name);
                    } else {
                        return format!("{} = sin(...)", var_name);
                    }
                } else if rhs.contains("+") && rhs.contains("10.0") {
                    return format!("{} = scaled_data + 10.0", var_name);
                } else if rhs.contains("*") {
                    return format!("{} = data * factor", var_name);
                } else {
                    return format!("{} = {}", var_name, rhs);
                }
            }
        }

        // Fallback based on variable name patterns
        match var_name {
            "data" => "data = findgen(100)".to_string(),
            "scaled_data" => "scaled_data = data * 0.2".to_string(),
            "func_data" => "func_data = sin(scaled_data)".to_string(),
            "shifted_data" => "shifted_data = scaled_data + 10.0".to_string(),
            _ => format!("plot({})", var_name),
        }
    }

    pub fn show(&mut self) {
        info!("Showing XDL GUI window");
        self.window.show();
    }
}

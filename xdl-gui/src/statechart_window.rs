// SPDX-License-Identifier: MIT
// Copyright (C) 2026 Ravindra Boddipalli

//! Interactive FLTK canvas for editing state-chart documents.

use fltk::{
    app,
    button::Button,
    dialog, draw,
    enums::{Align, Color, Event, FrameType},
    frame::Frame,
    prelude::*,
    window::Window,
};
use std::cell::RefCell;
use std::rc::Rc;
use xdl_gnc_core::DurationNs;
use xdl_model::{
    ModelDocument, StateChartDocument, StateChartEditor, StateChartEditorError,
    StateChartEditorEvent, StateChartHitTarget, StateId, StateMachine, TransitionKey,
};

pub struct StateChartWindow {
    window: Window,
    #[allow(dead_code)]
    editor: Rc<RefCell<StateChartEditor>>,
}

impl StateChartWindow {
    pub fn new(document: StateChartDocument) -> Result<Self, String> {
        let editor = Rc::new(RefCell::new(
            StateChartEditor::new(document).map_err(|error| error.to_string())?,
        ));
        let window = Window::new(180, 100, 1040, 760, "XDL State Chart Editor");
        let mut add_state = Button::new(10, 10, 100, 32, "Add State");
        let mut add_event = Button::new(120, 10, 128, 32, "Add Event");
        let mut add_timer = Button::new(258, 10, 128, 32, "Add Timed");
        let mut save = Button::new(396, 10, 100, 32, "Save Chart");
        let mut delete = Button::new(506, 10, 110, 32, "Delete Selected");
        let mut set_initial = Button::new(626, 10, 110, 32, "Set Initial");
        let mut export_runtime = Button::new(746, 10, 142, 32, "Export Runtime");
        let mut edit_transition = Button::new(898, 10, 124, 32, "Edit Transition");
        let mut insert_model = Button::new(10, 48, 150, 32, "Insert in Model...");
        let mut canvas = Frame::new(10, 88, 1020, 662, "");
        canvas.set_frame(FrameType::DownBox);
        canvas.set_color(Color::White);

        let draw_editor = Rc::clone(&editor);
        canvas.draw(move |frame| draw_state_chart(frame, &draw_editor));

        let editor_for_handle = Rc::clone(&editor);
        let drag_position = Rc::new(RefCell::new(None::<(i32, i32)>));
        let drag_position_handle = Rc::clone(&drag_position);
        canvas.handle(move |frame, event| match event {
            Event::Push => {
                let x = (app::event_x() - frame.x()) as f64;
                let y = (app::event_y() - frame.y()) as f64;
                if let Ok(mut editor) = editor_for_handle.try_borrow_mut() {
                    let _ = editor.select_at(x, y, 8.0);
                }
                *drag_position_handle.borrow_mut() = Some((app::event_x(), app::event_y()));
                frame.redraw();
                true
            }
            Event::Drag => {
                let current = (app::event_x(), app::event_y());
                if let Some(previous) = *drag_position_handle.borrow() {
                    if let Ok(mut editor) = editor_for_handle.try_borrow_mut() {
                        let _ = editor.apply(StateChartEditorEvent::MoveSelected {
                            dx: f64::from(current.0 - previous.0),
                            dy: f64::from(current.1 - previous.1),
                        });
                    }
                    *drag_position_handle.borrow_mut() = Some(current);
                    frame.redraw();
                }
                true
            }
            Event::Released => {
                *drag_position_handle.borrow_mut() = None;
                true
            }
            _ => false,
        });

        let editor_for_add_state = Rc::clone(&editor);
        let mut canvas_for_add_state = canvas.clone();
        add_state.set_callback(move |_| {
            if let Some(name) = dialog::input_default("New state name", "STATE") {
                if let Ok(mut editor) = editor_for_add_state.try_borrow_mut() {
                    if let Err(error) = editor.document.add_state(name) {
                        dialog::alert_default(&error.to_string());
                    }
                }
                canvas_for_add_state.redraw();
            }
        });

        let editor_for_add_event = Rc::clone(&editor);
        let mut canvas_for_add_event = canvas.clone();
        add_event.set_callback(move |_| {
            if let Some((from, to, event, guard)) = prompt_event_transition() {
                if let Ok(mut editor) = editor_for_add_event.try_borrow_mut() {
                    if let Err(error) = editor.document.add_event_transition(
                        StateId::new(from),
                        StateId::new(to),
                        event,
                        guard,
                    ) {
                        dialog::alert_default(&error.to_string());
                    }
                }
                canvas_for_add_event.redraw();
            }
        });

        let editor_for_add_timer = Rc::clone(&editor);
        let mut canvas_for_add_timer = canvas.clone();
        add_timer.set_callback(move |_| {
            if let Some((from, to, delay)) = prompt_timed_transition() {
                if let Ok(mut editor) = editor_for_add_timer.try_borrow_mut() {
                    if let Err(error) = editor.document.add_timed_transition(
                        StateId::new(from),
                        StateId::new(to),
                        DurationNs::from_nanoseconds(delay),
                    ) {
                        dialog::alert_default(&error.to_string());
                    }
                }
                canvas_for_add_timer.redraw();
            }
        });

        let editor_for_delete = Rc::clone(&editor);
        let mut canvas_for_delete = canvas.clone();
        delete.set_callback(move |_| {
            if let Ok(mut editor) = editor_for_delete.try_borrow_mut() {
                let result = match editor.selection {
                    Some(StateChartHitTarget::State(state)) => editor.document.remove_state(state),
                    Some(StateChartHitTarget::Transition(transition)) => {
                        editor.document.remove_transition(transition)
                    }
                    None => {
                        dialog::alert_default("Select a state or transition to delete.");
                        return;
                    }
                };
                match result {
                    Ok(()) => editor.selection = None,
                    Err(error) => dialog::alert_default(&error.to_string()),
                }
            }
            canvas_for_delete.redraw();
        });

        let editor_for_initial = Rc::clone(&editor);
        let mut canvas_for_initial = canvas.clone();
        set_initial.set_callback(move |_| {
            if let Ok(mut editor) = editor_for_initial.try_borrow_mut() {
                let result = match editor.selection {
                    Some(StateChartHitTarget::State(state)) => editor
                        .document
                        .machine
                        .set_initial_state(state)
                        .map_err(|error| error.to_string())
                        .and_then(|()| {
                            editor
                                .document
                                .validate()
                                .map_err(|error| error.to_string())
                        }),
                    Some(StateChartHitTarget::Transition(_)) => {
                        dialog::alert_default("Select a state to make it the initial state.");
                        return;
                    }
                    None => {
                        dialog::alert_default("Select a state to make it the initial state.");
                        return;
                    }
                };
                if let Err(error) = result {
                    dialog::alert_default(&error.to_string());
                }
            }
            canvas_for_initial.redraw();
        });

        let editor_for_save = Rc::clone(&editor);
        save.set_callback(move |_| {
            if let Some(path) = dialog::file_chooser("Save State Chart", "*.json", ".", true) {
                let result = editor_for_save
                    .try_borrow()
                    .map_err(|_| "state chart is busy".to_string())
                    .and_then(|editor| editor.document.to_json().map_err(|error| error.to_string()))
                    .and_then(|json| {
                        std::fs::write(&path, json).map_err(|error| error.to_string())
                    });
                if let Err(error) = result {
                    dialog::alert_default(&format!("Could not save state chart: {error}"));
                }
            }
        });

        let editor_for_export = Rc::clone(&editor);
        export_runtime.set_callback(move |_| {
            let Some(period) =
                dialog::input_default("Runtime sample period (nanoseconds)", "100000000")
            else {
                return;
            };
            let result = period
                .parse::<i64>()
                .map_err(|error| error.to_string())
                .and_then(|period_ns| {
                    editor_for_export
                        .try_borrow()
                        .map_err(|_| "state chart is busy".to_string())?
                        .document
                        .runtime_block_parameters(period_ns)
                        .map_err(|error| error.to_string())
                })
                .and_then(|parameters| {
                    serde_json::to_string_pretty(&parameters).map_err(|error| error.to_string())
                });
            match result {
                Ok(json) => {
                    if let Some(path) = dialog::file_chooser(
                        "Export State Machine Runtime Parameters",
                        "*.json",
                        ".",
                        true,
                    ) {
                        if let Err(error) = std::fs::write(&path, json) {
                            dialog::alert_default(&format!(
                                "Could not export runtime parameters: {error}"
                            ));
                        }
                    }
                }
                Err(error) => {
                    dialog::alert_default(&format!("Could not export runtime parameters: {error}"))
                }
            }
        });

        let editor_for_edit_transition = Rc::clone(&editor);
        let mut canvas_for_edit_transition = canvas.clone();
        edit_transition.set_callback(move |_| {
            let selected = editor_for_edit_transition
                .try_borrow()
                .ok()
                .and_then(|editor| match editor.selection {
                    Some(StateChartHitTarget::Transition(key)) => {
                        Some((key, editor.document.machine.clone()))
                    }
                    _ => None,
                });
            let Some((key, machine)) = selected else {
                dialog::alert_default("Select an event or timed transition to edit.");
                return;
            };
            let edited = match key {
                TransitionKey::Event(index) => {
                    machine.transitions.get(index).and_then(|transition| {
                        prompt_edit_event_transition(
                            transition.from.value(),
                            transition.to.value(),
                            &transition.event,
                            transition.guard.as_deref().unwrap_or(""),
                        )
                        .map(|(from, to, event, guard)| (from, to, Some(event), guard, None))
                    })
                }
                TransitionKey::Timed(index) => {
                    machine.timed_transitions.get(index).and_then(|transition| {
                        prompt_edit_timed_transition(
                            transition.from.value(),
                            transition.to.value(),
                            transition.after.as_nanoseconds(),
                        )
                        .map(|(from, to, delay)| (from, to, None, None, Some(delay)))
                    })
                }
            };
            if let Some((from, to, event, guard, delay)) = edited {
                if let Ok(mut editor) = editor_for_edit_transition.try_borrow_mut() {
                    let result = (|| -> Result<(), StateChartEditorError> {
                        match key {
                            TransitionKey::Event(index) => editor.document.edit_event_transition(
                                index,
                                StateId::new(from),
                                StateId::new(to),
                                event.ok_or_else(|| {
                                    StateChartEditorError::InvalidInteraction(
                                        "event transition is missing its event name".to_string(),
                                    )
                                })?,
                                guard,
                            ),
                            TransitionKey::Timed(index) => editor.document.edit_timed_transition(
                                index,
                                StateId::new(from),
                                StateId::new(to),
                                DurationNs::from_nanoseconds(delay.ok_or_else(|| {
                                    StateChartEditorError::InvalidInteraction(
                                        "timed transition is missing its delay".to_string(),
                                    )
                                })?),
                            ),
                        }
                    })();
                    if let Err(error) = result {
                        dialog::alert_default(&error.to_string());
                    }
                }
                canvas_for_edit_transition.redraw();
            }
        });

        let editor_for_insert_model = Rc::clone(&editor);
        insert_model.set_callback(move |_| {
            let Some(source_path) =
                dialog::file_chooser("Open Model Document", "*.json", ".", false)
            else {
                return;
            };
            let result = (|| -> Result<String, String> {
                let model_json = std::fs::read_to_string(&source_path)
                    .map_err(|error| format!("could not read model document: {error}"))?;
                let mut document =
                    ModelDocument::from_json(&model_json).map_err(|error| error.to_string())?;
                let (machine_name, parameters) = {
                    let chart = editor_for_insert_model
                        .try_borrow()
                        .map_err(|_| "state chart is busy".to_string())?;
                    let machine_name = chart.document.machine.name.clone();
                    let period =
                        dialog::input_default("Runtime sample period (nanoseconds)", "100000000")
                            .ok_or_else(|| "cancelled".to_string())?
                            .parse::<i64>()
                            .map_err(|error| error.to_string())?;
                    (machine_name, period)
                };
                let block_name = dialog::input_default("New block name", &machine_name)
                    .ok_or_else(|| "cancelled".to_string())?;
                editor_for_insert_model
                    .try_borrow()
                    .map_err(|_| "state chart is busy".to_string())?
                    .document
                    .insert_runtime_block(&mut document, block_name, parameters)
                    .map_err(|error| error.to_string())?;
                document.to_json().map_err(|error| error.to_string())
            })();
            match result {
                Ok(json) => {
                    if let Some(output_path) =
                        dialog::file_chooser("Save Model with State Machine", "*.json", ".", true)
                    {
                        if let Err(error) = std::fs::write(output_path, json) {
                            dialog::alert_default(&format!(
                                "Could not save integrated model: {error}"
                            ));
                        }
                    }
                }
                Err(error) if error != "cancelled" => {
                    dialog::alert_default(&format!("Could not insert state machine: {error}"));
                }
                Err(_) => {}
            }
        });

        window.end();
        Ok(Self { window, editor })
    }

    pub fn show(&mut self) {
        self.window.show();
    }
}

pub fn starter_document() -> Result<StateChartDocument, String> {
    let mut machine = StateMachine::new("flight_readiness");
    let cold = machine
        .add_state("COLD")
        .map_err(|error| error.to_string())?;
    let ready = machine
        .add_state("READY")
        .map_err(|error| error.to_string())?;
    let flying = machine
        .add_state("FLYING")
        .map_err(|error| error.to_string())?;
    machine
        .set_initial_state(cold)
        .map_err(|error| error.to_string())?;
    machine
        .add_guarded_transition(cold, ready, "thermal_ready", "ready_guard")
        .map_err(|error| error.to_string())?;
    machine
        .add_transition(ready, flying, "launch")
        .map_err(|error| error.to_string())?;
    machine
        .add_timed_transition(flying, ready, DurationNs::from_nanoseconds(30_000_000_000))
        .map_err(|error| error.to_string())?;
    StateChartDocument::from_machine(machine).map_err(|error| error.to_string())
}

fn prompt_event_transition() -> Option<(u32, u32, String, Option<String>)> {
    let from = dialog::input_default("Source state ID", "0")?
        .parse()
        .ok()?;
    let to = dialog::input_default("Destination state ID", "1")?
        .parse()
        .ok()?;
    let event = dialog::input_default("Event name", "event")?;
    let guard = dialog::input_default("Guard name (optional)", "")?;
    Some((from, to, event, (!guard.trim().is_empty()).then_some(guard)))
}

fn prompt_timed_transition() -> Option<(u32, u32, i64)> {
    let from = dialog::input_default("Source state ID", "0")?
        .parse()
        .ok()?;
    let to = dialog::input_default("Destination state ID", "1")?
        .parse()
        .ok()?;
    let delay = dialog::input_default("Delay (nanoseconds)", "1000000000")?
        .parse::<i64>()
        .ok()?;
    Some((from, to, delay))
}

fn prompt_edit_event_transition(
    from: u32,
    to: u32,
    event: &str,
    guard: &str,
) -> Option<(u32, u32, String, Option<String>)> {
    let from = dialog::input_default("Source state ID", &from.to_string())?
        .parse()
        .ok()?;
    let to = dialog::input_default("Destination state ID", &to.to_string())?
        .parse()
        .ok()?;
    let event = dialog::input_default("Event name", event)?;
    let guard = dialog::input_default("Guard name (optional)", guard)?;
    Some((from, to, event, (!guard.trim().is_empty()).then_some(guard)))
}

fn prompt_edit_timed_transition(from: u32, to: u32, delay: i64) -> Option<(u32, u32, i64)> {
    let from = dialog::input_default("Source state ID", &from.to_string())?
        .parse()
        .ok()?;
    let to = dialog::input_default("Destination state ID", &to.to_string())?
        .parse()
        .ok()?;
    let delay = dialog::input_default("Delay (nanoseconds)", &delay.to_string())?
        .parse()
        .ok()?;
    Some((from, to, delay))
}

fn draw_state_chart(frame: &Frame, editor: &Rc<RefCell<StateChartEditor>>) {
    let Ok(editor) = editor.try_borrow() else {
        return;
    };
    let document = &editor.document;
    let offset = (frame.x(), frame.y());
    draw::set_draw_color(Color::White);
    draw::draw_rectf(frame.x(), frame.y(), frame.w(), frame.h());

    for (index, transition) in document.machine.transitions.iter().enumerate() {
        let key = TransitionKey::Event(index);
        draw_transition(
            document,
            transition.from,
            transition.to,
            offset,
            editor.selection == Some(StateChartHitTarget::Transition(key)),
        );
        let label = match &transition.guard {
            Some(guard) => format!("{} [{}]", transition.event, guard),
            None => transition.event.clone(),
        };
        if let Some(position) = document.layout.transitions.get(&key) {
            draw::set_draw_color(Color::Black);
            draw::draw_text2(
                &label,
                offset.0 + position.x as i32 - 80,
                offset.1 + position.y as i32 - 12,
                160,
                20,
                Align::Center,
            );
        }
    }
    for (index, transition) in document.machine.timed_transitions.iter().enumerate() {
        let key = TransitionKey::Timed(index);
        draw_transition(
            document,
            transition.from,
            transition.to,
            offset,
            editor.selection == Some(StateChartHitTarget::Transition(key)),
        );
        if let Some(position) = document.layout.transitions.get(&key) {
            draw::set_draw_color(Color::Black);
            draw::draw_text2(
                &format!("after {} ns", transition.after.as_nanoseconds()),
                offset.0 + position.x as i32 - 80,
                offset.1 + position.y as i32 - 12,
                160,
                20,
                Align::Center,
            );
        }
    }
    for state in document.machine.states.values() {
        let geometry = document.layout.states[&state.id];
        draw::set_draw_color(
            if editor.selection == Some(StateChartHitTarget::State(state.id)) {
                Color::Blue
            } else {
                Color::from_rgb(40, 55, 70)
            },
        );
        draw::draw_rect(
            offset.0 + geometry.x as i32,
            offset.1 + geometry.y as i32,
            geometry.width as i32,
            geometry.height as i32,
        );
        draw::draw_text2(
            &format!("{} ({})", state.name, state.id.value()),
            offset.0 + geometry.x as i32,
            offset.1 + geometry.y as i32,
            geometry.width as i32,
            geometry.height as i32,
            Align::Center,
        );
    }
}

fn draw_transition(
    document: &StateChartDocument,
    from: StateId,
    to: StateId,
    offset: (i32, i32),
    selected: bool,
) {
    let from_layout = document.layout.states[&from];
    let to_layout = document.layout.states[&to];
    let start = center(from_layout);
    let end = center(to_layout);
    draw::set_draw_color(if selected { Color::Blue } else { Color::Dark3 });
    if from == to {
        let left = start.0 - from_layout.width * 0.2;
        let right = start.0 + from_layout.width * 0.2;
        let top = from_layout.y;
        let first = (right, top);
        let control1 = (right + 56.0, top - 72.0);
        let control2 = (left - 56.0, top - 72.0);
        let last = (left, top);
        let mut previous = first;
        for index in 1..=24 {
            let t = index as f64 / 24.0;
            let inverse = 1.0 - t;
            let current = (
                inverse.powi(3) * first.0
                    + 3.0 * inverse.powi(2) * t * control1.0
                    + 3.0 * inverse * t.powi(2) * control2.0
                    + t.powi(3) * last.0,
                inverse.powi(3) * first.1
                    + 3.0 * inverse.powi(2) * t * control1.1
                    + 3.0 * inverse * t.powi(2) * control2.1
                    + t.powi(3) * last.1,
            );
            draw::draw_line(
                offset.0 + previous.0 as i32,
                offset.1 + previous.1 as i32,
                offset.0 + current.0 as i32,
                offset.1 + current.1 as i32,
            );
            previous = current;
        }
    } else {
        let direction = (end.0 - start.0, end.1 - start.1);
        let start = rectangle_edge(from_layout, direction);
        let end = rectangle_edge(to_layout, (-direction.0, -direction.1));
        draw::draw_line(
            offset.0 + start.0 as i32,
            offset.1 + start.1 as i32,
            offset.0 + end.0 as i32,
            offset.1 + end.1 as i32,
        );
        draw_arrow(offset, start, end);
    }
}

fn rectangle_edge(layout: xdl_model::StateNodeLayout, direction: (f64, f64)) -> (f64, f64) {
    let center = center(layout);
    let horizontal_scale = if direction.0 == 0.0 {
        f64::INFINITY
    } else {
        layout.width / 2.0 / direction.0.abs()
    };
    let vertical_scale = if direction.1 == 0.0 {
        f64::INFINITY
    } else {
        layout.height / 2.0 / direction.1.abs()
    };
    let scale = horizontal_scale.min(vertical_scale);
    (
        center.0 + direction.0 * scale,
        center.1 + direction.1 * scale,
    )
}

fn draw_arrow(offset: (i32, i32), start: (f64, f64), end: (f64, f64)) {
    let direction = (end.0 - start.0, end.1 - start.1);
    let length = (direction.0 * direction.0 + direction.1 * direction.1).sqrt();
    if length == 0.0 {
        return;
    }
    let unit = (direction.0 / length, direction.1 / length);
    let base = (end.0 - unit.0 * 12.0, end.1 - unit.1 * 12.0);
    let wing = 5.0;
    for sign in [-1.0, 1.0] {
        draw::draw_line(
            offset.0 + end.0 as i32,
            offset.1 + end.1 as i32,
            offset.0 + (base.0 - sign * unit.1 * wing) as i32,
            offset.1 + (base.1 + sign * unit.0 * wing) as i32,
        );
    }
}

fn center(layout: xdl_model::StateNodeLayout) -> (f64, f64) {
    (
        layout.x + layout.width / 2.0,
        layout.y + layout.height / 2.0,
    )
}

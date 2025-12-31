//! Widget Functions for GUI Programming
//!
//! This module provides IDL-compatible widget functions for creating
//! graphical user interfaces. These are placeholder implementations
//! that provide API compatibility. Full GUI support requires the xdl-gui crate.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use xdl_core::{XdlError, XdlResult, XdlValue};

/// Counter for generating unique widget IDs
static WIDGET_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

/// Widget storage for tracking created widgets
static WIDGET_STORE: Mutex<Option<HashMap<usize, WidgetInfo>>> = Mutex::new(None);

/// Widget types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetType {
    Base,
    Button,
    Slider,
    Text,
    Label,
    List,
    Droplist,
    Draw,
    Table,
    Tree,
    Tab,
    Combobox,
    PropertySheet,
}

impl WidgetType {
    fn name(&self) -> &'static str {
        match self {
            Self::Base => "BASE",
            Self::Button => "BUTTON",
            Self::Slider => "SLIDER",
            Self::Text => "TEXT",
            Self::Label => "LABEL",
            Self::List => "LIST",
            Self::Droplist => "DROPLIST",
            Self::Draw => "DRAW",
            Self::Table => "TABLE",
            Self::Tree => "TREE",
            Self::Tab => "TAB",
            Self::Combobox => "COMBOBOX",
            Self::PropertySheet => "PROPERTYSHEET",
        }
    }
}

/// Widget information
#[derive(Debug, Clone)]
pub struct WidgetInfo {
    pub id: usize,
    pub widget_type: WidgetType,
    pub parent_id: Option<usize>,
    pub title: String,
    pub uvalue: Option<XdlValue>,
    pub sensitive: bool,
    pub visible: bool,
    pub realized: bool,
}

fn get_next_widget_id() -> usize {
    WIDGET_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

fn store_widget(info: WidgetInfo) {
    let mut store = WIDGET_STORE.lock().unwrap();
    if store.is_none() {
        *store = Some(HashMap::new());
    }
    if let Some(ref mut map) = *store {
        map.insert(info.id, info);
    }
}

fn get_widget(id: usize) -> Option<WidgetInfo> {
    let store = WIDGET_STORE.lock().unwrap();
    store.as_ref().and_then(|map| map.get(&id).cloned())
}

fn update_widget<F>(id: usize, f: F) -> bool
where
    F: FnOnce(&mut WidgetInfo),
{
    let mut store = WIDGET_STORE.lock().unwrap();
    if let Some(ref mut map) = *store {
        if let Some(widget) = map.get_mut(&id) {
            f(widget);
            return true;
        }
    }
    false
}

/// Helper to extract usize from XdlValue
fn value_to_usize(v: &XdlValue) -> Option<usize> {
    match v {
        XdlValue::Int(i) => Some(*i as usize),
        XdlValue::Long(l) => Some(*l as usize),
        XdlValue::Float(f) => Some(*f as usize),
        XdlValue::Double(d) => Some(*d as usize),
        _ => None,
    }
}

/// WIDGET_BASE - Create a base widget (container)
/// IDL syntax: id = WIDGET_BASE([parent] [, /COLUMN] [, /ROW] [, TITLE=title])
pub fn widget_base(args: &[XdlValue], keywords: &HashMap<String, XdlValue>) -> XdlResult<XdlValue> {
    let parent_id = args.first().and_then(|v| value_to_usize(v));

    let title = keywords
        .get("TITLE")
        .and_then(|v| match v {
            XdlValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| "Widget Base".to_string());

    let is_column = keywords.contains_key("COLUMN");
    let is_row = keywords.contains_key("ROW");
    let is_modal = keywords.contains_key("MODAL");
    let is_floating = keywords.contains_key("FLOATING");

    let id = get_next_widget_id();

    let info = WidgetInfo {
        id,
        widget_type: WidgetType::Base,
        parent_id,
        title: title.clone(),
        uvalue: keywords.get("UVALUE").cloned(),
        sensitive: true,
        visible: true,
        realized: false,
    };

    store_widget(info);

    let layout = if is_column {
        "COLUMN"
    } else if is_row {
        "ROW"
    } else {
        "DEFAULT"
    };

    println!(
        "WIDGET_BASE: Created base widget {} (parent={:?}, layout={}, modal={}, floating={})",
        id, parent_id, layout, is_modal, is_floating
    );
    println!("  Title: {}", title);

    Ok(XdlValue::Long(id as i32))
}

/// WIDGET_BUTTON - Create a button widget
/// IDL syntax: id = WIDGET_BUTTON(parent, VALUE=label [, /MENU] [, UVALUE=uvalue])
pub fn widget_button(
    args: &[XdlValue],
    keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "WIDGET_BUTTON: Expected parent widget ID".to_string(),
        ));
    }

    let parent_id = value_to_usize(&args[0]).ok_or_else(|| {
        XdlError::TypeMismatch {
            expected: "integer".to_string(),
            actual: format!("{:?}", args[0]),
        }
    })?;

    let label = keywords
        .get("VALUE")
        .and_then(|v| match v {
            XdlValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| "Button".to_string());

    let is_menu = keywords.contains_key("MENU");
    let is_bitmap = keywords.contains_key("BITMAP");

    let id = get_next_widget_id();

    let info = WidgetInfo {
        id,
        widget_type: WidgetType::Button,
        parent_id: Some(parent_id),
        title: label.clone(),
        uvalue: keywords.get("UVALUE").cloned(),
        sensitive: true,
        visible: true,
        realized: false,
    };

    store_widget(info);

    println!(
        "WIDGET_BUTTON: Created button {} in parent {} (menu={}, bitmap={})",
        id, parent_id, is_menu, is_bitmap
    );
    println!("  Label: {}", label);

    Ok(XdlValue::Long(id as i32))
}

/// WIDGET_SLIDER - Create a slider widget
/// IDL syntax: id = WIDGET_SLIDER(parent [, MINIMUM=min] [, MAXIMUM=max] [, VALUE=value])
pub fn widget_slider(
    args: &[XdlValue],
    keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "WIDGET_SLIDER: Expected parent widget ID".to_string(),
        ));
    }

    let parent_id = value_to_usize(&args[0]).ok_or_else(|| {
        XdlError::TypeMismatch {
            expected: "integer".to_string(),
            actual: format!("{:?}", args[0]),
        }
    })?;

    let minimum = keywords
        .get("MINIMUM")
        .and_then(|v| match v {
            XdlValue::Int(i) => Some(*i as i64),
            XdlValue::Long(l) => Some(*l as i64),
            _ => None,
        })
        .unwrap_or(0);

    let maximum = keywords
        .get("MAXIMUM")
        .and_then(|v| match v {
            XdlValue::Int(i) => Some(*i as i64),
            XdlValue::Long(l) => Some(*l as i64),
            _ => None,
        })
        .unwrap_or(100);

    let value = keywords
        .get("VALUE")
        .and_then(|v| match v {
            XdlValue::Int(i) => Some(*i as i64),
            XdlValue::Long(l) => Some(*l as i64),
            _ => None,
        })
        .unwrap_or(minimum);

    let id = get_next_widget_id();

    let info = WidgetInfo {
        id,
        widget_type: WidgetType::Slider,
        parent_id: Some(parent_id),
        title: format!("Slider ({}-{})", minimum, maximum),
        uvalue: keywords.get("UVALUE").cloned(),
        sensitive: true,
        visible: true,
        realized: false,
    };

    store_widget(info);

    println!(
        "WIDGET_SLIDER: Created slider {} in parent {} (range={}-{}, value={})",
        id, parent_id, minimum, maximum, value
    );

    Ok(XdlValue::Long(id as i32))
}

/// WIDGET_TEXT - Create a text widget
/// IDL syntax: id = WIDGET_TEXT(parent [, VALUE=text] [, /EDITABLE] [, XSIZE=cols] [, YSIZE=rows])
pub fn widget_text(args: &[XdlValue], keywords: &HashMap<String, XdlValue>) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "WIDGET_TEXT: Expected parent widget ID".to_string(),
        ));
    }

    let parent_id = value_to_usize(&args[0]).ok_or_else(|| {
        XdlError::TypeMismatch {
            expected: "integer".to_string(),
            actual: format!("{:?}", args[0]),
        }
    })?;

    let text = keywords
        .get("VALUE")
        .and_then(|v| match v {
            XdlValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_default();

    let editable = keywords.contains_key("EDITABLE");
    let xsize = keywords
        .get("XSIZE")
        .and_then(|v| value_to_usize(v))
        .unwrap_or(40);
    let ysize = keywords
        .get("YSIZE")
        .and_then(|v| value_to_usize(v))
        .unwrap_or(1);

    let id = get_next_widget_id();

    let info = WidgetInfo {
        id,
        widget_type: WidgetType::Text,
        parent_id: Some(parent_id),
        title: text.clone(),
        uvalue: keywords.get("UVALUE").cloned(),
        sensitive: true,
        visible: true,
        realized: false,
    };

    store_widget(info);

    println!(
        "WIDGET_TEXT: Created text {} in parent {} ({}x{}, editable={})",
        id, parent_id, xsize, ysize, editable
    );

    Ok(XdlValue::Long(id as i32))
}

/// WIDGET_LABEL - Create a label widget
/// IDL syntax: id = WIDGET_LABEL(parent, VALUE=text)
pub fn widget_label(
    args: &[XdlValue],
    keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "WIDGET_LABEL: Expected parent widget ID".to_string(),
        ));
    }

    let parent_id = value_to_usize(&args[0]).ok_or_else(|| {
        XdlError::TypeMismatch {
            expected: "integer".to_string(),
            actual: format!("{:?}", args[0]),
        }
    })?;

    let text = keywords
        .get("VALUE")
        .and_then(|v| match v {
            XdlValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| "Label".to_string());

    let id = get_next_widget_id();

    let info = WidgetInfo {
        id,
        widget_type: WidgetType::Label,
        parent_id: Some(parent_id),
        title: text.clone(),
        uvalue: keywords.get("UVALUE").cloned(),
        sensitive: true,
        visible: true,
        realized: false,
    };

    store_widget(info);

    println!(
        "WIDGET_LABEL: Created label {} in parent {}: \"{}\"",
        id, parent_id, text
    );

    Ok(XdlValue::Long(id as i32))
}

/// WIDGET_DRAW - Create a drawing widget
/// IDL syntax: id = WIDGET_DRAW(parent [, XSIZE=width] [, YSIZE=height])
pub fn widget_draw(args: &[XdlValue], keywords: &HashMap<String, XdlValue>) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "WIDGET_DRAW: Expected parent widget ID".to_string(),
        ));
    }

    let parent_id = value_to_usize(&args[0]).ok_or_else(|| {
        XdlError::TypeMismatch {
            expected: "integer".to_string(),
            actual: format!("{:?}", args[0]),
        }
    })?;

    let xsize = keywords
        .get("XSIZE")
        .and_then(|v| value_to_usize(v))
        .unwrap_or(640);
    let ysize = keywords
        .get("YSIZE")
        .and_then(|v| value_to_usize(v))
        .unwrap_or(480);

    let id = get_next_widget_id();

    let info = WidgetInfo {
        id,
        widget_type: WidgetType::Draw,
        parent_id: Some(parent_id),
        title: format!("Draw {}x{}", xsize, ysize),
        uvalue: keywords.get("UVALUE").cloned(),
        sensitive: true,
        visible: true,
        realized: false,
    };

    store_widget(info);

    println!(
        "WIDGET_DRAW: Created draw widget {} in parent {} ({}x{})",
        id, parent_id, xsize, ysize
    );

    Ok(XdlValue::Long(id as i32))
}

/// WIDGET_LIST - Create a list widget
/// IDL syntax: id = WIDGET_LIST(parent [, VALUE=items] [, YSIZE=rows])
pub fn widget_list(args: &[XdlValue], keywords: &HashMap<String, XdlValue>) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "WIDGET_LIST: Expected parent widget ID".to_string(),
        ));
    }

    let parent_id = value_to_usize(&args[0]).ok_or_else(|| {
        XdlError::TypeMismatch {
            expected: "integer".to_string(),
            actual: format!("{:?}", args[0]),
        }
    })?;

    let num_items = match keywords.get("VALUE") {
        Some(XdlValue::NestedArray(arr)) => arr.len(),
        Some(XdlValue::Array(arr)) => arr.len(),
        _ => 0,
    };

    let ysize = keywords
        .get("YSIZE")
        .and_then(|v| value_to_usize(v))
        .unwrap_or(5);

    let id = get_next_widget_id();

    let info = WidgetInfo {
        id,
        widget_type: WidgetType::List,
        parent_id: Some(parent_id),
        title: format!("List ({} items)", num_items),
        uvalue: keywords.get("UVALUE").cloned(),
        sensitive: true,
        visible: true,
        realized: false,
    };

    store_widget(info);

    println!(
        "WIDGET_LIST: Created list {} in parent {} ({} items, {} rows)",
        id, parent_id, num_items, ysize
    );

    Ok(XdlValue::Long(id as i32))
}

/// WIDGET_DROPLIST - Create a dropdown list widget
/// IDL syntax: id = WIDGET_DROPLIST(parent [, VALUE=items] [, TITLE=title])
pub fn widget_droplist(
    args: &[XdlValue],
    keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "WIDGET_DROPLIST: Expected parent widget ID".to_string(),
        ));
    }

    let parent_id = value_to_usize(&args[0]).ok_or_else(|| {
        XdlError::TypeMismatch {
            expected: "integer".to_string(),
            actual: format!("{:?}", args[0]),
        }
    })?;

    let title = keywords
        .get("TITLE")
        .and_then(|v| match v {
            XdlValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_default();

    let num_items = match keywords.get("VALUE") {
        Some(XdlValue::NestedArray(arr)) => arr.len(),
        Some(XdlValue::Array(arr)) => arr.len(),
        _ => 0,
    };

    let id = get_next_widget_id();

    let info = WidgetInfo {
        id,
        widget_type: WidgetType::Droplist,
        parent_id: Some(parent_id),
        title,
        uvalue: keywords.get("UVALUE").cloned(),
        sensitive: true,
        visible: true,
        realized: false,
    };

    store_widget(info);

    println!(
        "WIDGET_DROPLIST: Created droplist {} in parent {} ({} items)",
        id, parent_id, num_items
    );

    Ok(XdlValue::Long(id as i32))
}

/// WIDGET_CONTROL - Control widget properties
/// IDL syntax: WIDGET_CONTROL, id [, /REALIZE] [, /DESTROY] [, SET_VALUE=value] [, /SENSITIVE]
pub fn widget_control(
    args: &[XdlValue],
    keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "WIDGET_CONTROL: Expected widget ID".to_string(),
        ));
    }

    let id = value_to_usize(&args[0]).ok_or_else(|| {
        XdlError::TypeMismatch {
            expected: "integer".to_string(),
            actual: format!("{:?}", args[0]),
        }
    })?;

    let realize = keywords.contains_key("REALIZE");
    let destroy = keywords.contains_key("DESTROY");
    let _set_value = keywords.get("SET_VALUE");
    let sensitive = keywords.get("SENSITIVE");
    let map = keywords.get("MAP");

    if destroy {
        println!("WIDGET_CONTROL: Destroying widget {}", id);
        // Remove widget from store
        let mut store = WIDGET_STORE.lock().unwrap();
        if let Some(ref mut map) = *store {
            map.remove(&id);
        }
        return Ok(XdlValue::Undefined);
    }

    if realize {
        update_widget(id, |w| w.realized = true);
        println!("WIDGET_CONTROL: Realizing widget {}", id);
    }

    if let Some(sens_val) = sensitive {
        let is_sensitive = match sens_val {
            XdlValue::Int(i) => *i != 0,
            XdlValue::Long(l) => *l != 0,
            _ => true,
        };
        update_widget(id, |w| w.sensitive = is_sensitive);
        println!(
            "WIDGET_CONTROL: Setting widget {} sensitive={}",
            id, is_sensitive
        );
    }

    if let Some(map_val) = map {
        let is_mapped = match map_val {
            XdlValue::Int(i) => *i != 0,
            XdlValue::Long(l) => *l != 0,
            _ => true,
        };
        update_widget(id, |w| w.visible = is_mapped);
        println!("WIDGET_CONTROL: Setting widget {} visible={}", id, is_mapped);
    }

    Ok(XdlValue::Undefined)
}

/// WIDGET_INFO - Get widget information
/// IDL syntax: result = WIDGET_INFO(id [, /VALID_ID] [, /PARENT] [, /TYPE])
pub fn widget_info(args: &[XdlValue], keywords: &HashMap<String, XdlValue>) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "WIDGET_INFO: Expected widget ID".to_string(),
        ));
    }

    let id = value_to_usize(&args[0]).ok_or_else(|| {
        XdlError::TypeMismatch {
            expected: "integer".to_string(),
            actual: format!("{:?}", args[0]),
        }
    })?;

    let valid_id = keywords.contains_key("VALID_ID");
    let get_parent = keywords.contains_key("PARENT");
    let get_type = keywords.contains_key("TYPE");
    let get_uvalue = keywords.contains_key("UVALUE");

    let widget = get_widget(id);

    if valid_id {
        return Ok(XdlValue::Long(if widget.is_some() { 1 } else { 0 }));
    }

    if let Some(w) = widget {
        if get_parent {
            return Ok(XdlValue::Long(w.parent_id.unwrap_or(0) as i32));
        }
        if get_type {
            return Ok(XdlValue::String(w.widget_type.name().to_string()));
        }
        if get_uvalue {
            return Ok(w.uvalue.unwrap_or(XdlValue::Undefined));
        }

        // Default: return basic info
        println!("WIDGET_INFO: Widget {} is {:?}", id, w.widget_type);
        Ok(XdlValue::Long(id as i32))
    } else {
        if valid_id {
            return Ok(XdlValue::Long(0));
        }
        Err(XdlError::RuntimeError(format!("Widget {} not found", id)))
    }
}

/// XMANAGER - Register and manage widget hierarchy
/// IDL syntax: XMANAGER, name, id [, /NO_BLOCK] [, EVENT_HANDLER=handler]
pub fn xmanager(args: &[XdlValue], keywords: &HashMap<String, XdlValue>) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "XMANAGER: Expected name and widget ID".to_string(),
        ));
    }

    let name = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => "unknown".to_string(),
    };

    let id = value_to_usize(&args[1]).ok_or_else(|| {
        XdlError::TypeMismatch {
            expected: "integer".to_string(),
            actual: format!("{:?}", args[1]),
        }
    })?;

    let no_block = keywords.contains_key("NO_BLOCK");
    let has_handler = keywords.contains_key("EVENT_HANDLER");

    println!("XMANAGER: Registering '{}' with widget {}", name, id);
    println!(
        "  Options: no_block={}, event_handler={}",
        no_block, has_handler
    );

    // Mark widget as realized
    update_widget(id, |w| w.realized = true);

    if !no_block {
        println!("XMANAGER: In CLI mode, event loop is not available.");
        println!("  Use xdl-gui for interactive widget applications.");
    }

    Ok(XdlValue::Undefined)
}

/// WIDGET_EVENT - Wait for widget event (placeholder)
/// IDL syntax: event = WIDGET_EVENT(widget_id [, /NOWAIT])
pub fn widget_event(
    args: &[XdlValue],
    keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "WIDGET_EVENT: Expected widget ID".to_string(),
        ));
    }

    let id = value_to_usize(&args[0]).ok_or_else(|| {
        XdlError::TypeMismatch {
            expected: "integer".to_string(),
            actual: format!("{:?}", args[0]),
        }
    })?;

    let nowait = keywords.contains_key("NOWAIT");

    println!("WIDGET_EVENT: Waiting for event on widget {} (nowait={})", id, nowait);
    println!("  Note: Event handling requires xdl-gui. Returning empty event.");

    // Return an empty event structure
    let mut event = HashMap::new();
    event.insert("ID".to_string(), XdlValue::Long(0));
    event.insert("TOP".to_string(), XdlValue::Long(id as i32));
    event.insert("HANDLER".to_string(), XdlValue::Long(0));

    Ok(XdlValue::Struct(event))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_base() {
        let args = vec![];
        let mut keywords = HashMap::new();
        keywords.insert("TITLE".to_string(), XdlValue::String("Test".to_string()));

        let result = widget_base(&args, &keywords);
        assert!(result.is_ok());
        if let XdlValue::Long(id) = result.unwrap() {
            assert!(id > 0);
        }
    }

    #[test]
    fn test_widget_info_valid() {
        // Create a widget first
        let args = vec![];
        let keywords = HashMap::new();
        let result = widget_base(&args, &keywords).unwrap();

        if let XdlValue::Long(id) = result {
            let args = vec![XdlValue::Long(id)];
            let mut keywords = HashMap::new();
            keywords.insert("VALID_ID".to_string(), XdlValue::Long(1));

            let info = widget_info(&args, &keywords).unwrap();
            if let XdlValue::Long(valid) = info {
                assert_eq!(valid, 1);
            }
        }
    }
}

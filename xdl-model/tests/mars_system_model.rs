use std::fs;

use xdl_model::{Model, ModelDocument, ModelLayout};

#[test]
fn mars_system_model_fixture_is_typed_and_acyclic() {
    let path = format!(
        "{}/../examples/mars_helicopter_system_level.model.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let fixture = fs::read_to_string(path).expect("Mars model fixture");
    let model: Model = serde_json::from_str(&fixture).expect("valid Mars model JSON");

    model.validate().expect("valid typed Mars model graph");
    let order = model.execution_order().expect("acyclic execution order");
    assert_eq!(order.len(), 10);
    assert_eq!(model.name, "mars_helicopter_system_level");
    assert_eq!(
        model.blocks[&xdl_model::BlockId::new(1)].name,
        "SixCellBatteryPack"
    );
    assert_eq!(
        model.blocks[&xdl_model::BlockId::new(7)].name,
        "CoaxialRotorGearbox"
    );
}

#[test]
fn mars_system_model_fixture_renders_as_a_structural_svg() {
    let path = format!(
        "{}/../examples/mars_helicopter_system_level.model.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let fixture = fs::read_to_string(path).expect("Mars model fixture");
    let model: Model = serde_json::from_str(&fixture).expect("valid Mars model JSON");
    let layout = ModelLayout::from_model(&model).expect("generated model layout");
    let document = ModelDocument::new(model, layout).expect("valid model document");
    let svg = document.to_svg().expect("rendered structural SVG");

    assert_eq!(svg.matches("<rect ").count(), 10);
    assert_eq!(svg.matches("<line ").count(), 9);
    assert!(svg.contains("SixCellBatteryPack"));
    assert!(svg.contains("CoaxialRotorGearbox"));
}

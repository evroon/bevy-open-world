pub mod paint {
    use bevy::color::{Color, ColorToComponents, Srgba};
    use usvg::BaseGradient;

    use crate::Convert;

    trait ToF32Array {
        fn to_f32_array(&self) -> [f32; 4];
    }

    impl ToF32Array for Option<&usvg::Stop> {
        fn to_f32_array(&self) -> [f32; 4] {
            self.map(Convert::convert)
                .unwrap_or(Color::NONE)
                .to_srgba()
                .to_f32_array()
        }
    }

    pub fn avg_gradient(gradient: &BaseGradient) -> Color {
        let first = gradient.stops().first().to_f32_array();
        let last = gradient.stops().last().to_f32_array();
        let avg = [
            first[0] + last[0],
            first[1] + last[1],
            first[2] + last[2],
            first[3] + last[3],
        ]
        .map(|x| x / 2.0);
        Color::Srgba(Srgba::from_f32_array(avg))
    }
}

use std::error::Error;
use xmltree::{Element, EmitterConfig, XMLNode};

/// Parses an SVG from a byte slice, replaces all `id` attributes with
/// the value of `inkscape:label` (if present), and returns the modified SVG as bytes.
///
/// xmltree already renames `inkscape:label` to `label`, so we take that.
pub(crate) fn update_svg_ids_from_labels(svg_bytes: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    fn process_element(element: &mut Element) {
        if let Some(label) = element.attributes.get("label").cloned() {
            element.attributes.insert("id".to_string(), label);
        }
        for child in &mut element.children {
            if let XMLNode::Element(child_element) = child {
                process_element(child_element);
            }
        }
    }

    let mut reader = svg_bytes;
    let mut root_element = Element::parse(&mut reader)?;

    process_element(&mut root_element);

    let mut output_bytes = Vec::new();
    root_element.write_with_config(
        &mut output_bytes,
        EmitterConfig {
            perform_indent: true,
            ..Default::default()
        },
    )?;

    Ok(output_bytes)
}

#[cfg(test)]
mod tests {
    use xmltree::Element;

    use crate::util::update_svg_ids_from_labels;

    #[test]
    fn test_replace_id_by_inkscape_label() {
        let svg_example = include_bytes!("../../../assets/tests/svg-example.svg");
        let mut svg_expected: &[u8] =
            include_bytes!("../../../assets/tests/svg-example-id-replaced.svg");
        let mut processed: &[u8] = &mut update_svg_ids_from_labels(svg_example).unwrap();

        let processed_element = Element::parse(&mut processed).unwrap();
        let expected_element = Element::parse(&mut svg_expected).unwrap();

        assert_eq!(processed_element, expected_element);
    }
}

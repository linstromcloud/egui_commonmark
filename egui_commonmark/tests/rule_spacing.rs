use egui::epaint::Shape;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

/// Gaps from the bottom of the text above the rule to the rule, and from the
/// rule to the top of the text below it.
fn rule_gaps(md: &str) -> (f32, f32) {
    let ctx = egui::Context::default();
    let mut cache = CommonMarkCache::default();
    let mut output = None;
    // The second pass lays out with sizes measured in the first.
    for _ in 0..2 {
        output = Some(ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                CommonMarkViewer::new().show(ui, &mut cache, md);
            });
        }));
    }

    fn walk(shape: &Shape, texts: &mut Vec<egui::Rect>, rule: &mut Option<f32>) {
        match shape {
            Shape::Vec(shapes) => shapes.iter().for_each(|s| walk(s, texts, rule)),
            Shape::Text(t) if !t.galley.text().trim().is_empty() => {
                texts.push(t.visual_bounding_rect())
            }
            Shape::LineSegment { points, .. } if points[0].y == points[1].y => {
                rule.get_or_insert(points[0].y);
            }
            _ => {}
        }
    }
    let mut texts = vec![];
    let mut rule = None;
    for clipped in &output.unwrap().shapes {
        walk(&clipped.shape, &mut texts, &mut rule);
    }
    let rule = rule.expect("no rule painted");
    let above = texts
        .iter()
        .filter(|r| r.max.y < rule)
        .map(|r| r.max.y)
        .fold(f32::MIN, f32::max);
    let below = texts
        .iter()
        .filter(|r| r.min.y > rule)
        .map(|r| r.min.y)
        .fold(f32::MAX, f32::min);
    (rule - above, below - rule)
}

#[test]
fn rule_spacing_is_balanced() {
    for md in [
        "text\n\n---\n\nafter\n",
        "- item\n---\nafter\n",
        "- item\n\n---\n\n- item\n",
        "# Heading\n\n---\n\nafter\n",
    ] {
        let (above, below) = rule_gaps(md);
        assert!(
            (above - below).abs() <= 2.0,
            "{md:?}: {above} px above the rule, {below} px below"
        );
    }
}

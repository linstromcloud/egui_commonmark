use egui::epaint::Shape;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

const MD: &str = "Short
: alpha body

Medium term
: beta body
: delta body

A considerably longer term that runs past a third of the width
: gamma body
";

/// First-line rect of every painted text run, keyed by its text, plus the
/// body-text line height.
fn layout(md: &str, width: f32) -> (Vec<(String, egui::Rect)>, f32) {
    let ctx = egui::Context::default();
    let mut cache = CommonMarkCache::default();
    let input = || egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(width, 800.0),
        )),
        ..Default::default()
    };
    let mut output = None;
    let mut body_height = 0.0;
    // The second pass lays out with sizes measured in the first.
    for _ in 0..2 {
        output = Some(ctx.run(input(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                body_height = ui.text_style_height(&egui::TextStyle::Body);
                CommonMarkViewer::new().show(ui, &mut cache, md);
            });
        }));
    }

    fn walk(shape: &Shape, texts: &mut Vec<(String, egui::Rect)>) {
        match shape {
            Shape::Vec(shapes) => shapes.iter().for_each(|s| walk(s, texts)),
            Shape::Text(t) if !t.galley.text().trim().is_empty() => {
                let row = t.galley.rows[0].rect().translate(t.pos.to_vec2());
                texts.push((t.galley.text().trim().to_owned(), row));
            }
            _ => {}
        }
    }
    let mut texts = vec![];
    for clipped in &output.unwrap().shapes {
        walk(&clipped.shape, &mut texts);
    }
    (texts, body_height)
}

fn rect(texts: &[(String, egui::Rect)], text: &str) -> egui::Rect {
    texts
        .iter()
        .find(|(t, _)| t == text)
        .unwrap_or_else(|| panic!("{text:?} not painted; painted: {texts:?}"))
        .1
}

#[test]
fn definition_list_renders_as_two_columns() {
    let (texts, gap) = layout(MD, 600.0);
    let short = rect(&texts, "Short");
    let medium = rect(&texts, "Medium term");
    let long = rect(
        &texts,
        "A considerably longer term that runs past a third of the width",
    );
    let [alpha, beta, delta, gamma] =
        ["alpha body", "beta body", "delta body", "gamma body"].map(|t| rect(&texts, t));

    // The left column fits the widest term within the cap.
    let column = beta.min.x - short.min.x;
    assert!(
        (column - (medium.width() + gap)).abs() <= 1.0,
        "column {column} px, widest fitting term {} px + gap {gap} px",
        medium.width()
    );
    for (name, def) in [("alpha", alpha), ("delta", delta), ("gamma", gamma)] {
        assert_eq!(def.min.x, beta.min.x, "{name} starts off the column edge");
    }

    // A term and its first definition share a first line.
    for (term, def) in [(short, alpha), (medium, beta)] {
        assert!((term.max.y - def.max.y).abs() <= 1.0, "{term:?} vs {def:?}");
    }
    // A second definition stacks below the first.
    assert!(delta.min.y >= beta.max.y);

    // A term past a third of the width takes its own line; its definition
    // starts below it at the column edge.
    assert!(long.width() > 600.0 / 3.0);
    assert!(gamma.min.y >= long.max.y, "{gamma:?} overlaps {long:?}");
}

#[test]
fn definition_paragraphs_stack() {
    // The list is the first element, where the viewer suppresses the leading
    // newline.
    let (texts, _) = layout("Term\n: first paragraph\n\n  second paragraph\n", 600.0);
    let first = rect(&texts, "first paragraph");
    let second = rect(&texts, "second paragraph");
    assert_eq!(second.min.x, first.min.x);
    assert!(second.min.y >= first.max.y, "{second:?} overlaps {first:?}");
}

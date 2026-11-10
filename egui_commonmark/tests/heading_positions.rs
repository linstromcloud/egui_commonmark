use std::cell::RefCell;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

#[test]
fn headings_report_source_ranges_and_laid_out_positions_without_changing_layout() {
    let text = "# First\n\nParagraph.\n\n> ## Nested\n\nSetext\n======\n\n```md\n# Code\n```\n\n# First\n";
    let render = |observe: bool| {
        let ctx = egui::Context::default();
        let headings = RefCell::new(Vec::new());
        let mut height = 0.0;
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let on_heading = |span, level, position| {
                    headings.borrow_mut().push((span, level, position));
                };
                height = CommonMarkViewer::new()
                    .on_heading(observe.then_some(&on_heading))
                    .show(ui, &mut CommonMarkCache::default(), text)
                    .response.rect.height();
            });
        });
        (headings.into_inner(), height)
    };
    let (headings, height) = render(true);
    assert_eq!(height, render(false).1);
    assert_eq!(headings.len(), 4);
    assert_eq!(headings.iter().map(|h| h.1).collect::<Vec<_>>(), [1, 2, 1, 1]);
    for pair in headings.windows(2) {
        assert!(pair[0].0.start < pair[1].0.start);
        assert!(pair[0].2.y < pair[1].2.y);
    }
    assert!(text[headings[2].0.clone()].starts_with("Setext"));
}

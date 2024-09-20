use std::sync::{Arc, RwLock};

use dagrid_core::control::ControlGraph;
use nih_plug::context::gui::GuiContext;
use nih_plug_vello::vello::kurbo::*;
use nih_plug_vello::vello::peniko::*;
use nih_plug_vello::vello::Scene;
use nih_plug_vello::SimpleText;
use nih_plug_vello::VelloContext;

pub(crate) struct Ctx {
    control_graph: Arc<RwLock<ControlGraph>>,
    time: std::time::SystemTime,
    text: SimpleText,
}

impl Ctx {
    pub fn new(control_graph: Arc<RwLock<ControlGraph>>) -> Self {
        Self {
            control_graph,
            time: std::time::SystemTime::now(),
            text: SimpleText::new(),
        }
    }
}

pub(crate) fn draw(
    ctx: &mut Arc<RwLock<VelloContext<Ctx, Option<Arc<(dyn GuiContext + 'static)>>>>>,
    scene: &mut Scene,
) {
    let ctx = &mut ctx.write().unwrap().user;
    let time = ctx.time.elapsed().unwrap().as_secs_f64();

    let cg = ctx.control_graph.write().unwrap();

    let idxs = cg.get_node_indexes();

    let text_size = 15.0 + 3.0 * (time as f64).sin();

    let rect = Rect::from_origin_size(Point::new(0.0, 0.0), (1000.0, 1000.0));
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(Color::rgb8(128, 128, 128)),
        None,
        &rect,
    );

    for (i, id) in idxs.enumerate() {
        let text = &format!("{}", cg.get_node(id).get_ident());
        let val = cg.get_node_val(id);
        ctx.text.add(
            scene,
            None,
            text_size as f32,
            Some(&Brush::Solid(Color::hlc(
                (val.r() + 1.0) * 180.0,
                (val.l() + 1.0) * 50.0,
                127.0,
            ))),
            Affine::translate((200.0, 50.0 + text_size * 1.5 * i as f64))
                .then_rotate((time + i as f64).sin() / std::f64::consts::TAU),
            text,
        );
    }
}

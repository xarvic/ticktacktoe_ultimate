use crate::data::{Mark, FieldMeta};
use druid::{Widget, LifeCycle, EventCtx, PaintCtx, LifeCycleCtx, BoxConstraints, Size, LayoutCtx, Event, Env, UpdateCtx, RenderContext, Color, Rect};
use druid::piet::{StrokeStyle, LineCap};
use druid::kurbo::Line;

pub fn draw_mark(ctx: &mut PaintCtx, bounds: Rect, line_width: f64, alpha: f64, mark: Mark) {
    match mark {
        Mark::Cross => {
            let brush = ctx.solid_brush(Color::RED.with_alpha(alpha));

            let mut stroke_style = StrokeStyle::new();
            stroke_style.line_cap = Some(LineCap::Round);

            ctx.stroke_styled(Line::new((bounds.x0, bounds.y0), (bounds.x1, bounds.y1)), &brush, line_width, &stroke_style);
            ctx.stroke_styled(Line::new((bounds.x0, bounds.y1), (bounds.x1, bounds.y0)), &brush, line_width, &stroke_style);
        }
        Mark::Circle => {
            let brush = ctx.solid_brush(Color::BLUE.with_alpha(alpha));

            ctx.stroke(bounds.to_ellipse(), &brush, line_width);

        }
    };
}

pub struct FieldWidget {
    won: Option<Mark>,
    hover: Option<(usize, usize)>,
}

impl FieldWidget {
    pub fn new() -> Self {
        FieldWidget {
            hover: None,
            won: None,
        }
    }

    pub fn draw_mark(&self, ctx: &mut PaintCtx, index: (usize, usize), mark: Mark, preview: bool) {
        let line_width = ctx.size().width / 30.0 as f64;
        let slot_size = ctx.size().width / 3.0 as f64;

        let bounds = Rect::new(
            index.0 as f64 * slot_size + line_width * 2.0,
            index.1 as f64 * slot_size + line_width * 2.0,
            (index.0 + 1) as f64 * slot_size - line_width * 2.0,
            (index.1 + 1) as f64 * slot_size - line_width * 2.0,
        );

        let alpha = if preview {
            0.5
        } else {
            1.0
        };

        draw_mark(ctx, bounds, line_width, alpha, mark);
    }
}

impl Widget<FieldMeta> for FieldWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut FieldMeta, _: &Env) {
        let slot_size = ctx.size().width / 3.0;

        match event {
            Event::MouseDown(_) => {
                if let Some(position) = self.hover {
                    if data[position].is_none() {
                        data.set(position);
                    }
                }
            }
            Event::MouseMove(me) => {
                let new_hover = (
                    (me.pos.x / slot_size) as usize,
                    (me.pos.y / slot_size) as usize
                );

                if Some(new_hover) != self.hover &&
                    me.pos.x > 0.0 && me.pos.y > 0.0 &&
                    me.pos.x < ctx.size().width && me.pos.y < ctx.size().height
                {
                    self.hover = Some(new_hover);
                    ctx.request_paint();
                }
            }
            _ => {}
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, _: &FieldMeta, _: &Env) {
        match event {
            LifeCycle::HotChanged(change) if *change == false => {
                self.hover = None;
                ctx.request_paint();
            }
            _ => {}
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _: &FieldMeta, data: &FieldMeta, _: &Env) {
        ctx.request_paint();
        self.won = data.finished();
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &FieldMeta, _env: &Env) -> Size {
        ctx.set_paint_insets(20.0);
        bc.constrain_aspect_ratio(1.0, 250.0)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &FieldMeta, env: &Env) {
        let size = ctx.size().width;
        let line_width = ctx.size().width / 90.0;
        let slot_size = ctx.size().width / 3.0;

        if data.is_active() {
            let shape = ctx.size()
                .to_rect()
                .inset(20.0)
                .to_rounded_rect(slot_size / 3.0);
            let brush = ctx.solid_brush(env.get(druid::theme::BACKGROUND_LIGHT));

            ctx.fill(shape, &brush);
        }

        let black = ctx.solid_brush(Color::BLACK);

        //Vertical
        ctx.stroke(Line::new((0.0, slot_size), (size, slot_size)), &black, line_width);
        ctx.stroke(Line::new((0.0, slot_size * 2.0), (size, slot_size * 2.0)), &black, line_width);

        //Horizontal
        ctx.stroke(Line::new((slot_size, 0.0), (slot_size, size)), &black, line_width);
        ctx.stroke(Line::new((slot_size * 2.0, 0.0), (slot_size * 2.0, size)), &black, line_width);

        if let Some(index) = self.hover {
            if data[index].is_none() && data.is_active() {
                self.draw_mark(ctx, index, data.next_turn(), true);
            }
        }

        for x in 0..3_usize {
            for y in 0..3_usize {
                if let Some(mark) = data[(x, y)] {
                    self.draw_mark(ctx, (x, y), mark, false);
                }
            }
        }

        if let Some(mark) = self.won {
            let shape = ctx.size()
                .to_rect()
                .inset(20.0)
                .to_rounded_rect(slot_size / 3.0);
            let brush = ctx.solid_brush(env.get(druid::theme::WINDOW_BACKGROUND_COLOR).with_alpha(0.7));

            ctx.fill(shape, &brush);

            let bounds = ctx.size().to_rect().inset(-line_width);
            let line_width = bounds.width() / 10.0;

            draw_mark(ctx, bounds.inset(-line_width), line_width, 1.0, mark);
        }
    }
}
use druid::{Widget, Lens, WidgetExt, UnitPoint, Color, RenderContext};
use crate::data::{GameData, FieldMeta};
use crate::ui::field::{FieldWidget, draw_mark};
use druid::lens::Map;
use druid::widget::{Flex, Painter, Label};
use druid::piet::{Text, TextLayoutBuilder};

mod field;

fn position_lens(x: usize, y: usize) -> impl Lens<GameData, FieldMeta> {
    let position = (x, y);
    Map::new(
        move|game_data|FieldMeta::from_data(game_data, position),
        move|game_data, field_meta|field_meta.write_back(game_data, position)
    )
}

pub fn row(y: usize) -> impl Widget<GameData> {
    Flex::row()
        .with_child(FieldWidget::new().lens(position_lens(0, y)))
        .with_spacer(60.0)
        .with_child(FieldWidget::new().lens(position_lens(1, y)))
        .with_spacer(60.0)
        .with_child(FieldWidget::new().lens(position_lens(2, y)))
}

pub fn main_ui() -> impl Widget<GameData> {
    let header = Flex::row()
        .with_child(Painter::new(|ctx, data: &GameData, _|{
            let mark = data.game.finished().unwrap_or(data.next_turn);
            draw_mark(ctx, ctx.size().to_rect().inset(-4.0), 4.0, 1.0, mark);

        }).fix_size(30.0, 30.0))
        .with_child(Label::dynamic(|a: &GameData, _|{
            if a.game.finished().is_some() {
                String::from("won the Game!")
            } else {
                String::from("'s turn")
            }
        }).with_text_size(20.0))
        .align_horizontal(UnitPoint::CENTER);

    let mut text = None;

    Flex::column()
        .with_spacer(20.0)
        .with_child(header)
        .with_spacer(30.0)
        .with_child(row(0))
        .with_spacer(60.0)
        .with_child(row(1))
        .with_spacer(60.0)
        .with_child(row(2))
        .with_spacer(40.0)
        .with_child(
            Painter::new(move|ctx, _data: &GameData, env|{

                    if text.is_none() {
                        text = Some(
                            ctx.text()
                                .new_text_layout("Reset")
                                .text_color(env.get(druid::theme::FOREGROUND_LIGHT))
                                .font(env.get(druid::theme::UI_FONT_BOLD).family, 20.0)
                                .build()
                                .unwrap()
                        );
                    }

                    let brush = if ctx.is_active() {
                        Color::GREEN.with_alpha(0.4)
                    } else if ctx.is_hot() {
                        Color::GREEN.with_alpha(0.8)
                    } else {
                        Color::GREEN.with_alpha(0.6)
                    };
                    let brush = ctx.solid_brush(brush);

                    let mut shape = ctx.size().to_rect();

                    if ctx.is_active() {
                        shape = shape.inset(-2.0);
                    }
                    ctx.fill(shape.to_rounded_rect(5.0), &brush);

                    ctx.draw_text(text.as_ref().unwrap(), (12.0, 3.0));
                })
                .fix_size(80.0, 35.0)
                .on_click(|_, data: &mut GameData, _|*data = GameData::new())
        )
        .padding((40.0, 0.0))
        .align_horizontal(UnitPoint::TOP)
}
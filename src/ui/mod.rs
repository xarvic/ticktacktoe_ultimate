use druid::{Widget, Lens, WidgetExt, Color, RenderContext, Data, UpdateCtx, Env};
use crate::data::{GameData, FieldMeta, Slot};
use crate::ui::field::{FieldWidget, draw_mark};
use druid::lens::Map;
use druid::widget::{Flex, Painter, Label, CrossAxisAlignment, MainAxisAlignment, Controller};
use druid::piet::{Text, TextLayoutBuilder, TextLayout};

mod field;

struct Client;

impl<W: Widget<GameData>> Controller<GameData, W> for Client {
    fn update(&mut self, child: &mut W, ctx: &mut UpdateCtx, old_data: &GameData, data: &GameData, env: &Env) {
        child.update(ctx, old_data, data, env);
        data.handle_opponent(ctx.get_external_handle());
    }
}

fn position_lens(x: usize, y: usize) -> impl Lens<GameData, FieldMeta> {
    let position = (x, y);
    Map::new(
        move|game_data|FieldMeta::from_data(game_data, position),
        move|game_data, field_meta|field_meta.write_back(game_data, position)
    )
}

pub fn row(y: usize) -> impl Widget<GameData> {
    Flex::row()
        .main_axis_alignment(MainAxisAlignment::Center)
        .must_fill_main_axis(true)
        .with_flex_child(FieldWidget::new().lens(position_lens(0, y)), 1.0)
        .with_spacer(60.0)
        .with_flex_child(FieldWidget::new().lens(position_lens(1, y)), 1.0)
        .with_spacer(60.0)
        .with_flex_child(FieldWidget::new().lens(position_lens(2, y)), 1.0)
}

pub fn main_ui() -> impl Widget<GameData> {
    let header = Flex::row()
        .with_child(Painter::new(|ctx, data: &GameData, _|{
            let mark = data.game.belongs_to().unwrap_or(data.next_turn);
            draw_mark(ctx, ctx.size().to_rect().inset(-4.0), 4.0, 1.0, mark);

        }).fix_size(30.0, 30.0))
        .with_child(Label::dynamic(|a: &GameData, _|{
            if a.game.belongs_to().is_some() {
                String::from("won the Game!")
            } else {
                String::from("'s turn")
            }
        }).with_text_size(20.0));

    let footer = Flex::row()
        .with_child(
            colored_button(Color::GREEN, "AI Easy", |data: &mut GameData|*data = GameData::ai(1))
        )
        .with_spacer(10.0)
        .with_child(
            colored_button(Color::GREEN, "AI Hard", |data: &mut GameData|*data = GameData::ai(3))
        )
        .with_spacer(10.0)
        .with_child(
            colored_button(Color::GREEN, "2 Players", |data: &mut GameData|*data = GameData::local())
        );

    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .with_spacer(20.0)
        .with_child(header)
        .with_spacer(30.0)
        .with_flex_child(row(0), 1.0)
        .with_spacer(60.0)
        .with_flex_child(row(1), 1.0)
        .with_spacer(60.0)
        .with_flex_child(row(2), 1.0)
        .with_spacer(40.0)
        .with_child(footer)
        .with_spacer(10.0)
        .padding((40.0, 0.0))
        .controller(Client)
}

fn colored_button<T: Data>(color: Color, string: &'static str, f: impl Fn(&mut T) + 'static) -> impl Widget<T> {
    let mut text = None;


    Painter::new(move|ctx, _data: &T, env|{

        if text.is_none() {
            text = Some(
            ctx.text()
            .new_text_layout(string)
            .text_color(env.get(druid::theme::FOREGROUND_LIGHT))
            .font(env.get(druid::theme::UI_FONT_BOLD).family, 20.0)
            .build()
            .unwrap()
            );
        }

        let brush = if ctx.is_active() {
            color.clone().with_alpha(0.4)
        } else if ctx.is_hot() {
            color.clone().with_alpha(0.8)
        } else {
            color.clone().with_alpha(0.6)
        };
        let brush = ctx.solid_brush(brush);

        let mut shape = ctx.size().to_rect();

        if ctx.is_active() {
        shape = shape.inset(-2.0);
        }
        ctx.fill(shape.to_rounded_rect(5.0), &brush);

        ctx.draw_text(text.as_ref().unwrap(), ((90.0 - text.as_ref().unwrap().size().width) / 2.0, 3.0));
    })
    .fix_size(90.0, 35.0)
    .on_click(move |_, data: &mut T, _|f(data))
}

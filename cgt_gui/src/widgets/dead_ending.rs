use crate::{GuiContext, IsCgtWindow, TitledWindow, UpdateKind, impl_titled_window};
use cgt::misere::game_form::{
    DeadEndingContext, DeadEndingFormContext, GameFormContext, ParseError, StandardFormContext,
};
use imgui::{Condition, ImColor32};
use std::{error::Error, fmt::Write};

#[derive(Debug, Clone)]
struct FormInput<G> {
    form: Result<G, String>,
    value_input: String,
    scratch_buffer: String,
}

impl<G> FormInput<G> {
    fn new<C>(context: &C) -> Self
    where
        C: GameFormContext<Form = G>,
    {
        let form = context.new_integer(0).unwrap();
        Self {
            form: Ok(form),
            value_input: String::from("0"),
            scratch_buffer: String::with_capacity(32),
        }
    }

    fn draw<C>(&mut self, label: &str, context: &C, ui: &imgui::Ui)
    where
        C: GameFormContext<Form = G>,
        ParseError<C::DicoticConstructionError, C::IntegerConstructionError>: Error,
    {
        if ui.input_text(label, &mut self.value_input).build() {
            self.form = context
                .from_str(&self.value_input)
                .map_err(|err| err.to_string());
        }

        match &self.form {
            Ok(g) => {
                write!(
                    self.scratch_buffer,
                    "P-Free {}: {}",
                    context.display(g),
                    context.is_p_free(g),
                )
                .unwrap();
                ui.text(&self.scratch_buffer);
                self.scratch_buffer.clear();
            }
            Err(err) => {
                ui.text_colored(ImColor32::from_rgb(0xdd, 0x00, 0x00).to_rgba_f32s(), err);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeadEndingWindow<C = DeadEndingFormContext<StandardFormContext>>
where
    C: GameFormContext,
{
    lhs: FormInput<C::Form>,
    rhs: FormInput<C::Form>,
    context: C,
}

impl DeadEndingWindow {
    pub fn new() -> DeadEndingWindow {
        let context = DeadEndingFormContext::new(StandardFormContext);
        DeadEndingWindow {
            lhs: FormInput::new(&context),
            rhs: FormInput::new(&context),
            context,
        }
    }
}

impl IsCgtWindow for TitledWindow<DeadEndingWindow> {
    impl_titled_window!("Dead-ending");

    fn initialize(&mut self, _ctx: &GuiContext) {}

    fn draw(&mut self, ui: &imgui::Ui, ctx: &mut GuiContext) {
        ui.window(&self.title)
            .position(ui.io().mouse_pos, Condition::Appearing)
            .size([600.0, 250.0], Condition::Appearing)
            .bring_to_front_on_focus(true)
            .menu_bar(true)
            .opened(&mut self.is_open)
            .build(|| {
                if let Some(_menu_bar) = ui.begin_menu_bar() {
                    if let Some(_new_menu) = ui.begin_menu("New") {
                        if ui.menu_item("Duplicate") {
                            let w = self.content.clone();
                            ctx.new_windows
                                .push(Box::new(TitledWindow::without_title(w)));
                        }
                    }
                }
                self.content.lhs.draw("G", &self.content.context, ui);
                self.content.rhs.draw("H", &self.content.context, ui);

                if let Ok(lhs) = &self.content.lhs.form
                    && let Ok(rhs) = &self.content.rhs.form
                {
                    write!(
                        self.scratch_buffer,
                        "{} >= {} (mod E): {}",
                        self.content.context.display(lhs),
                        self.content.context.display(rhs),
                        self.content.context.ge_mod_dead_ending(lhs, rhs)
                    )
                    .unwrap();
                    ui.text(&self.scratch_buffer);
                    self.scratch_buffer.clear();

                    write!(
                        self.scratch_buffer,
                        "{} >= {} (mod E): {}",
                        self.content.context.display(rhs),
                        self.content.context.display(lhs),
                        self.content.context.ge_mod_dead_ending(rhs, lhs)
                    )
                    .unwrap();
                    ui.text(&self.scratch_buffer);
                    self.scratch_buffer.clear();
                }
            });
    }

    fn update(&mut self, _update: UpdateKind) {}
}

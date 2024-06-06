use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::*;
use bevy_fluent::Localization;

use crate::{
    localization::LocalizationExt,
    metadata::{ButtonStyle, FontStyle, GameMeta},
    utils::ResetController,
    GameState,
};

use super::{
    widgets::{bordered_button::BorderedButton, bordered_frame::BorderedFrame, EguiUIExt},
    EguiContextExt,
};

pub fn pause_menu(
    _commands: Commands,
    // mut egui_context: ResMut<EguiContext>,
    mut egui_context: Query<&mut EguiContext, With<PrimaryWindow>>,
    game: Res<GameMeta>,
    localization: Res<Localization>,
    reset_controller: ResetController,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let ui_theme = &game.ui_theme;
    let mut egui_context = egui_context.get_single_mut().unwrap();

    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(egui_context.get_mut(), |ui| {
            let screen_rect = ui.max_rect();

            let pause_menu_width = 300.0;
            let x_margin = (screen_rect.width() - pause_menu_width) / 2.0;
            let outer_margin = egui::style::Margin::symmetric(x_margin, screen_rect.height() * 0.2);

            BorderedFrame::new(&ui_theme.panel.border)
                .margin(outer_margin)
                .padding(ui_theme.panel.padding.into())
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());

                    let heading_font = ui_theme
                        .font_styles
                        .get(&FontStyle::Heading)
                        .expect("Missing 'heading' font style")
                        .colored(ui_theme.panel.font_color);

                    ui.vertical_centered(|ui| {
                        ui.themed_label(&heading_font, &localization.get("paused"));

                        ui.add_space(10.0);

                        let width = ui.available_width();

                        let continue_button = BorderedButton::themed(
                            ui_theme,
                            &ButtonStyle::Normal,
                            &localization.get("continue"),
                        )
                        .min_size(egui::vec2(width, 0.0))
                        .show(ui);

                        // Focus continue button by default
                        if ui.memory(|i| i.focus().is_none()) {
                            // if ui.memory().focus().is_none() {
                            continue_button.request_focus();
                        }

                        if continue_button.clicked() {
                            next_state.set(GameState::InGame);
                        }

                        if BorderedButton::themed(
                            ui_theme,
                            &ButtonStyle::Normal,
                            &localization.get("main-menu"),
                        )
                        .min_size(egui::vec2(width, 0.0))
                        .show(ui)
                        .clicked()
                        {
                            reset_controller.reset_world();

                            // Show the main menu
                            next_state.set(GameState::MainMenu);
                            ui.ctx().clear_focus();
                        }
                    });
                })
        });
}

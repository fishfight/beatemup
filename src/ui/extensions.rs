//! Extension traits for egui

use bevy_egui::egui;

/// Extension trait with helpers for the egui context
pub trait EguiContextExt {
    /// Clear the UI focus
    fn clear_focus(self);
}

impl EguiContextExt for &egui::Context {
    fn clear_focus(self) {
        self.memory_mut(|m| m.request_focus(egui::Id::null()));
    }
}

/// Extension trait with helpers for egui responses
pub trait EguiResponseExt {
    /// Set this response to focused if nothing else is focused
    fn focus_by_default(self, ui: &mut egui::Ui) -> egui::Response;
}

impl EguiResponseExt for egui::Response {
    fn focus_by_default(self, ui: &mut egui::Ui) -> egui::Response {
        if ui.ctx().memory_mut(|m| m.focus().is_none()) {
            ui.ctx().memory_mut(|m| m.request_focus(self.id));

            self
        } else {
            self
        }
    }
}

use crate::*;

#[derive(Resource, Default)]
pub(crate) struct Theme {
    is_dark: bool,
}

impl Theme {
    fn toggle(&mut self, camera: &mut Camera) {
        camera.clear_color = if self.is_dark {
            Color::srgb(0.22, 0.22, 0.22).into()
        } else {
            Color::srgb(0.75, 0.75, 0.75).into()
        };

        self.is_dark = !self.is_dark;
    }
}

pub(crate) fn set_theme(
    mut contexts: EguiContexts, 
    mut theme: ResMut<Theme>,
    mut query_camera: Query<&mut Camera>,
) {
    let ctx = contexts.ctx_mut();
    let mut camera = query_camera.single_mut();

    egui::Window::new("Theme")
        .anchor(egui::Align2::LEFT_TOP, [2.0, 2.0])
        .resizable(false)
        .movable(false)
        .title_bar(false)
        .show(ctx,  |ui| {
            // ui.visuals_mut().window_fill = egui::Color32::RED;
            // ui.visuals_mut().window_shadow = Shadow::NONE;

            let icon = if theme.is_dark { "☀" } else { "🌙" };
            if ui.button(icon).clicked() {
                theme.toggle(&mut camera);
            };
    });
}
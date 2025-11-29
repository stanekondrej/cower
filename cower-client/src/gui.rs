use crate::Args;

#[cfg(feature = "gui")]
#[derive(Default, Debug)]
struct App {}

#[cfg(feature = "gui")]
impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        use eframe::egui;

        egui::CentralPanel::default().show(ctx, |_ui| {});
    }
}

#[cfg(feature = "gui")]
pub(crate) fn open_gui(_args: &Args) -> eframe::Result {
    let options = eframe::NativeOptions {
        ..Default::default()
    };

    eframe::run_native(
        "Cower client",
        options,
        Box::new(|_cc| Ok(Box::<App>::default())),
    )
}

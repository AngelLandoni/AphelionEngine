use engine::{
    egui::{
        Color32, Grid, Id, Image, InnerResponse, Response, Rounding,
        ScrollArea, Sense, Ui,
    },
    graphics::scene::Scene,
    log::{debug, info},
    scene::scene_state::SceneState,
    wgpu_graphics::pipelines::sky_pipeline::SkyUpdater,
};
use shipyard::{AllStoragesViewMut, UniqueView, UniqueViewMut, World};

use super::asset_server_section::EguiAssetServer;

pub fn render_scene_config_section(ui: &mut Ui, world: &World) -> Response {
    let mut scene_state = world.borrow::<UniqueViewMut<SceneState>>().unwrap();
    let egui_assets_server =
        world.borrow::<UniqueView<EguiAssetServer>>().unwrap();

    ScrollArea::vertical().show(ui, |ui| {
        render_scene_component(
            ui,
            &mut scene_state.main,
            &egui_assets_server,
            world,
        );
        for (_, scene) in &mut scene_state.sub_scenes {
            render_scene_component(ui, scene, &egui_assets_server, world);
        }
    });

    ui.label("")
}

fn render_scene_component(
    ui: &mut Ui,
    scene: &mut Scene,
    egui_asset_server: &EguiAssetServer,
    world: &World,
) {
    ui.label(format!("{}", scene.label));

    let InnerResponse { inner, response } = ui.menu_button("Skybox", |ui| {
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .id_source("skybox texture inspector")
            .show(ui, |ui| {
                ui.set_height(250.0);

                Grid::new("scene_skybox_texture_grid").show(ui, |ui| {
                    let mut i = 1;
                    for (id, texture) in &egui_asset_server.textures {
                        let response = ui
                            .vertical(|ui| {
                                let image = Image::new((
                                    *texture,
                                    engine::egui::Vec2::new(100.0, 100.0),
                                ))
                                .rounding(Rounding::same(4.0));
                                // Render the scene.
                                ui.add(image);
                                ui.label(id)
                            })
                            .response;

                        let response = ui.interact(
                            response.rect,
                            Id::new(id),
                            Sense::click(),
                        );

                        let is_clicked = response.clicked();

                        response.on_hover_cursor(
                            engine::egui::CursorIcon::PointingHand,
                        );

                        if is_clicked {
                            ui.close_menu();
                            return Some(id);
                        }

                        if i % 3 == 0 {
                            ui.end_row();
                        }

                        i += 1;
                    }

                    None
                })
            })
    });

    response.on_hover_cursor(engine::egui::CursorIcon::PointingHand);

    if let Some(id) = inner.and_then(|i| i.inner.inner) {
        info!("Adding sky updater!");
        world.add_unique(SkyUpdater::new(
            id.to_owned(),
            "WorkbenchScene".to_owned(),
        ));
    }
}

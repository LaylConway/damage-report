use cgmath::{Vector2};
use input::{Input};
use palette::pixel::{Srgb};

use calcium_game::{AverageDelta, delta_to_fps};
use calcium_rendering::{Renderer, WindowRenderer, Error};
use calcium_rendering_simple2d::{Simple2DRenderTarget, Simple2DRenderer};
use calcium_ui::{UiRenderer, Ui, Element, ElementId};
use calcium_ui::style::{Style, Position, Lrtb, Size, SizeValue};

use model::{MapEditorModel};

pub struct UiView {
    ui_renderer: UiRenderer,

    ui: Ui,
    button_id: ElementId,
    fps_id: ElementId,

    average_delta: AverageDelta,
}

impl UiView {
    pub fn new<R: Renderer>(renderer: &mut R) -> Result<Self, Error> {
        let ui_renderer = UiRenderer::new();

        let mut ui = Ui::new();
        let root_id = ui.root_id();

        // Create the top ribbon
        let ribbon_style = Style {
            size: Size::new(SizeValue::Scale(1.0), SizeValue::Units(66.0)),
            background_color: Some(Srgb::new(0.18, 0.20, 0.21).into()),
            .. Style::new()
        };
        let ribbon = Element::new(ribbon_style.clone());
        let ribbon_id = ui.add_child(ribbon, root_id);

        // Add a few buttons to the top ribbon
        let button_style = Style {
            margin: Lrtb::uniform(3.0),
            size: Size::units(60.0, 60.0),
            background_color: Some(Srgb::new(0.53, 0.54, 0.52).into()),
            .. Style::new()
        };

        let button = Element::new(button_style.clone());
        let button_id = ui.add_child(button, ribbon_id);

        let button2 = Element::new(button_style.clone());
        ui.add_child(button2, ribbon_id);

        // Add a FPS label
        let fps = Element::new(Style {
            position: Position::Relative(Vector2::new(500.0, 0.0)),
            margin: Lrtb::uniform(3.0),
            size: Size::units(120.0, 18.0),
            text_color: Srgb::new(1.0, 1.0, 1.0).into(),
            .. Style::new()
        });
        let fps_id = ui.add_child(fps, root_id);

        Ok(UiView {
            ui_renderer,

            ui,
            button_id,
            fps_id,

            average_delta: AverageDelta::new(),
        })
    }

    pub fn handle_event<R: Renderer>(&mut self, event: &Input, window_renderer: &R::WindowRenderer) {
        let size = window_renderer.size();
    }

    pub fn update(&mut self, delta: f32, editor: &mut MapEditorModel) {
        self.average_delta.accumulate(delta);

        {
            let button = &mut self.ui[self.button_id];
            if button.clicked() {
                button.style.text = "1".to_string();
                editor.new_brush();
            }
        }

        {
            let fps = &mut self.ui[self.fps_id];
            fps.style.text = format!("{}", delta_to_fps(self.average_delta.get()));
        }
    }

    pub fn render<R: Renderer, SR: Simple2DRenderer<R>>(
        &mut self, frame: &mut R::Frame,
        renderer: &mut R, window_renderer: &mut R::WindowRenderer,
        simple2d_renderer: &mut SR,
        simple2d_rendertarget: &mut Simple2DRenderTarget<R, SR>,
    ) -> Result<(), Error> {
        let ui_batches = self.ui_renderer.draw(&mut self.ui, window_renderer.size().cast());

        simple2d_renderer.render(
            &ui_batches, simple2d_rendertarget,
            renderer, window_renderer, frame
        );

        Ok(())
    }

    pub fn cursor_over_ui(&self) -> bool {
        /*let widget = self.ui.global_input().current.widget_under_mouse;
        widget
            // If we're over a widget, pass through the background canvas
            .map(|w| w != self.ids.canvas)
            // If there no widget, we're not over ui either way
            .unwrap_or(false)*/
        false
    }
}

use ggez::{
    event::{EventHandler, MouseButton},
    graphics::{self, Canvas, Color, Drawable, Text, Transform, Rect, TextFragment, MeshBuilder, Mesh},
    mint::{Vector2, Point2},
    Context, GameError, input::{mouse, keyboard::KeyInput}, winit::event::VirtualKeyCode,
};

use crate::alg::{ALL_PLAYER_TYPES, PlayerTypeSupplier};

use super::chess_display::{PlayerType, ChessDisplay};

#[derive(Debug, Clone)]
struct Button {
    text: Text,
    color: Color,
    hover_color: Color,
    dims: Vector2<f32>,
    rect: graphics::Mesh,

    bounds: graphics::Rect,

    pos: Vector2<f32>,

    just_pressed: bool,
    hovered: bool,
}

impl Button {
    pub fn new(ctx: &mut Context, text: Text, color: Color, hover_color: Color, pos: Vector2<f32>) -> Self {
        const PADDING: f32 = 10.0;

        let dims = text.measure(ctx).unwrap();

        let bounds = graphics::Rect::new(
            -dims.x / 2.0 - PADDING,
            -dims.y / 2.0 - PADDING,
            dims.x + (PADDING * 2.0),
            dims.y + (PADDING * 2.0),
        );

        let rect = graphics::Mesh::new_rounded_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            bounds,
            10.0,
            Color::WHITE,
        )
        .unwrap();

        Button {
            text,
            color,
            hover_color,
            dims,
            rect,
            bounds,
            pos,
            just_pressed: false,
            hovered: false,
        }
    }

    pub fn set_pos(&mut self, pos: Vector2<f32>) {
        self.pos = pos;
    }

    pub fn process_click(&mut self, x: f32, y: f32, button: MouseButton) {
        if self.bounds.contains([x - self.pos.x, y - self.pos.y]) {
            self.just_pressed = true;
        }
    }

    pub fn process_hover(&mut self, x: f32, y: f32) {
        if self.bounds.contains([x - self.pos.x, y - self.pos.y]) {
            self.hovered = true;
        } else {
            self.hovered = false;
        }
    }

    pub fn just_pressed(&mut self) -> bool {
        let pressed = self.just_pressed;
        self.just_pressed = false;
        pressed
    }
}

impl Drawable for Button {
    fn draw(&self, canvas: &mut Canvas, param: impl Into<graphics::DrawParam>) {
        let param = param.into();

        let mut rect_params = param.clone().dest(self.pos);

        if self.hovered {
            rect_params = rect_params.color(self.hover_color);
        } else {
            rect_params = rect_params.color(self.color);
        }

        canvas.draw(&self.rect, rect_params);

        //Center on dest
        let text_x = self.pos.x - (self.dims.x / 2.0);
        let text_y = self.pos.y - (self.dims.y / 2.0);

        canvas.draw(
            &self.text,
            graphics::DrawParam::default().dest([text_x, text_y]),
        );
    }

    fn dimensions(
        &self,
        gfx: &impl ggez::context::Has<graphics::GraphicsContext>,
    ) -> Option<graphics::Rect> {
        Some(graphics::Rect::new(0.0, 0.0, self.dims.x, self.dims.y))
    }
}

pub struct PlayerTypePicker {
    name: Text,
    options: Vec<(PlayerTypeSupplier, Text)>,
    selected: usize,

    max_option_width: f32,
    scroll_offset: f32,

    list_region: Rect,
    just_clicked_list: bool,
}

impl PlayerTypePicker {
    pub fn new(ctx: &mut Context, name: &str) -> Self {
        let mut text = Text::new(
            TextFragment::new(name)
                .scale(75.0)
                .color(Color::new(0.7, 0.7, 0.7, 1.0))
        );

        let mut options = vec![];

        for (name, func) in ALL_PLAYER_TYPES.iter() {
            let mut text = Text::new(
                TextFragment::new(*name)
                    .scale(50.0)
                    .color(Color::new(0.5, 0.5, 0.5, 1.0))
            );

            options.push((*func, text));
        }

        let max_option_width = options.iter()
            .map(|(_, text)| text.measure(ctx).unwrap().x + 20.0)
            .reduce(|a, b| a.max(b))
            .unwrap_or(20.0);

        PlayerTypePicker {
            name: text,
            options,
            selected: 0,
            max_option_width,
            scroll_offset: 0.0,

            list_region: Rect::new(0.0, 0.0, 0.0, 0.0),
            just_clicked_list: false,
        }
    }

    pub fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas, bounds: Rect) -> Result<(), GameError> {
        let dims = self.name.measure(ctx)?;

        canvas.draw(
            &self.name,
            graphics::DrawParam::default().dest([
                bounds.x + (bounds.w / 2.0) - (dims.x / 2.0),
                bounds.y,
            ]),
        );

        self.list_region = Rect::new(bounds.x, bounds.y + dims.y, bounds.w, bounds.h - dims.y);

        canvas.set_scissor_rect(self.list_region)?;

        const PADDING: f32 = 8.0;

        let num_columns = (((bounds.w  - PADDING * 2.0) / self.max_option_width).floor() as usize).min(self.options.len());
        let total_width = num_columns as f32 * self.max_option_width;

        let base_x = bounds.x + (bounds.w / 2.0) - (total_width / 2.0);
        let base_y = bounds.y + dims.y + PADDING;

        const HEIGHT: f32 = 50.0;

        let total_height = (self.options.len() as f32 / num_columns as f32).ceil() * HEIGHT;
        let available_height = self.list_region.h - PADDING * 2.0;
        let max_scroll_offset = (total_height - available_height).max(0.0);

        self.scroll_offset = self.scroll_offset.min(max_scroll_offset).max(0.0);

        let base_y = base_y - self.scroll_offset;

        let mut xi = 0;
        let mut yi = 0;

        for (idx, (_, text)) in self.options.iter().enumerate() {
            let text_dims = text.measure(ctx)?;

            let base_cell_x = base_x + (xi as f32 * self.max_option_width);
            let base_cell_y = base_y + (yi as f32 * HEIGHT);

            let x = base_cell_x + (self.max_option_width - text_dims.x) / 2.0;
            let y = base_cell_y + (HEIGHT - text_dims.y) / 2.0;

            let bounds = graphics::Rect::new(
                base_cell_x,
                base_cell_y,
                self.max_option_width,
                HEIGHT,
            );

            let hovered = bounds.contains(ctx.mouse.position());
            let mut outline_color = None;

            if self.just_clicked_list && hovered {
                self.selected = idx;
            } else if hovered {
                outline_color = Some(Color::new(0.5, 0.5, 0.5, 1.0))
            }

            if idx == self.selected {
                outline_color = Some(Color::new(0.0, 0.0, 0.0, 1.0));
            }

            canvas.draw(
                text,
                graphics::DrawParam::default().dest([x, y]),
            );

            if let Some(col) = outline_color {
                let outline_bounds = graphics::Rect::new(
                    bounds.x + 4.0,
                    bounds.y + 2.0,
                    bounds.w - 8.0,
                    bounds.h - 4.0,
                );

                let rect = graphics::Mesh::new_rounded_rectangle(
                    ctx,
                    graphics::DrawMode::stroke(3.0),
                    outline_bounds,
                    5.0,
                    col
                )?;

                canvas.draw(
                    &rect,
                    graphics::DrawParam::default(),
                );
            }

            xi += 1;

            if xi >= num_columns {
                xi = 0;
                yi += 1;
            }
        }

        canvas.set_default_scissor_rect();

        //Draw rectangle border around list region
        let rect = graphics::Mesh::new_rounded_rectangle(
            ctx,
            graphics::DrawMode::stroke(3.0),
            self.list_region,
            5.0,
            Color::new(0.7, 0.7, 0.7, 1.0)
        )?;

        canvas.draw(&rect, graphics::DrawParam::default());

        self.just_clicked_list = false;

        Ok(())
    }

    pub fn mouse_wheel_event(&mut self, ctx: &mut Context, _x: f32, y: f32) {
        if self.list_region.contains(ctx.mouse.position()) {
            self.scroll_offset += y * 10.0;
        }
    }

    pub fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        //self.hovering_over_list = self.list_region.contains([x, y]);
    }

    pub fn mouse_button_down_event(&mut self, _ctx: &mut Context, _button: MouseButton, x: f32, y: f32) {
        if self.list_region.contains([x, y]) {
            self.just_clicked_list = true;
        }
    }

    pub fn get(&self, color: chess::Color) -> PlayerType {
        (self.options[self.selected].0)(color)
    }
}

enum State {
    MainMenu { new_game_button: Button },

    GameCreator {
        white_picker: PlayerTypePicker,
        black_picker: PlayerTypePicker,

        launch_button: Button,
    },

    Game {
        chess: ChessDisplay
    },
}

impl State {
    fn main_menu(ctx: &mut Context) -> Self {
        let mut text = Text::new("New Game");
        text.set_scale(50.0);

        let button = Button::new(
            ctx, 
            text, 
            Color::new(0.0, 0.0, 0.0, 1.0), 
            Color::new(0.1, 0.1, 0.1, 1.0),
            [0.0, 0.0].into()
        );

        State::MainMenu {
            new_game_button: button,
        }
    }

    fn game_creator(ctx: &mut Context) -> Self {
        let mut launch_text = Text::new("Start!");
        launch_text.set_scale(50.0);

        State::GameCreator {
            white_picker: PlayerTypePicker::new(ctx, "White"),
            black_picker: PlayerTypePicker::new(ctx, "Black"),

            launch_button: Button::new(
                ctx,
                launch_text,
                Color::new(0.0, 0.0, 0.0, 1.0),
                Color::new(0.1, 0.1, 0.1, 1.0),
                [0.0, 0.0].into()
            ),
        }
    }

    fn game(ctx: &mut Context, white: PlayerType, black: PlayerType) -> Self {
        State::Game {
            chess: ChessDisplay::new(ctx, white, black),
        }
    }

    pub fn update(&mut self, ctx: &mut Context) -> Result<Option<State>, GameError> {
        match self {
            State::MainMenu {new_game_button} => {
                if new_game_button.just_pressed() {
                    return Ok(Some(State::game_creator(ctx)));
                }
            }

            State::GameCreator {white_picker, black_picker, launch_button} => {
                if launch_button.just_pressed() {
                    return Ok(Some(State::game(
                        ctx, 
                        white_picker.get(chess::Color::White), 
                        black_picker.get(chess::Color::Black)
                    )));
                }
            }

            State::Game {chess} => {
                chess.update(ctx)?;
            }
        }

        Ok(None)
    }

    pub fn draw(
        &mut self,
        ctx: &mut Context,
        canvas: &mut Canvas,
    ) -> Result<Option<State>, GameError> {
        //get draw bounds
        let width = canvas.screen_coordinates().unwrap().w;
        let height = canvas.screen_coordinates().unwrap().h;

        match self {
            State::MainMenu {new_game_button} => {
                let mut title_text = Text::new("Chess Arena");
                title_text.set_scale(100.0);

                let measure = title_text.measure(ctx)?;
                let text_height = measure.y;
                let text_width = measure.x;

                let text_x = (width / 2.0) - (text_width / 2.0);
                let text_y = (height * 0.4) - text_height;

                canvas.draw(
                    &title_text,
                    graphics::DrawParam::default()
                        .dest([text_x, text_y])
                        .color(Color::from_rgb(255, 255, 255)),
                );

                new_game_button.set_pos([width / 2.0, height * 0.6].into());

                canvas.draw(
                    new_game_button,
                    graphics::DrawParam::default()
                        .color(Color::from_rgb(255, 255, 255)),
                );
            }

            State::GameCreator {white_picker, black_picker, launch_button} => {
                let mut title_text = Text::new("Game Creator");
                title_text.set_scale(100.0);

                let measure = title_text.measure(ctx)?;
                let text_height = measure.y;
                let text_width = measure.x;

                //Title should be 20px from the top and centered
                let text_x = (width / 2.0) - (text_width / 2.0);
                let text_y = 20.0;

                canvas.draw(
                    &title_text,
                    graphics::DrawParam::default()
                        .dest([text_x, text_y])
                        .color(Color::from_rgb(255, 255, 255)),
                );

                let top = text_height + 40.0;

                let halfway = width / 2.0;

                let white_bounds = Rect::new(10.0, top, halfway - 20.0, height - top - 100.0);
                let black_bounds = Rect::new(halfway + 10.0, top, halfway - 20.0, height - top - 100.0);

                white_picker.draw(ctx, canvas, white_bounds)?;
                black_picker.draw(ctx, canvas, black_bounds)?;

                launch_button.set_pos([width / 2.0, height - 50.0].into());

                canvas.draw(
                    launch_button,
                    graphics::DrawParam::default()
                        .color(Color::from_rgb(255, 255, 255)),
                );

                /*//Make a black line to separate the pickers
                let mut line = MeshBuilder::new();
                line.line(
                    &[
                        Point2 {x: halfway, y: white_bounds.top()},
                        Point2 {x: halfway, y: white_bounds.bottom()}
                    ],
                    2.0,
                    Color::from_rgb(0, 0, 0),
                )?;
                let line = line.build();
                let line = Mesh::from_data(ctx, line);

                canvas.draw(
                    &line,
                    graphics::DrawParam::default()
                );*/
            }

            State::Game {chess} => {
                chess.draw(ctx, canvas, 0.0, 0.0, width, height)?;
            }
        }

        Ok(None)
    }

    pub fn mouse_button_down_event(
        &mut self,
        ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> Result<Option<State>, GameError> {
        match self {
            State::MainMenu {new_game_button} => {
                new_game_button.process_click(x, y, button);
            }

            State::GameCreator {white_picker, black_picker, launch_button} => {
                white_picker.mouse_button_down_event(ctx, button, x, y);
                black_picker.mouse_button_down_event(ctx, button, x, y);
                launch_button.process_click(x, y, button);
            }

            State::Game {chess} => {
                chess.mouse_button_down_event(ctx, button, x, y)?;
            }
        }

        Ok(None)
    }

    pub fn mouse_motion_event(
        &mut self,
        ctx: &mut Context,
        x: f32,
        y: f32,
        dx: f32,
        dy: f32,
    ) -> Result<Option<State>, GameError> {
        match self {
            State::MainMenu {new_game_button} => {
                new_game_button.process_hover(x, y);
            }

            State::GameCreator {white_picker, black_picker, launch_button} => {
                white_picker.mouse_motion_event(ctx, x, y, dx, dy);
                black_picker.mouse_motion_event(ctx, x, y, dx, dy);
                launch_button.process_hover(x, y);
            }

            State::Game {..} => {}
        }

        Ok(None)
    }

    pub fn mouse_wheel_event(
        &mut self,
        ctx: &mut Context,
        x: f32,
        y: f32,
    ) -> Result<Option<State>, GameError> {
        match self {
            State::MainMenu {..} => {}

            State::GameCreator {white_picker, black_picker, ..} => {
                white_picker.mouse_wheel_event(ctx, x, y);
                black_picker.mouse_wheel_event(ctx, x, y);
            }

            State::Game {..} => {}
        }

        Ok(None)
    }
}

pub struct MainGUI {
    state: State,
}

impl MainGUI {
    pub fn new(ctx: &mut Context) -> Self {
        MainGUI {
            state: State::main_menu(ctx),
        }
    }

    fn state_change(&mut self, ctx: &mut Context, new_state: Option<State>) {
        if let Some(new_state) = new_state {
            self.state = new_state;
        }
    }
}

impl EventHandler for MainGUI {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        let res = self.state.update(ctx)?;
        self.state_change(ctx, res);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::new(0.2, 0.2, 0.2, 1.0));

        let res = self.state.draw(ctx, &mut canvas)?;
        self.state_change(ctx, res);

        canvas.finish(ctx)?;

        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> Result<(), GameError> {
        let res = self.state.mouse_button_down_event(ctx, button, x, y)?;
        self.state_change(ctx, res);

        Ok(())
    }

    fn mouse_motion_event(
            &mut self,
            ctx: &mut Context,
            x: f32,
            y: f32,
            dx: f32,
            dy: f32,
        ) -> Result<(), GameError> {
        let res = self.state.mouse_motion_event(ctx, x, y, dx, dy)?;
        self.state_change(ctx, res);

        Ok(())
    }

    fn mouse_wheel_event(&mut self, ctx: &mut Context, x: f32, y: f32) -> Result<(), GameError> {
        let res = self.state.mouse_wheel_event(ctx, x, y)?;
        self.state_change(ctx, res);

        Ok(())
    }

    fn key_down_event(
            &mut self,
            ctx: &mut Context,
            input: ggez::input::keyboard::KeyInput,
            _repeated: bool,
        ) -> Result<(), GameError> {
        match input.keycode {
            Some(VirtualKeyCode::Escape) => {
                self.state = State::main_menu(ctx);
            },

            _ => {}
        }

        Ok(())
    }
}

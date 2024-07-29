use eframe::egui;
use egui::{DragValue, Frame, Image, Pos2, Rect, ScrollArea, TextBuffer, Vec2};

/* TODO:
- Better other play movement and drawing entry
- Highlight moves
- Automatic testing
*/

const PIP_MAX_U8: u8 = 12;
const PIP_MAX_USIZE: usize = PIP_MAX_U8 as usize;
const PIP_MAX_FLOAT: f32 = PIP_MAX_U8 as f32;

const UNDO_SHORTCUT: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Z);

#[derive(Clone)]
struct DominoSet {
    /*
    DominoSet is implemented as a bit array (2x u128) to optimize for adding,
    removing, and testing a domino as well as copying the entire set.
    max: min
    low
     0: 0 . . . . . . . . .  .  .  . . . .
     1: 0 1 . . . . . . . .  .  .  . . . .
     2: 0 1 2 . . . . . . .  .  .  . . . .
     3: 0 1 2 3 . . . . . .  .  .  . . . .
     4: 0 1 2 3 4 . . . . .  .  .  . . . .
     5: 0 1 2 3 4 5 . . . .  .  .  . . . .
     6: 0 1 2 3 4 5 6 . . .  .  .  . . . .
     7: 0 1 2 3 4 5 6 7 . .  .  .  . . . .
    high
     8: 0 1 2 3 4 5 6 7 8 .  .  .  . . . .
     9: 0 1 2 3 4 5 6 7 8 9  .  .  . . . .
    10: 0 1 2 3 4 5 6 7 8 9 10  .  . . . .
    11: 0 1 2 3 4 5 6 7 8 9 10 11  . . . .
    12: 0 1 2 3 4 5 6 7 8 9 10 11 12 . . .
     */
    low: u128,
    high: u128,
}

impl DominoSet {
    fn clear(&mut self) {
        self.low = 0;
        self.high = 0;
    }
    fn fill(&mut self) {
        self.low = u128::MAX;
        self.high = u128::MAX;
    }
    fn has(&self, min: u8, max: u8) -> bool {
        debug_assert!(min <= max && max <= PIP_MAX_U8);
        // Check if corresponding bit is set.
        if max < 8 {
            (self.low & (1 << (max * 16 + min))) >> (max * 16 + min) != 0
        } else {
            (self.high & (1 << ((max - 8) * 16 + min))) >> ((max - 8) * 16 + min) != 0
        }
    }
    fn add(&mut self, min: u8, max: u8) {
        // Set corresponding bit to 1.
        debug_assert!(min <= max && max <= PIP_MAX_U8);
        if max < 8 {
            self.low |= 1 << (max * 16 + min);
        } else {
            self.high |= 1 << ((max - 8) * 16 + min);
        }
    }
    fn remove(&mut self, min: u8, max: u8) {
        debug_assert!(min <= max && max <= PIP_MAX_U8);
        // Set corresponding bit to 0.
        if max < 8 {
            self.low &= !(1 << (max * 16 + min));
        } else {
            self.high &= !(1 << ((max - 8) * 16 + min));
        }
    }
    fn toggle(&mut self, min: u8, max: u8) {
        debug_assert!(min <= max && max <= PIP_MAX_U8);
        // Set corresponding bit to 0.
        if max < 8 {
            self.low ^= 1 << (max * 16 + min);
        } else {
            self.high ^= 1 << ((max - 8) * 16 + min);
        }
    }
    fn inverted(&self) -> Self {
        DominoSet {
            low: !self.low,
            high: !self.high,
        }
    }
    fn as_vector(&self) -> Vec<(u8, u8)> {
        // Convert to vector.
        let mut dominoes: Vec<(u8, u8)> = vec![];
        let mut bits = self.low;
        for max in 0..8 {
            for min in 0..(max + 1) {
                if bits & 1 != 0 {
                    dominoes.push((min, max));
                }
                bits >>= 1;
            }
            bits >>= 15 - max;
        }
        bits = self.high;
        for max in 8..(PIP_MAX_U8 + 1) {
            for min in 0..(max + 1) {
                if bits & 1 != 0 {
                    dominoes.push((min, max));
                }
                bits >>= 1;
            }
            bits >>= 15 - max;
        }
        dominoes
    }
}

#[derive(Default, Clone)]
struct DoubleDomino {
    pips: u8,
    count: u8,
    first: bool,
}

impl DoubleDomino {
    fn max_count(&self) -> u8 {
        if self.first {
            4
        } else {
            3
        }
    }
}

#[derive(Clone)]
struct GameState {
    /// `Some(DoubleDomino)` if a double (other than the first) is in play. Set to None when count reaches 4 (first) or 3 (not first).
    double: Option<DoubleDomino>,
    /// Count of endpoints (i.e., where dominoes can be played) with [index] pips
    endpoints: [u8; PIP_MAX_USIZE + 1],
    /// Dominoes in play
    played: DominoSet,
    /// Dominoes in the user's hand
    hand: DominoSet,
}

impl GameState {
    /// Play a domino. Places the `min` end on the endpoint if `min_matches` is `true`.
    /// Did nothing if `Err` is returned.
    fn play(&mut self, min: u8, max: u8, min_matches: bool) -> Result<(), String> {
        #[cfg(debug_assertions)]
        if self.played.has(min, max) {
            return Err(format!("Domino {min} {max} has already been played."));
        }

        // Rotate domino.
        let (previous_endpoint, next_endpoint) = if min_matches {
            (min as usize, max as usize)
        } else {
            (max as usize, min as usize)
        };

        // Play on a double domino.
        if let Some(double_domino) = &mut self.double {
            debug_assert!(double_domino.count < 4);
            if usize::from(double_domino.pips) != previous_endpoint {
                return Err(format!(
                    "Invalid Move: play a {} on the double domino.",
                    double_domino.pips
                ));
            }
            self.endpoints[next_endpoint] += 1;
            double_domino.count += 1;
            if double_domino.count >= double_domino.max_count() {
                self.double = None;
            }
            self.played.add(min, max);
            return Ok(());
        }

        // Play a double domino.
        if min == max {
            if self.endpoints[previous_endpoint] == 0 {
                return Err("Invalid Move: play on an end.".to_owned());
            }
            self.endpoints[previous_endpoint] -= 1;
            self.double = Some(DoubleDomino {
                pips: min,
                count: 0,
                first: false,
            });
            self.played.add(min, max);
            return Ok(());
        }

        // Play a non-double domino on another non-double domino.
        if self.endpoints[previous_endpoint] == 0 {
            return Err("Invalid Move: play on an end.".to_owned());
        }
        self.endpoints[previous_endpoint] -= 1;
        self.endpoints[next_endpoint] += 1;
        self.played.add(min, max);
        return Ok(());
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            double: Some(DoubleDomino {
                pips: 0,
                count: 0,
                first: true,
            }),
            endpoints: [0; PIP_MAX_USIZE + 1],
            played: DominoSet { low: 0, high: 0 },
            hand: DominoSet { low: 0, high: 0 },
        }
    }
}

fn domino_image<'a>(min: u8, max: u8) -> Image<'a> {
    Image::new(egui::include_image!("../generateDominoes/set.png")).uv(Rect::from_min_size(
        Pos2::new(min as f32 / 13.0, max as f32 / 13.0),
        Vec2::new(1.0 / 13.0, 1.0 / 13.0),
    ))
}
fn pips_image<'a>(pips: u8) -> Image<'a> {
    Image::new(egui::include_image!("../generateDominoes/pips.png")).uv(Rect::from_min_size(
        Pos2::new(pips as f32 / 13.0, 0.0),
        Vec2::new(1.0 / 13.0, 1.0 / 13.0),
    ))
}

struct MainWindow {
    game_state: GameState,
    stack: Vec<GameState>,
    text_edit: String,
    info: String,
    painter: egui::Painter,
}

impl MainWindow {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let self_ = Self {
            game_state: GameState::default(),
            stack: Vec::new(),
            text_edit: String::default(),
            info: String::default(),
            painter: cc.egui_ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("overlay"),
            )),
        };
        self_
    }

    fn push_stack(&mut self) {
        self.stack.push(self.game_state.clone());
    }

    fn draw_played_dominoes(&mut self, ui: &mut egui::Ui) {
        ui.heading("Played Dominoes");
        let re = ui
            .image(egui::include_image!("../generateDominoes/set.png"))
            .interact(egui::Sense::click());
        if re.clicked() {
            if let Some(Pos2 { x, y }) = re.interact_pointer_pos() {
                let min =
                    ((x - re.rect.left()) / re.rect.width() * 13.0).clamp(0.0, PIP_MAX_FLOAT) as u8;
                let max =
                    ((y - re.rect.top()) / re.rect.height() * 13.0).clamp(0.0, PIP_MAX_FLOAT) as u8;
                if min <= max && max <= PIP_MAX_U8 {
                    self.push_stack();
                    self.game_state.played.toggle(min, max);
                }
            }
        }

        for (min, max) in self.game_state.played.inverted().as_vector() {
            /*self.painter.rect_stroke(
                Rect::from_min_size(
                    re.rect.lerp_inside(Vec2::new(
                        (min as f32 + 0.1) / 13.0,
                        (max as f32 + 0.1) / 13.0,
                    )),
                    Vec2::new(re.rect.width() / 13.0 * 0.8, re.rect.height() / 13.0 * 0.8),
                ),
                0.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 255, 0)),
            );*/
            self.painter.rect_filled(
                Rect::from_min_size(
                    re.rect
                        .lerp_inside(Vec2::new(min as f32 / 13.0, max as f32 / 13.0)),
                    Vec2::new(re.rect.width() / 13.0, re.rect.height() / 13.0),
                ),
                0.0,
                egui::Color32::from_rgba_premultiplied(0, 0, 0, 128),
            );
        }
    }

    fn draw_double(&mut self, ui: &mut egui::Ui) {
        let mut frame = Frame::group(ui.style()).begin(ui);
        frame.content_ui.heading("Double Domino");
        let mut active = self.game_state.double.is_some();
        if frame.content_ui.checkbox(&mut active, "Active").changed() {
            self.push_stack();
            self.game_state.double = if active {
                Some(DoubleDomino::default())
            } else {
                None
            };
        }
        frame.content_ui.horizontal(|ui| {
            let max_count = self
                .game_state
                .double
                .as_ref()
                .and_then(|d| Some(d.max_count()));
            let state_copy = self.game_state.clone();
            if let Some(DoubleDomino { pips, count, first }) = &mut self.game_state.double {
                let max_count = max_count.unwrap();
                ui.horizontal(|ui| {
                    let label = if *first {
                        "First Domino"
                    } else {
                        "Chickenfoot"
                    };
                    ui.toggle_value(first, label);
                    ui.label("Pips:");
                    ui.add(DragValue::new(pips).speed(0.05).range(0..=PIP_MAX_U8));
                    ui.label(format!("Played / {max_count}:"));
                    ui.add(DragValue::new(count).speed(0.05).range(0..=(max_count - 1)));
                });
                if *first && ui.button("Start Game").clicked() {
                    self.stack.push(state_copy);
                    self.game_state.played.clear();
                    self.game_state.played.add(*pips, *pips);
                }
            }
        });
        if let Some(DoubleDomino { pips, .. }) = self.game_state.double {
            frame
                .content_ui
                .add_sized([60.0, 120.0], domino_image(pips, pips));
        }
        frame.end(ui);
    }

    fn draw_endpoints(&mut self, ui: &mut egui::Ui) {
        let mut frame = Frame::group(ui.style()).begin(ui);
        frame.content_ui.heading("Endpoints");
        frame.content_ui.horizontal(|ui| {
            ui.label("Set pips and count:");
            let re = ui.text_edit_singleline(&mut self.text_edit);
            if re.lost_focus() && re.ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                let parts: Vec<&str> = self.text_edit.split(" ").collect();
                if parts.len() == 2 {
                    match (parts[0].parse::<u8>(), parts[1].parse::<u8>()) {
                        (Ok(pips), Ok(count))if pips <= PIP_MAX_U8 && count <= PIP_MAX_U8 => {
                            self.push_stack();
                            self.game_state.endpoints[pips as usize] = count;
                            self.text_edit.clear();
                            self.info.clear();
                        }
                        _ => {
                            self.info.replace_with(
                                "Set endpoints command must be entered as two numbers 0-12 seperated by a space.",
                            );
                        }
                    }
                } else {
                    self.info.replace_with(
                        "Set endpoints command must be entered as two numbers 0-12 seperated by a space.",
                    );
                }
                re.request_focus();
            }
        });
        frame.content_ui.horizontal_wrapped(|ui| {
            let mut pips: u8 = 0;
            for count in self.game_state.endpoints {
                ui.vertical(|ui| {
                    for _ in 0..count {
                        ui.add_sized([60.0, 60.0], pips_image(pips));
                    }
                });
                pips += 1;
            }
        });
        frame.end(ui);
    }

    fn draw_hand(&mut self, ui: &mut egui::Ui) {
        let mut frame = Frame::group(ui.style()).begin(ui);
        frame.content_ui.heading("Player's Hand");
        frame
            .content_ui
            .label("Left click end to play. Right click to remove.");

        // List of dominoes in the player's hand:
        frame.content_ui.horizontal_wrapped(|ui| {
            for (min, max) in self.game_state.hand.as_vector() {
                let domino = ui
                    .add_sized([60.0, 120.0], domino_image(min, max))
                    .interact(egui::Sense::click());
                if domino.clicked() {
                    if let Some(Pos2 { x: _, y }) = domino.interact_pointer_pos() {
                        // Top or bottom of domino was clicked. Rotate and attempt to play.
                        let state_copy = self.game_state.clone();
                        if let Err(text) =
                            self.game_state.play(min, max, y < domino.rect.center().y)
                        {
                            // Report invalid play.
                            self.info = text;
                        } else {
                            // Play successful: remove domino from player's hand.
                            self.stack.push(state_copy);
                            self.game_state.hand.remove(min, max);
                        }
                    }
                } else if domino.secondary_clicked() {
                    // Domino was right clicked: remove from player's hand.
                    self.push_stack();
                    self.game_state.hand.remove(min, max);
                }
            }
        });

        // Add dominoes to player's hand.
        frame.content_ui.horizontal(|ui| {
            ui.label("Draw:");
            let re = ui.text_edit_singleline(&mut self.text_edit);
            if re.lost_focus() && re.ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                let parts: Vec<&str> = self.text_edit.split(" ").collect();
                if parts.len() == 2 {
                    match (parts[0].parse::<u8>(), parts[1].parse::<u8>()) {
                        (Ok(a), Ok(b)) if a <= PIP_MAX_U8 && b <= PIP_MAX_U8 => {
                            self.push_stack();
                            self.game_state.hand.add(a.min(b), a.max(b));
                            self.text_edit.clear();
                            self.info.clear();
                        }
                        _ => {
                            self.info.replace_with(
                                "Domino must be entered as two numbers 0-12 seperated by a space.",
                            );
                        }
                    }
                } else {
                    self.info.replace_with(
                        "Domino must be entered as two numbers 0-12 seperated by a space.",
                    );
                }
                re.request_focus();
            }
        });
        frame.end(ui);
    }

    fn other_players(&mut self, ui: &mut egui::Ui) {
        let mut frame = Frame::group(ui.style()).begin(ui);
        frame.content_ui.heading("Other Players");
        frame.content_ui.label("Enter matching end first.");
        frame.content_ui.horizontal(|ui| {
            ui.label("Play:");
            let re = ui.text_edit_singleline(&mut self.text_edit);
            if re.lost_focus() && re.ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                let parts: Vec<&str> = self.text_edit.split(" ").collect();
                if parts.len() == 2 {
                    match (parts[0].parse::<u8>(), parts[1].parse::<u8>()) {
                        (Ok(a), Ok(b)) if a <= PIP_MAX_U8 && b <= PIP_MAX_U8 => {
                            let state_copy = self.game_state.clone();
                            if let Err(text) =
                                self.game_state.play(a.min(b), a.max(b), a.min(b) == a)
                            {
                                self.info = text;
                            } else {
                                self.stack.push(state_copy);
                                self.text_edit.clear();
                                self.info.clear();
                            }
                        }
                        _ => {
                            self.info.replace_with(
                                "Domino must be entered as two numbers 0-12 seperated by a space.",
                            );
                        }
                    }
                } else {
                    self.info.replace_with(
                        "Domino must be entered as two numbers 0-12 seperated by a space.",
                    );
                }
                re.request_focus();
            }
        });
        frame.end(ui);
    }
}

impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.input_mut(|i| {
            if i.consume_shortcut(&UNDO_SHORTCUT) {
                if let Some(new_state) = self.stack.pop() {
                    self.game_state = new_state;
                    self.info = format!("{} undo frames remain.", self.stack.len());
                } else {
                    self.info.replace_with("Nothing to undo.")
                }
            }
        });

        egui::SidePanel::left("played_dominoes")
            .exact_width(ctx.screen_rect().width().min(ctx.screen_rect().height()) / 2.0)
            .show(ctx, |ui| {
                // Draw played dominoes.
                self.draw_played_dominoes(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(&self.info);
            ScrollArea::vertical().show(ui, |ui| {
                // Start or double/chickenfoot
                self.draw_double(ui);

                // Player's hand
                self.draw_hand(ui);

                // Other players' play
                self.other_players(ui);

                // Endpoints
                self.draw_endpoints(ui);
            });
        });
    }
}

fn main() {
    let mut native_options = eframe::NativeOptions::default();
    native_options.viewport.title = Some("Chickenfoot".to_owned());
    native_options.viewport.min_inner_size = Some(Vec2::new(850.0, 500.0));
    let _ = eframe::run_native(
        "bwestley-chickenfoot",
        native_options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(MainWindow::new(cc)))
        }),
    );
}

use std::{collections::HashMap, fs};
use eframe::egui::{self, Color32, Id, Sense, TextureOptions, Vec2};
use egui_modal::Modal;

mod rom;
mod viewport;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([864.0, 648.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Hibachi",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<HibachiApp>::default())
        }),
    )
}

struct HibachiApp {
    rom_path: String,
    rom: Option<Box<[u8]>>,
    lasterr: String,
    errbox: Vec<Modal>, // terrible hack

    levels: [Vec<[usize; 2]>; 8],
    areas: [Vec<Area>; 4],

    world_editor_selected_world: usize,

    game_graphics: HashMap<MarioGraphics, ViewportSprite>,
    game_palette: HashMap<MarioColorPalette, HashMap<MarioColor, [egui::Color32; 3]>>,
    game_skycolor: HashMap<MarioColorPalette, egui::Color32>,
    game_font: HashMap<char, ViewportSprite>,

    current_editing_area: Option<[usize; 2]>
}

impl Default for HibachiApp {
    fn default() -> Self {
        Self {
            rom_path: String::from(""),
            rom: None,
            lasterr: String::from("Task failed successfully!"),
            errbox: vec!(),
            levels: [vec!(), vec!(), vec!(), vec!(), vec!(), vec!(), vec!(), vec!()],
            areas: [vec!(), vec!(), vec!(), vec!()],
            world_editor_selected_world: 0,
            game_graphics: HashMap::new(),
            game_palette: HashMap::new(),
            game_skycolor: HashMap::new(),
            game_font: HashMap::new(),
            current_editing_area: None
        }
    }
}

impl eframe::App for HibachiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.errbox.push(Modal::new(ctx, "errbox"));
        self.errbox[0].show(|ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Error");
                ui.label(&self.lasterr);
                if ui.button("Close").clicked() {
                    self.errbox[0].close();
                }
            });
        });

        let world_header_editor = Modal::new(ctx, "wrldedit");
        world_header_editor.show(|ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Edit World Header Info");

                egui::ComboBox::from_label("World").selected_text(format!("World {}", self.world_editor_selected_world + 1)).show_ui(ui, |ui| {
                    for n in 0..=7 {
                        ui.selectable_value(&mut self.world_editor_selected_world, n, format!("World {}", n + 1));
                    }
                });

                let mut ln = 1;

                for n in 0..self.levels[self.world_editor_selected_world].len() {
                    ui.horizontal(|ui| {
                        if self.areas[self.levels[self.world_editor_selected_world][n][0]][self.levels[self.world_editor_selected_world][n][1]].mario_height == AreaMarioStartHeightSetting::Autowalk {
                            ui.label("Cutscene");
                        }else{
                            ui.label(format!("Level #{}", ln));
                            ln += 1;
                        }

                        egui::ComboBox::new(6969 + n, "Area Type").selected_text(
                            match self.levels[self.world_editor_selected_world][n][0] {
                                0 => "Water",
                                1 => "Ground",
                                2 => "Cave",
                                3 => "Castle",
                                _ => "Invalid"
                            }
                        ).show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.levels[self.world_editor_selected_world][n][0], 1, "Ground");
                            ui.selectable_value(&mut self.levels[self.world_editor_selected_world][n][0], 2, "Cave");
                            ui.selectable_value(&mut self.levels[self.world_editor_selected_world][n][0], 0, "Water");
                            ui.selectable_value(&mut self.levels[self.world_editor_selected_world][n][0], 3, "Castle");
                        });

                        if self.levels[self.world_editor_selected_world][n][1] >= self.areas[self.levels[self.world_editor_selected_world][n][0]].len() {
                            self.levels[self.world_editor_selected_world][n][1] = 0;
                        }

                        egui::ComboBox::new(420420 + n, "Area ID").selected_text(format!("{:#04x}", self.levels[self.world_editor_selected_world][n][1])).show_ui(ui, |ui| {
                            for k in 0..self.areas[self.levels[self.world_editor_selected_world][n][0]].len() {
                                ui.selectable_value(&mut self.levels[self.world_editor_selected_world][n][1], k, format!("{:#04x}", k));
                            }
                        });
                    });
                }

                if ui.button("Close").clicked() {
                    world_header_editor.close();
                }
            });
        });

        let area_list_editor = Modal::new(ctx, "arealist");
        area_list_editor.show(|ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Areas");
            });

            egui::ScrollArea::vertical().max_height(640.0).show(ui, |ui| {
                for t in 0..4 {
                    let rt = match t {
                        0 => 1,
                        1 => 2,
                        2 => 0,
                        n => n
                    };

                    ui.heading(match t {
                        0 => "Ground Areas",
                        1 => "Cave Areas",
                        2 => "Water Areas",
                        3 => "Castle Areas",
                        _ => "uuuuuuhhhhhhhhhhhhhhhhhh???",
                    });
                    
                    if self.areas[rt].len() == 0 {
                        ui.horizontal(|ui| {
                            ui.label("No areas!");
                            if ui.button("Add Area").clicked() {
                                self.areas[rt].push(Area::default_for_type(rt));
                            }
                        });
                    }else{
                        for n in 0..self.areas[rt].len() {
                            ui.horizontal(|ui| {
                                ui.label(format!("{:#04x}", n));

                                if ui.add_enabled(self.areas[rt].len() < 0x20, egui::Button::new("Add Above")).clicked() {
                                    self.areas[rt].insert(n, Area::default_for_type(rt));
                                }

                                if ui.add_enabled(self.areas[rt].len() < 0x20, egui::Button::new("Add Below")).clicked() {
                                    self.areas[rt].insert(n + 1, Area::default_for_type(rt));
                                }

                                if ui.add(egui::Button::new("Remove")).clicked() {
                                    self.areas[rt].remove(n);
                                }
                            });
                        }
                    }
                }
            });

            ui.vertical_centered(|ui| {
                if ui.button("Close").clicked() {
                    area_list_editor.close();
                }
            });
        });

        let area_edit_picker = Modal::new(ctx, "areapicker");
        area_edit_picker.show(|ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Areas");
            });

            egui::ScrollArea::vertical().max_height(640.0).show(ui, |ui| {
                for t in 0..4 {
                    let rt = match t {
                        0 => 1,
                        1 => 2,
                        2 => 0,
                        n => n
                    };

                    ui.heading(match t {
                        0 => "Ground Areas",
                        1 => "Cave Areas",
                        2 => "Water Areas",
                        3 => "Castle Areas",
                        _ => "uuuuuuhhhhhhhhhhhhhhhhhh???",
                    });
                    
                    if self.areas[rt].len() == 0 {
                        ui.horizontal(|ui| {
                            ui.label("No areas!");
                        });
                    }else{
                        for n in 0..self.areas[rt].len() {
                            ui.horizontal(|ui| {
                                ui.label(format!("{:#04x}", n));

                                if ui.add(egui::Button::new("Edit")).clicked() {
                                    self.current_editing_area = Some([rt, n]);
                                    area_edit_picker.close();
                                }
                            });
                        }
                    }
                }
            });

            ui.vertical_centered(|ui| {
                if ui.button("Close").clicked() {
                    self.current_editing_area = None;
                    area_edit_picker.close();
                }
            });
        });

        egui::TopBottomPanel::top(Id::new("topmenu")).show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.add(egui::Button::image(egui::include_image!("../pics/open.png"))).clicked() {
                    if let Some(path) = tinyfiledialogs::open_file_dialog("Open ROM", "smb1.nes", None) {
                        match (fs::read(&path)) {
                            Ok(v) => {
                                self.rom = Some(v.into_boxed_slice());

                                if self.validate_rom() {
                                    self.rom_path = path.clone();

                                    self.reload_graphics(ctx);

                                    self.current_editing_area = None;

                                    if self.is_patched_organisation() {
                                        self.reload_levels();
                                    }else{
                                        self.reload_levels_unpatched();
                                        self.install_organisation_patch();
                                    }
                                }else{
                                    self.rom = None;
                                }
                            },
                            Err(e) => {
                                self.err("Failed to read the ROM file.");
                                self.rom = None;
                            }
                        }
                    }
                }

                if ui.add_enabled(self.is_rom_open(), egui::Button::image(egui::include_image!("../pics/save.png"))).clicked() {
                    self.err("unimplemented.");

                    /*
                    self.rewrite_levels();

                    match fs::write(&self.rom_path, self.rom.clone().unwrap()) {
                        Ok(_) => {},
                        Err(_) => {
                            self.err("Failed to save the file.");
                        }
                    }
                    */
                }

                if ui.add_enabled(self.is_rom_open(), egui::Button::image(egui::include_image!("../pics/saveas.png"))).clicked() {
                    self.err("unimplemented.");
                }
            });
        });

        egui::SidePanel::left("toolbox").min_width(200.0).max_width(500.0).resizable(true).show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                if self.is_rom_open() {
                    if ui.button("Edit World Headers").clicked() {
                        world_header_editor.open();
                    }

                    if ui.button("Add or Remove Areas").clicked() {
                        area_list_editor.open();
                    }

                    ui.separator();

                    if ui.button("Modify Areas").clicked() {
                        area_edit_picker.open();
                    }
                }else{
                    ui.heading("No Rom Loaded");
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::canvas(ui.style()).inner_margin(0.0).outer_margin(0.0).show(ui, |ui| {
                if let Some(a) = self.current_editing_area {
                    self.area_editor(ui, a);
                }
            });
        });
    }
}

impl HibachiApp {
    fn err(&mut self, errstring: &str) {
        self.lasterr = String::from(errstring);
        self.errbox[0].open();
    }

    fn area_editor(&mut self, ui: &mut egui::Ui, arid: [usize; 2]) {
        let apal = match arid[0] {
            0 => MarioColorPalette::Water,
            1 => MarioColorPalette::Ground,
            2 => MarioColorPalette::Cave,
            _ => MarioColorPalette::Castle
        };

        egui::ScrollArea::horizontal().hscroll(true).show(ui, |ui| {

            let (resp, canvas) = ui.allocate_painter(Vec2::new(32.0*16.0*32.0, 14.0*32.0), Sense::click_and_drag());
            canvas.rect_filled(
                resp.rect,
                egui::Rounding::ZERO,
                self.game_skycolor[&apal],
            );

            self.paint_sprite(&canvas, resp.rect.min.to_vec2(), 80, 11 * 32, MarioGraphics::MarioStart, apal);
            self.paint_tag(&canvas, resp.rect.min.to_vec2(), 112, 11 * 32, "Entrance to", format!("Area {}", arid[1]).as_str(), apal);
            
            for xn in 0..16*32 {
                self.paint_sprite(&canvas, resp.rect.min.to_vec2(), xn * 32, 12 * 32, MarioGraphics::TerrainRocky, apal);
                self.paint_sprite(&canvas, resp.rect.min.to_vec2(), xn * 32, 13 * 32, MarioGraphics::TerrainRocky, apal);
            }

            println!("----");

            for s in &self.areas[arid[0]][arid[1]].stuff {
                print!("{}, ", s.x);
                println!("{}", s.y);
                if self.game_graphics.contains_key(&MarioGraphics::SpriteObject(s.variety.clone())) {
                    self.paint_sprite(&canvas, resp.rect.min.to_vec2(), s.x as isize * 32, s.y as isize * 32, MarioGraphics::SpriteObject(s.variety.clone()), apal);
                }else{
                    self.paint_sprite(&canvas, resp.rect.min.to_vec2(), s.x as isize * 32, s.y as isize * 32, MarioGraphics::SpriteObject(SpriteObjectType::StupidRedKoopa), apal);

                }
            }

            ui.allocate_exact_size(resp.rect.size(), Sense::focusable_noninteractive());
        });
    }
}

#[inline(always)]
fn rom_addr(n: usize) -> usize {
    n - 32752
}

fn nes_color(n: u8) -> egui::Color32 {
    match n % 0x40 {
        0x00 => egui::Color32::from_rgb(0x62, 0x62, 0x62),
        0x01 => egui::Color32::from_rgb(0x00, 0x2c, 0x7c),
        0x02 => egui::Color32::from_rgb(0x11, 0x15, 0x9c),
        0x03 => egui::Color32::from_rgb(0x36, 0x03, 0x9c),
        0x04 => egui::Color32::from_rgb(0x55, 0x00, 0x7c),
        0x05 => egui::Color32::from_rgb(0x67, 0x00, 0x44),
        0x06 => egui::Color32::from_rgb(0x67, 0x07, 0x03),
        0x07 => egui::Color32::from_rgb(0x55, 0x1c, 0x00),
        0x08 => egui::Color32::from_rgb(0x36, 0x32, 0x00),
        0x09 => egui::Color32::from_rgb(0x11, 0x44, 0x00),
        0x0A => egui::Color32::from_rgb(0x00, 0x4e, 0x00),
        0x0B => egui::Color32::from_rgb(0x00, 0x4c, 0x03),
        0x0C => egui::Color32::from_rgb(0x00, 0x40, 0x44),
        0x0D => egui::Color32::from_rgb(0x00, 0x00, 0x00),
        0x0E => egui::Color32::from_rgb(0x00, 0x00, 0x00),
        0x0F => egui::Color32::from_rgb(0x00, 0x00, 0x00),
        0x10 => egui::Color32::from_rgb(0xab, 0xab, 0xab),
        0x11 => egui::Color32::from_rgb(0x12, 0x60, 0xce),
        0x12 => egui::Color32::from_rgb(0x3d, 0x42, 0xfa),
        0x13 => egui::Color32::from_rgb(0x6e, 0x29, 0xfa),
        0x14 => egui::Color32::from_rgb(0x99, 0x1c, 0xce),
        0x15 => egui::Color32::from_rgb(0xb1, 0x1e, 0x81),
        0x16 => egui::Color32::from_rgb(0xb1, 0x2f, 0x29),
        0x17 => egui::Color32::from_rgb(0x99, 0x4a, 0x00),
        0x18 => egui::Color32::from_rgb(0x6e, 0x69, 0x00),
        0x19 => egui::Color32::from_rgb(0x3d, 0x82, 0x00),
        0x1A => egui::Color32::from_rgb(0x12, 0x8f, 0x00),
        0x1B => egui::Color32::from_rgb(0x00, 0x8d, 0x29),
        0x1C => egui::Color32::from_rgb(0x00, 0x7c, 0x81),
        0x1D => egui::Color32::from_rgb(0x00, 0x00, 0x00),
        0x1E => egui::Color32::from_rgb(0x00, 0x00, 0x00),
        0x1F => egui::Color32::from_rgb(0x00, 0x00, 0x00),
        0x20 => egui::Color32::from_rgb(0xff, 0xff, 0xff),
        0x21 => egui::Color32::from_rgb(0x60, 0xb2, 0xff),
        0x22 => egui::Color32::from_rgb(0x8d, 0x92, 0xff),
        0x23 => egui::Color32::from_rgb(0xc0, 0x78, 0xff),
        0x24 => egui::Color32::from_rgb(0xec, 0x6a, 0xff),
        0x25 => egui::Color32::from_rgb(0xff, 0x6d, 0xd4),
        0x26 => egui::Color32::from_rgb(0xff, 0x7f, 0x79),
        0x27 => egui::Color32::from_rgb(0xec, 0x9b, 0x2a),
        0x28 => egui::Color32::from_rgb(0xc0, 0xba, 0x00),
        0x29 => egui::Color32::from_rgb(0x8d, 0xd4, 0x00),
        0x2A => egui::Color32::from_rgb(0x60, 0xe2, 0x2a),
        0x2B => egui::Color32::from_rgb(0x47, 0xe0, 0x79),
        0x2C => egui::Color32::from_rgb(0x47, 0xce, 0xd4),
        0x2D => egui::Color32::from_rgb(0x4e, 0x4e, 0x4e),
        0x2E => egui::Color32::from_rgb(0x00, 0x00, 0x00),
        0x2F => egui::Color32::from_rgb(0x00, 0x00, 0x00),
        0x30 => egui::Color32::from_rgb(0xff, 0xff, 0xff),
        0x31 => egui::Color32::from_rgb(0xbf, 0xe0, 0xff),
        0x32 => egui::Color32::from_rgb(0xd1, 0xd3, 0xff),
        0x33 => egui::Color32::from_rgb(0xe6, 0xc9, 0xff),
        0x34 => egui::Color32::from_rgb(0xf7, 0xc3, 0xff),
        0x35 => egui::Color32::from_rgb(0xff, 0xc4, 0xee),
        0x36 => egui::Color32::from_rgb(0xff, 0xcb, 0xc9),
        0x37 => egui::Color32::from_rgb(0xf7, 0xd7, 0xa9),
        0x38 => egui::Color32::from_rgb(0xe6, 0xe3, 0x97),
        0x39 => egui::Color32::from_rgb(0xd1, 0xee, 0x97),
        0x3A => egui::Color32::from_rgb(0xbf, 0xf3, 0xa9),
        0x3B => egui::Color32::from_rgb(0xb5, 0xf2, 0xc9),
        0x3C => egui::Color32::from_rgb(0xb5, 0xeb, 0xee),
        0x3D => egui::Color32::from_rgb(0xb8, 0xb8, 0xb8),
        0x3E => egui::Color32::from_rgb(0x00, 0x00, 0x00),
        0x3F => egui::Color32::from_rgb(0x00, 0x00, 0x00),
        _ => egui::Color32::from_rgb(0xFF, 0x00, 0x00)
    }
}

struct Area {
    timer: AreaTimerSetting,
    mario_height: AreaMarioStartHeightSetting,
    bg: AreaBackdropSetting,
    special: AreaSpecialSetting,
    bg2: AreaScenerySetting,
    initial_tilepat: u8,
    terrain: Vec<AreaTerrainObject>,
    stuff: Vec<AreaSpriteObject>
}

impl Area {
    fn default_for_type(t: usize) -> Area {
        match t {
            0 => Area {
                timer: AreaTimerSetting::T300,
                mario_height: AreaMarioStartHeightSetting::VeryHigh,
                bg: AreaBackdropSetting::Underwater,
                special: AreaSpecialSetting::Ordinary,
                bg2: AreaScenerySetting::Blank,
                initial_tilepat: 1,
                terrain: vec!(),
                stuff: vec!()
            },
            1 => Area {
                timer: AreaTimerSetting::T300,
                mario_height: AreaMarioStartHeightSetting::Ground,
                bg: AreaBackdropSetting::Overworld,
                special: AreaSpecialSetting::Mushroom,
                bg2: AreaScenerySetting::Hills,
                initial_tilepat: 1,
                terrain: vec!(),
                stuff: vec!()
            },
            2 => Area {
                timer: AreaTimerSetting::T300,
                mario_height: AreaMarioStartHeightSetting::High,
                bg: AreaBackdropSetting::Overworld,
                special: AreaSpecialSetting::Cannon,
                bg2: AreaScenerySetting::Blank,
                initial_tilepat: 1,
                terrain: vec!(),
                stuff: vec!()
            },
            3 => Area {
                timer: AreaTimerSetting::T300,
                mario_height: AreaMarioStartHeightSetting::Midair,
                bg: AreaBackdropSetting::Overworld,
                special: AreaSpecialSetting::Ordinary,
                bg2: AreaScenerySetting::Blank,
                initial_tilepat: 1,
                terrain: vec!(),
                stuff: vec!()
            },
            _ => Area {
                timer: AreaTimerSetting::Sublevel,
                mario_height: AreaMarioStartHeightSetting::Ground,
                bg: AreaBackdropSetting::Grayscale,
                special: AreaSpecialSetting::Ordinary,
                bg2: AreaScenerySetting::Blank,
                initial_tilepat: 1,
                terrain: vec!(),
                stuff: vec!()
            },
        }
    }
}

#[derive(PartialEq)]
enum AreaTimerSetting {
    T200,    // 11
    T300,    // 10
    T400,    // 01
    Sublevel // 00
}

#[derive(PartialEq)]
enum AreaMarioStartHeightSetting {
    VeryHigh, // used for water levels
    High, // cave
    Ground,
    Midair, // castle
    /*
    Unused,
    Unused2,
    Unused3Autowalk,
    */
    Autowalk
}

#[derive(PartialEq)]
enum AreaBackdropSetting {
    Overworld,
    Underwater,
    GreatWall,
    Seaside,
    Nighttime,
    SnowDay,
    SnowNight,
    Grayscale
}

#[derive(PartialEq)]
enum AreaSpecialSetting {
    Ordinary,
    Mushroom,
    Cannon,
    Sky
}

#[derive(PartialEq)]
enum AreaScenerySetting {
    Blank,
    Clouds,
    Hills,
    Fences
}

struct AreaTerrainObject {
    x: usize,
    y: u8,  // 0 <= y <= 11 (12..=15 are obj types)
    variety: TerrainObjectType
}

#[derive(PartialEq)]
enum TerrainObjectType {
    RawHex(u8), // fallback / hacky stuff

    SpecialPlat(u8), // 001 ...
    BrickRow(u8),
    BlockRow(u8),
    CoinRow(u8),
    BrickColumn(u8),
    BlockColumn(u8), // ... 110

    Pipe(bool, u8), // 111

    PowerupQBlock,    // 000:0000 ...
    CoinQBlock,
    CoinHiddenBlock,
    OneupHiddenBlock,
    PowerupBrick,
    VineBrick,
    StarBrick,
    LotsOfCoinsBrick,
    OneupBrick,
    HitBlock,
    SidePipeExit,
    Trampoline,
    LPipe,
    Flagpole,   // ... 000:1101

    // height 12 types, from 000 to 111 (sized)

    Hole(u8),
    Pulley(u8),
    Y7Bridge(u8),
    Y8Bridge(u8),
    Y10Bridge(u8),
    WaterHole(u8),
    Y3QBlocks(u8),
    Y7QBlocks(u8),

    // height 13 types , top bit unset
    
    // ScreenSkipCommand(u8) will be handled internally :)

    // height 13 types , top bit set

    // LPipe (duplicate?)
    // Flagpole (duplicate?)

    BridgeAx, // 000010...
    BridgeChain,
    BowserBridge,
    WarpZoneCommand,
    NoScrollCommand,
    NoScroll2Command,
    CheepCheepSpawnCommand,
    BulletSpawnCommand,
    NoSpawnCommand,
    EndlessHallwayCommand,

    // height 14 types , top bit unset

    SceneryAndFloorPatternSwitch(u8, u8), // scenery , floor pat

    // height 14 types , top bit set

    BackdropSwitchCommand(u8),

    // height 15 types , 000 to 101 sized

    LeftPulleyRope(u8),
    RightPulleyRope(u8),
    Fortress(u8),
    BlockStaircase(u8),
    ExitLPipe(u8),
    UnusedVine(u8),
}

struct AreaSpriteObject {
    variety: SpriteObjectType,
    lategame: bool, // originally labeled this variable 'hard', quickly regretted that
    x: usize,
    y: u8 // 0 <= y <= 13
}

#[derive(PartialEq, Eq, Hash, Clone)]
enum SpriteObjectType {
    RawHex(u8), // backup or hacky

    GreenKoopa,
    StupidRedKoopa,
    Buzzy,
    RedKoopa,
    ReallyStupidGreenKoopa,
    HammerBro,
    Goomba,
    Blooper,
    BulletBill,
    YellowParatroopa,
    SlowCheep,
    FastCheep,
    Podoboo,
    Pirahna, // is this used?
    JumpyGreenParatroopa,
    VerticalRedParatroopa,
    HorizontalGreenParatroopa,
    Lakitu,
    SpinyDontUse, // dont use! (do not use)

    // 0x13 undefined

    JumpingCheepsGenerator,
    BowserFireGenerator,
    FireworkGenerator, // again is this used???
    BulletBillGenerator, // all my sources are saying these can also do cheeps . wtf does that mean

    // 0x18-0x1A undefined

    ClockwiseFireBar,
    FastClockwiseFireBar,
    CCWFireBar,
    FastCCWFireBar,
    BigClockwiseFireBar,

    // 0x20-0x23 undefined

    BalanceLift,
    UpAndDownLift,
    UpLift,
    DownLift,
    AcrossLift,
    FallingLift,
    RightLift,
    ShortUpLift,
    ShortDownLift,
    Bowser,

    // 0x2E - 0x33 undefined

    WarpZone, // this is already a tile !!! i hate these devs. i guess ill find out how this is used once i start loading levels
    Toad, // peach in w8

    // 0x36 undefined

    Y10_2Goombas,
    Y10_3Goombas,
    Y6_2Goombas,
    Y6_3Goombas,
    Y10_2Koopas,
    Y10_3Koopas,
    Y6_2Koopas,
    Y6_3Koopas,

    // 0x3F on undefined

    // y14

    WarpInfo(u8, u8, u8) // for enterable pipes: exit lev id, enterance world num, return screen num

    // y15

    // ScreenSkipCommand(u8),
}

#[derive(PartialEq, Eq, Hash, Clone)]
enum MarioGraphics {
    MarioStart,

    TerrainRocky,
    TerrainSearock,
    TerrainCastleBrick,

    SpriteObject(SpriteObjectType),
}

#[derive(PartialEq, Eq, Hash, Clone)]
enum MarioColor {
    Vegetation,
    Brick,
    Cloud,
    Shiny,

    Mario,
    GreenEnemy,
    RedEnemy,
    BlackEnemy
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum MarioColorPalette {
    Ground,
    Cave,
    Water,
    Castle,

    //variants:
    Night,
    SnowDay,
    SnowNight,
    // gray uses castle
    Mushroom,
    Bowser
}

struct ViewportSprite {
    slices: [egui::TextureHandle; 3],
    offset: [isize; 2],
    size: [usize; 2],
    palette: MarioColor
}

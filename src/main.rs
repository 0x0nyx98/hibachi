use std::fs;

use eframe::{egui::{self, load::TextureLoadResult, CollapsingResponse, Id}, glow::{Fence, PACK_COMPRESSED_BLOCK_DEPTH}};
use egui_modal::Modal;

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

    world_editor_selected_world: usize
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
            world_editor_selected_world: 0
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

                for n in 0..self.levels[self.world_editor_selected_world].len() {
                    ui.horizontal(|ui| {
                        ui.label(format!("Level #{}", n + 1));

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

        egui::TopBottomPanel::top(Id::new("topmenu")).show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.add(egui::Button::image(egui::include_image!("../pics/open.png"))).clicked() {
                    if let Some(path) = tinyfiledialogs::open_file_dialog("Open ROM", "smb1.nes", None) {
                        match (fs::read(&path)) {
                            Ok(v) => {
                                self.rom = Some(v.into_boxed_slice());
                                if self.validate_rom() {
                                    self.rom_path = path.clone();

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

        egui::SidePanel::left("toolbox").exact_width(200.0).show(ctx, |ui| {
            if self.is_rom_open() {
                if ui.button("Edit World Headers").clicked() {
                    world_header_editor.open();
                }

                if ui.button("Add or Remove Areas").clicked() {
                    area_list_editor.open();
                }
            }else{
                ui.heading("No Rom Loaded");
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            /* ui.heading("My egui Application");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Increment").clicked() {
                self.age += 1;
            }
            ui.label(format!("Hello '{}', age {}", self.name, self.age));

            ui.image(egui::include_image!("../../../crates/egui/assets/ferris.png")); */
        });
    }
}

impl HibachiApp {
    fn err(&mut self, errstring: &str) {
        self.lasterr = String::from(errstring);
        self.errbox[0].open();
    }

    fn validate_rom(&mut self) -> bool {
        let raw_rom = self.rom.clone().unwrap();

        if !(raw_rom[0..4] == [0x4e, 0x45, 0x53, 0x1a]) {
            self.err("ROM is not in iNES format!");
            return false;
        }

        // todo theres gotta some clever way to like actually figure out if the rom is mario or not :skull: i cant wait to try loading up battletoads in this shit

        true
    }

    // (vanilla info)

        // level ids start at 0x1CCC
        // world offsets 0x1CC4

        // tile data offsets start at 0x1D38
        // sprite data offsets start at 0x1CF0

        // spr pointer high begins @ 0x1D16
        // spr pointer low 0x1CF4
        // tile point high 0x1D5E
        // tile point low 0x1D3C

    fn reload_levels_unpatched(&mut self) {
        let raw_rom = self.rom.clone().unwrap();

        let mut world_offs: [usize; 9] = [0; 9];

        for i in 0x1CC4..0x1CCC {
            world_offs[i - 0x1CC4] = raw_rom[i] as usize;
        }

        world_offs[8] = 36;

        let mut tile_offs: [usize; 5] = [0; 5];

        for i in 0x1D38..0x1D3C {
            tile_offs[i - 0x1D38] = raw_rom[i] as usize;
        }

        let mut spr_offs: [usize; 5] = [0; 5];

        for i in 0x1CF0..0x1CF4 {
            spr_offs[i - 0x1CF0] = raw_rom[i] as usize;
        }

        for n in 0..8 {
            let mut k = world_offs[n];

            while (k < world_offs[n + 1]) {
                let arid = raw_rom[0x1CCC + k] as usize;
                self.levels[n].push([(arid / 0x20) % 4, arid % 0x20]);
                k += 1;
            }
        }

        // note2self there are 34 (0x22) valid araes

        for n in 0..4 {
            let mut k = spr_offs[n];
            let mut t = tile_offs[n];

            'arload: while k < 0x22 {
                self.load_area_from_pointers(n,
                    HibachiApp::rom_addr(((raw_rom[0x1D5E + t] as usize) << 8) + (raw_rom[0x1D3C + t] as usize)),
                    HibachiApp::rom_addr(((raw_rom[0x1D16 + k] as usize) << 8) + (raw_rom[0x1CF4 + k] as usize))
                );

                k += 1;
                t += 1;

                for j in 0..4 {
                    if spr_offs[j] == k {break 'arload;}
                }
            }
        }
    }

    // (after org patch)

        // level ids 0x1CD4
        // world offsets 0x1CCC (00 05 0a 0e...)

        // tile data off 0x1CC8
        // spr data off 0x1CC4

        // spr pointer high begins @ 0x1D1A
        // spr pointer low 0x1CF8
        // tile point high 0x1D5E
        // tile point low 0x1D3C

    fn reload_levels(&mut self) {

    }

    fn rewrite_levels(&mut self) {
        // max capacity 2EEA - 1D80 !!!!!!!!!!
    }

    #[inline(always)]
    fn rom_addr(n: usize) -> usize {
        n - 32752
    }

    fn is_patched_organisation(&self) -> bool {
        let raw_rom = self.rom.clone().unwrap();
        raw_rom[0x1C27] == 0xBC
    }

    fn install_organisation_patch(&mut self) {
        let mut raw_rom = self.rom.clone().unwrap();

        raw_rom[0x1C27] = 0xBC;
        raw_rom[0x1C2F] = 0xC4;
        raw_rom[0x1C42] = 0xB4;
        raw_rom[0x1C4A] = 0xE8;
        raw_rom[0x1C4F] = 0x0A;
        raw_rom[0x1C57] = 0xB8;
        raw_rom[0x1C58] = 0x9C;
        raw_rom[0x5F3A] = 0xBC;
        raw_rom[0x5F3D] = 0xC4;

        self.rom = Some(raw_rom);
    }

    fn is_rom_open(&self) -> bool {
        match (&self.rom) {
            Some(_) => { true },
            None => { false }
        }
    }

    fn load_area_from_pointers(&mut self, atype: usize, tile_start: usize, spr_start: usize) {
        let raw_rom = self.rom.clone().unwrap();
        let mut a = Area::default_for_type(atype);

        let head_hi = raw_rom[tile_start];
        let head_lo = raw_rom[tile_start + 1];

        a.timer = match (head_hi >> 6) % 4 {
            1 => AreaTimerSetting::T400,
            2 => AreaTimerSetting::T300,
            3 => AreaTimerSetting::T200,
            _ => AreaTimerSetting::Sublevel
        };

        a.mario_height = match (head_hi >> 3) % 8 {
            0 => AreaMarioStartHeightSetting::VeryHigh,
            1 => AreaMarioStartHeightSetting::High,
            2 => AreaMarioStartHeightSetting::Ground,
            3 => AreaMarioStartHeightSetting::Midair,
            _ => AreaMarioStartHeightSetting::Autowalk
        };

        a.bg = match head_hi % 8 {
            0 => AreaBackdropSetting::Overworld,
            1 => AreaBackdropSetting::Underwater,
            2 => AreaBackdropSetting::GreatWall,
            3 => AreaBackdropSetting::Seaside,
            4 => AreaBackdropSetting::Nighttime,
            5 => AreaBackdropSetting::SnowDay,
            6 => AreaBackdropSetting::SnowNight,
            _ => AreaBackdropSetting::Grayscale
        };

        a.special = match (head_lo >> 6) % 4 {
            0 => AreaSpecialSetting::Ordinary,
            1 => AreaSpecialSetting::Mushroom,
            2 => AreaSpecialSetting::Cannon,
            _ => AreaSpecialSetting::Sky
        };

        a.bg2 = match (head_lo >> 4) % 4 {
            0 => AreaScenerySetting::Blank,
            1 => AreaScenerySetting::Clouds,
            2 => AreaScenerySetting::Hills,
            _ => AreaScenerySetting::Fences
        };

        a.initial_tilepat = (head_lo % 16) as u8;

        let mut t = tile_start + 2;
        let mut scrx = 0usize;

        while (raw_rom[t] != 0xFD) {
            let pos = raw_rom[t];
            let data = raw_rom[t + 1];

            if data > 127 { scrx += 16; }

            let x = (pos >> 4) % 16;
            let y = pos % 16;

            let dat_hi = (data >> 4) % 8;
            let dat_lo = data % 16;

            if y < 12 {
                let result_obj = match dat_hi {
                    1 => TerrainObjectType::SpecialPlat(dat_lo),
                    2 => TerrainObjectType::BrickRow(dat_lo),
                    3 => TerrainObjectType::BlockRow(dat_lo),
                    4 => TerrainObjectType::CoinRow(dat_lo),
                    5 => TerrainObjectType::BrickColumn(dat_lo),
                    6 => TerrainObjectType::BlockColumn(dat_lo),
                    7 => TerrainObjectType::Pipe(dat_lo > 7, dat_lo % 8),

                    _ => match dat_lo {
                        0 => TerrainObjectType::PowerupQBlock,
                        1 => TerrainObjectType::CoinQBlock,
                        2 => TerrainObjectType::CoinHiddenBlock,
                        3 => TerrainObjectType::OneupHiddenBlock,
                        4 => TerrainObjectType::PowerupBrick,
                        5 => TerrainObjectType::VineBrick,
                        6 => TerrainObjectType::StarBrick,
                        7 => TerrainObjectType::LotsOfCoinsBrick,
                        8 => TerrainObjectType::OneupBrick,
                        9 => TerrainObjectType::HitBlock,
                        10 => TerrainObjectType::SidePipeExit,
                        11 => TerrainObjectType::Trampoline,
                        12 => TerrainObjectType::LPipe,
                        13 => TerrainObjectType::Flagpole,
                        _ => TerrainObjectType::RawHex(data)
                    }
                };

                a.terrain.push(AreaTerrainObject {
                    variety: result_obj,
                    x: scrx as usize + x as usize,
                    y: y
                });
            }

            if y == 12 {
                let result_obj = match dat_hi {
                    0 => TerrainObjectType::Hole(dat_lo),
                    1 => TerrainObjectType::Pulley(dat_lo),
                    2 => TerrainObjectType::Y7Bridge(dat_lo),
                    3 => TerrainObjectType::Y8Bridge(dat_lo),
                    4 => TerrainObjectType::Y10Bridge(dat_lo),
                    5 => TerrainObjectType::WaterHole(dat_lo),
                    6 => TerrainObjectType::Y3QBlocks(dat_lo),
                    _ => TerrainObjectType::Y7QBlocks(dat_lo)
                };

                a.terrain.push(AreaTerrainObject {
                    variety: result_obj,
                    x: scrx as usize + x as usize,
                    y: 0
                });
            }

            if y == 13 && (dat_hi < 4) {
                scrx = 16 * (((dat_hi % 2) << 4) + dat_lo) as usize;
            }

            if y == 13 && (dat_hi > 3) {
                let result_obj = match ((dat_hi % 2) << 4) + dat_lo {
                    0 => TerrainObjectType::LPipe,
                    1 => TerrainObjectType::Flagpole,
                    2 => TerrainObjectType::BridgeAx,
                    3 => TerrainObjectType::BridgeChain,
                    4 => TerrainObjectType::BowserBridge,
                    5 => TerrainObjectType::WarpZoneCommand,
                    6 => TerrainObjectType::NoScrollCommand,
                    7 => TerrainObjectType::NoScroll2Command,
                    8 => TerrainObjectType::CheepCheepSpawnCommand,
                    9 => TerrainObjectType::BulletSpawnCommand,
                    10 => TerrainObjectType::NoSpawnCommand,
                    11 => TerrainObjectType::EndlessHallwayCommand,
                    _ =>  TerrainObjectType::RawHex(data)
                };

                a.terrain.push(AreaTerrainObject {
                    variety: result_obj,
                    x: scrx as usize + x as usize,
                    y: 0
                });
            }

            if y == 14 && (dat_hi < 4)  {
                a.terrain.push(AreaTerrainObject {
                    variety: TerrainObjectType::SceneryAndFloorPatternSwitch(dat_hi % 4, dat_lo),
                    x: scrx as usize + x as usize,
                    y: 0
                });
            }

            if y == 14 && (dat_hi > 3)  {
                a.terrain.push(AreaTerrainObject {
                    variety: TerrainObjectType::BackdropSwitchCommand(dat_lo % 8),
                    x: scrx as usize + x as usize,
                    y: 0
                });
            }

            if y == 15 {
                let result_obj = match dat_hi {
                    0 => TerrainObjectType::LeftPulleyRope(dat_lo),
                    1 => TerrainObjectType::RightPulleyRope(dat_lo),
                    2 => TerrainObjectType::Fortress(dat_lo),
                    3 => TerrainObjectType::BlockStaircase(dat_lo),
                    4 => TerrainObjectType::ExitLPipe(dat_lo),
                    5 => TerrainObjectType::UnusedVine(dat_lo),
                    _ => TerrainObjectType::RawHex(data)
                };

                a.terrain.push(AreaTerrainObject {
                    variety: result_obj,
                    x: scrx as usize + x as usize,
                    y: 0
                });
            }

            t += 2;
        }

        let mut s = spr_start;
        let mut scrx = 0usize;

        while (raw_rom[s] != 0xFF) {
            let pos = raw_rom[t];
            let data = raw_rom[t + 1];

            if data > 127 { scrx += 16; }

            let x = (pos >> 4) % 16;
            let y = pos % 16;

            if y == 15 {
                scrx = 16 * (data % 32) as usize;
            }

            else if y == 14 {
                // warp info handling is todo
                s += 1;
            }

            else {
                let objtype = match data % 64 {
                    0 => SpriteObjectType::GreenKoopa,
                    1 => SpriteObjectType::StupidRedKoopa,
                    2 => SpriteObjectType::Buzzy,
                    3 => SpriteObjectType::RedKoopa,
                    4 => SpriteObjectType::ReallyStupidGreenKoopa,
                    5 => SpriteObjectType::HammerBro,
                    6 => SpriteObjectType::Goomba,
                    7 => SpriteObjectType::Blooper,
                    8 => SpriteObjectType::BulletBill,
                    9 => SpriteObjectType::YellowParatroopa,
                    10 => SpriteObjectType::SlowCheep,
                    11 => SpriteObjectType::FastCheep,
                    12 => SpriteObjectType::Podoboo,
                    13 => SpriteObjectType::Pirahna,
                    14 => SpriteObjectType::JumpyGreenParatroopa,
                    15 => SpriteObjectType::VerticalRedParatroopa,
                    16 => SpriteObjectType::HorizontalGreenParatroopa,
                    17 => SpriteObjectType::Lakitu,
                    18 => SpriteObjectType::SpinyDontUse,
                    20 => SpriteObjectType::JumpingCheepsGenerator,
                    21 => SpriteObjectType::BowserFireGenerator,
                    22 => SpriteObjectType::FireworkGenerator,
                    23 => SpriteObjectType::BulletBillGenerator,
                    27 => SpriteObjectType::ClockwiseFireBar,
                    28 => SpriteObjectType::FastClockwiseFireBar,
                    29 => SpriteObjectType::CCWFireBar,
                    30 => SpriteObjectType::FastCCWFireBar,
                    31 => SpriteObjectType::BigClockwiseFireBar,
                    36 => SpriteObjectType::BalanceLift,
                    37 => SpriteObjectType::UpAndDownLift,
                    38 => SpriteObjectType::UpLift,
                    39 => SpriteObjectType::DownLift,
                    40 => SpriteObjectType::AcrossLift,
                    41 => SpriteObjectType::FallingLift,
                    42 => SpriteObjectType::RightLift,
                    43 => SpriteObjectType::ShortUpLift,
                    44 => SpriteObjectType::ShortDownLift,
                    45 => SpriteObjectType::Bowser,
                    52 => SpriteObjectType::WarpZone,
                    53 => SpriteObjectType::Toad,
                    55 => SpriteObjectType::Y10_2Goombas,
                    56 => SpriteObjectType::Y10_3Goombas,
                    57 => SpriteObjectType::Y6_2Goombas,
                    58 => SpriteObjectType::Y6_3Goombas,
                    59 => SpriteObjectType::Y10_2Koopas,
                    60 => SpriteObjectType::Y10_3Koopas,
                    61 => SpriteObjectType::Y6_2Koopas,
                    62 => SpriteObjectType::Y6_3Koopas,

                    _ => SpriteObjectType::RawHex(data % 64)
                };

                a.stuff.push(AreaSpriteObject {
                    variety: objtype,
                    lategame: (data % 128) > 63,
                    x: scrx + x as usize,
                    y: y
                })
            }

            s += 2;
        }

        self.areas[atype].push(a);
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

enum AreaTimerSetting {
    T200,    // 11
    T300,    // 10
    T400,    // 01
    Sublevel // 00
}

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

enum AreaSpecialSetting {
    Ordinary,
    Mushroom,
    Cannon,
    Sky
}

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

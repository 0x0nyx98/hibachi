use std::fs;

use eframe::{egui::{self, Id}, glow::PACK_COMPRESSED_BLOCK_DEPTH};
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

            egui::ScrollArea::vertical().show(ui, |ui| {
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
                                self.areas[rt].push(Area::default());
                            }
                        });
                    }else{
                        for n in 0..self.areas[rt].len() {
                            ui.horizontal(|ui| {
                                ui.label(format!("{:#04x}", n));

                                if ui.add_enabled(self.areas[rt].len() < 0x20, egui::Button::new("Add Above")).clicked() {
                                    self.areas[rt].insert(n, Area::default());
                                }

                                if ui.add_enabled(self.areas[rt].len() < 0x20, egui::Button::new("Add Below")).clicked() {
                                    self.areas[rt].insert(n + 1, Area::default());
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
                        match (fs::read(path)) {
                            Ok(v) => {
                                self.rom = Some(v.into_boxed_slice());
                                if self.validate_rom() {
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

                }

                if ui.add_enabled(self.is_rom_open(), egui::Button::image(egui::include_image!("../pics/saveas.png"))).clicked() {

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
                    ((raw_rom[0x1D5E + t] as usize) << 8) + (raw_rom[0x1D3C + t] as usize),
                    ((raw_rom[0x1D16 + k] as usize) << 8) + (raw_rom[0x1CF4 + k] as usize)
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
    fn rom_addr(n: i32) -> i32 {
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
        let mut a = Area {};

        self.areas[atype].push(a);
    }
}

struct Area {

}

impl Default for Area {
    fn default() -> Self {
        Self {}
    }
}

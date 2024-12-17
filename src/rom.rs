use crate::*;

impl HibachiApp {
    pub fn validate_rom(&mut self) -> bool {
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

    pub fn reload_levels_unpatched(&mut self) {
        self.levels = [vec!(), vec!(), vec!(), vec!(), vec!(), vec!(), vec!(), vec!()];
        self.areas = [vec!(), vec!(), vec!(), vec!()];

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
                    rom_addr(((raw_rom[0x1D5E + t] as usize) << 8) + (raw_rom[0x1D3C + t] as usize)),
                    rom_addr(((raw_rom[0x1D16 + k] as usize) << 8) + (raw_rom[0x1CF4 + k] as usize))
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

    pub fn reload_levels(&mut self) {

    }

    pub fn rewrite_levels(&mut self) {
        // max capacity 2EEA - 1D80 !!!!!!!!!!
    }

    pub fn is_patched_organisation(&self) -> bool {
        let raw_rom = self.rom.clone().unwrap();
        raw_rom[0x1C27] == 0xBC
    }

    pub fn install_organisation_patch(&mut self) {
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

    pub fn is_rom_open(&self) -> bool {
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
            let pos = raw_rom[s];
            let data = raw_rom[s + 1];

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

    pub fn reload_graphics(&mut self, ctx: &egui::Context) {
        self.game_graphics = HashMap::new();

        self.reload_palettes();
        self.reload_sprite(ctx, MarioGraphics::MarioStart, MarioColor::Mario, vec!(
            [0,0,0x32,1],
            [8,0,0x33,1],
            [0,8,0x34,1],
            [8,8,0x35,1],
        ));

        self.reload_sprite(ctx, MarioGraphics::TerrainRocky, MarioColor::Brick, vec!(
            [0,0,0x1B4,1],
            [8,0,0x1B5,1],
            [0,8,0x1B6,1],
            [8,8,0x1B7,1],
        ));

        self.reload_sprite(ctx, MarioGraphics::TerrainSearock, MarioColor::Brick, vec!(
            [0,0,0x182,1],
            [8,0,0x184,1],
            [0,8,0x183,1],
            [8,8,0x185,1],
        ));

        self.reload_sprite(ctx, MarioGraphics::TerrainCastleBrick, MarioColor::Brick, vec!(
            [0,0,0x15E,1],
            [8,0,0x15E,1],
            [0,8,0x15D,1],
            [8,8,0x15D,1],
        ));

        self.reload_sprite(ctx, MarioGraphics::SpriteObject(SpriteObjectType::GreenKoopa), MarioColor::GreenEnemy, vec!(
            [0,-8,0xA5,-1],
            [0,0,0xA7,-1],
            [8,0,0xA6,-1],
            [0,8,0xA9,-1],
            [8,8,0xA8,-1],
        ));

        self.reload_sprite(ctx, MarioGraphics::SpriteObject(SpriteObjectType::StupidRedKoopa), MarioColor::RedEnemy, vec!(
            [0,-8,0xA5,-1],
            [8,-8,0x12B,1],
            [0,0,0xA7,-1],
            [8,0,0xA6,-1],
            [0,8,0xA9,-1],
            [8,8,0xA8,-1],
        ));

        self.reload_sprite(ctx, MarioGraphics::SpriteObject(SpriteObjectType::Buzzy), MarioColor::BlackEnemy, vec!(
            [0,0,0xAB,-1],
            [8,0,0xAA,-1],
            [0,8,0xAD,-1],
            [8,8,0xAC,-1],
        ));

        self.reload_sprite(ctx, MarioGraphics::SpriteObject(SpriteObjectType::RedKoopa), MarioColor::RedEnemy, vec!(
            [0,-8,0xA5,-1],
            [0,0,0xA7,-1],
            [8,0,0xA6,-1],
            [0,8,0xA9,-1],
            [8,8,0xA8,-1],
        ));

        self.reload_sprite(ctx, MarioGraphics::SpriteObject(SpriteObjectType::ReallyStupidGreenKoopa), MarioColor::GreenEnemy, vec!(
            [0,-8,0xA5,-1],
            [8,-8,0x12B,1],
            [0,0,0xA7,-1],
            [8,0,0xA6,-1],
            [0,8,0xA9,-1],
            [8,8,0xA8,-1],
        ));

        self.reload_sprite(ctx, MarioGraphics::SpriteObject(SpriteObjectType::HammerBro), MarioColor::GreenEnemy, vec!(
            [0,-8,0x7C,-1],
            [8,-8,0x7D,-1],
            [0,0,0x8C,-1],
            [8,0,0x89,-1],
            [0,8,0x8A,-1],
            [8,8,0x8B,-1],
        ));

        self.reload_sprite(ctx, MarioGraphics::SpriteObject(SpriteObjectType::Goomba), MarioColor::BlackEnemy, vec!(
            [0,0,0x70,1],
            [8,0,0x71,1],
            [0,8,0x72,1],
            [8,8,0x73,1],
        ));

        self.reload_sprite(ctx, MarioGraphics::SpriteObject(SpriteObjectType::Blooper), MarioColor::GreenEnemy, vec!(
            [0,0,0xDC,1],
            [8,0,0xDC,-1],
            [0,8,0xDD,1],
            [8,8,0xDD,-1],
            [0,16,0xDE,1],
            [8,16,0xDE,-1],
        ));

        self.reload_sprite(ctx, MarioGraphics::SpriteObject(SpriteObjectType::BulletBill), MarioColor::BlackEnemy, vec!(
            [0,0,0xE7,1],
            [8,0,0xE8,1],
            [0,8,0xE9,1],
            [8,8,0xEA,1],
        ));

        self.reload_font(ctx);
    }

    fn reload_sprite(&mut self, ctx: &egui::Context, g: MarioGraphics, p: MarioColor, patches: Vec<[isize; 4]>) {
        let raw_rom = self.rom.clone().unwrap();

        let mut min_x = isize::MAX;
        let mut min_y = isize::MAX;
        let mut max_x = isize::MIN;
        let mut max_y = isize::MIN;

        for p in patches.iter() {
            if p[0] < min_x { min_x = p[0]; }
            if p[0] + 8 > max_x { max_x = p[0] + 8; }
            if p[1] < min_y { min_y = p[1]; }
            if p[1] + 8 > max_y { max_y = p[1] + 8; }
        }

        let mut i1 = egui::ColorImage::new([(max_x - min_x) as usize, (max_y - min_y) as usize], Color32::TRANSPARENT);
        let mut i2 = egui::ColorImage::new([(max_x - min_x) as usize, (max_y - min_y) as usize], Color32::TRANSPARENT);
        let mut i3 = egui::ColorImage::new([(max_x - min_x) as usize, (max_y - min_y) as usize], Color32::TRANSPARENT);

        for p in patches.iter() {
            let hflip = p[3] == -1;

            let x = p[0] - min_x;
            let y = p[1] - min_y;

            let mut xx = if hflip {7} else {0};
            let mut yy = 0;

            let saddr =(0x8010 + 16 * p[2]) as usize;

            for b in 0..8 {
                for s in 0..8 {
                    let px = ((y + yy) * (max_x - min_x) + (x + xx)) as usize;

                    let shift = 7 - s;
                    let hi = (raw_rom[saddr + b + 8] >> shift) % 2;
                    let lo = (raw_rom[saddr + b] >> shift) % 2;
                    let d = 2 * hi + lo;

                    if d == 1 { i1.pixels[px] = Color32::WHITE; }
                    if d == 2 { i2.pixels[px] = Color32::WHITE; }
                    if d == 3 { i3.pixels[px] = Color32::WHITE; }

                    xx += if hflip {-1} else {1};
                    if xx == (if hflip {-1} else {8}) { xx = if hflip {7} else {0}; yy += 1; }
                }
            }
        }

        let c1 = ctx.load_texture("texture" /* :skull */, i1, TextureOptions::NEAREST);
        let c2 = ctx.load_texture("texture" /* :skull */, i2, TextureOptions::NEAREST);
        let c3 = ctx.load_texture("texture" /* :skull */, i3, TextureOptions::NEAREST);

        self.game_graphics.insert(g,
            ViewportSprite {
                slices: [c1, c2, c3],
                offset: [min_x, min_y],
                size: [(max_x - min_x) as usize, (max_y - min_y) as usize],
                palette: p,
            }
        );
    }

    fn reload_font(&mut self, ctx: &egui::Context) {
        let raw_rom = self.rom.clone().unwrap();

        let mut bn = 0x10A;

        for c in "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars() {
            let mut i: egui::ColorImage = egui::ColorImage::new([8, 8], Color32::TRANSPARENT);

            let x = 0;
            let y = 0;

            let mut xx = 0;
            let mut yy = 0;

            let saddr =(0x8010 + 16 * bn) as usize;

            for b in 0..8 {
                for s in 0..8 {
                    let px = ((y + yy) * 8 + (x + xx)) as usize;

                    let shift = 7 - s;
                    let hi = (raw_rom[saddr + b + 8] >> shift) % 2;
                    let lo = (raw_rom[saddr + b] >> shift) % 2;
                    let d = 2 * hi + lo;

                    if d != 0 { i.pixels[px] = Color32::WHITE; }

                    xx += 1;
                    if xx == 8 { xx = 0; yy += 1; }
                }
            }
            
            let c1 = ctx.load_texture("fontLetter", i.clone(), TextureOptions::NEAREST);
            let c2 = ctx.load_texture("fontLetter", i.clone(), TextureOptions::NEAREST);
            let c3 = ctx.load_texture("fontLetter", i, TextureOptions::NEAREST);
    
            self.game_font.insert(c,
                ViewportSprite {
                    slices: [c1, c2, c3],
                    offset: [0, 0],
                    size: [8, 8],
                    palette: MarioColor::BlackEnemy,
                }
            );

            bn += 1;
        }
    }

    fn reload_palettes(&mut self) {
        self.reload_palette(MarioColorPalette::Water, 0x0CB4, 0x05DF);
        self.reload_palette(MarioColorPalette::Ground, 0x0CD8, 0x05E0);
        self.reload_palette(MarioColorPalette::Cave, 0x0CFC, 0x05E1);
        self.reload_palette(MarioColorPalette::Castle, 0x0D20, 0x05E2);

        // temporary:
        self.game_palette.insert(MarioColorPalette::Night, self.game_palette.get(&MarioColorPalette::Ground).unwrap().clone());
        self.game_palette.insert(MarioColorPalette::SnowDay, self.game_palette.get(&MarioColorPalette::Ground).unwrap().clone());
        self.game_palette.insert(MarioColorPalette::SnowNight, self.game_palette.get(&MarioColorPalette::Ground).unwrap().clone());
        self.game_palette.insert(MarioColorPalette::Mushroom, self.game_palette.get(&MarioColorPalette::Ground).unwrap().clone());
        self.game_palette.insert(MarioColorPalette::Bowser, self.game_palette.get(&MarioColorPalette::Castle).unwrap().clone());
    }

    fn reload_palette(&mut self, pal: MarioColorPalette, offset: usize, sky_offset: usize) {
        let raw_rom = self.rom.clone().unwrap();

        self.game_skycolor.insert(pal.clone(), nes_color(raw_rom[sky_offset]));

        let mut p = HashMap::new();

        p.insert(MarioColor::Vegetation, [nes_color(raw_rom[offset + 4]), nes_color(raw_rom[offset + 5]), nes_color(raw_rom[offset + 6])]);
        p.insert(MarioColor::Brick, [nes_color(raw_rom[offset + 8]), nes_color(raw_rom[offset + 9]), nes_color(raw_rom[offset + 10])]);
        p.insert(MarioColor::Cloud, [nes_color(raw_rom[offset + 12]), nes_color(raw_rom[offset + 13]), nes_color(raw_rom[offset + 14])]);
        p.insert(MarioColor::Shiny, [nes_color(raw_rom[offset + 16]), nes_color(raw_rom[offset + 17]), nes_color(raw_rom[offset + 18])]);
        p.insert(MarioColor::Mario, [nes_color(raw_rom[offset + 20]), nes_color(raw_rom[offset + 21]), nes_color(raw_rom[offset + 22])]);
        p.insert(MarioColor::GreenEnemy, [nes_color(raw_rom[offset + 24]), nes_color(raw_rom[offset + 25]), nes_color(raw_rom[offset + 26])]);
        p.insert(MarioColor::RedEnemy, [nes_color(raw_rom[offset + 28]), nes_color(raw_rom[offset + 29]), nes_color(raw_rom[offset + 30])]);
        p.insert(MarioColor::BlackEnemy, [nes_color(raw_rom[offset + 32]), nes_color(raw_rom[offset + 33]), nes_color(raw_rom[offset + 34])]);

        self.game_palette.insert(pal.clone(), p);
    }
}
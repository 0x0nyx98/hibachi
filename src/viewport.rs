use crate::*;

impl HibachiApp {
    pub fn paint_sprite(&self, pntr: &egui::Painter, o: Vec2, x: isize, y: isize, spr: MarioGraphics, pal: MarioColorPalette) {
        let sprdata = &self.game_graphics[&spr];

        pntr.image(egui::TextureId::from(&sprdata.slices[0]), egui::Rect::from_points(&[
            egui::pos2((x + 2 * sprdata.offset[0]) as f32, (y + 2 * sprdata.offset[1]) as f32),
            egui::pos2((x + 2 * sprdata.offset[0] + 2 * sprdata.size[0] as isize) as f32, (y + 2 * sprdata.offset[1] + 2 * sprdata.size[1] as isize) as f32)
        ]).translate(o), egui::Rect{min: egui::pos2(0.0, 0.0), max: egui::pos2(1.0, 1.0)}, self.game_palette[&pal][&sprdata.palette][0]);

        pntr.image(egui::TextureId::from(&sprdata.slices[1]), egui::Rect::from_points(&[
            egui::pos2((x + 2 * sprdata.offset[0]) as f32, (y + 2 * sprdata.offset[1]) as f32),
            egui::pos2((x + 2 * sprdata.offset[0] + 2 * sprdata.size[0] as isize) as f32, (y + 2 * sprdata.offset[1] + 2 * sprdata.size[1] as isize) as f32)
        ]).translate(o), egui::Rect{min: egui::pos2(0.0, 0.0), max: egui::pos2(1.0, 1.0)}, self.game_palette[&pal][&sprdata.palette][1]);

        pntr.image(egui::TextureId::from(&sprdata.slices[2]), egui::Rect::from_points(&[
            egui::pos2((x + 2 * sprdata.offset[0]) as f32, (y + 2 * sprdata.offset[1]) as f32),
            egui::pos2((x + 2 * sprdata.offset[0] + 2 * sprdata.size[0] as isize) as f32, (y + 2 * sprdata.offset[1] + 2 * sprdata.size[1] as isize) as f32)
        ]).translate(o), egui::Rect{min: egui::pos2(0.0, 0.0), max: egui::pos2(1.0, 1.0)}, self.game_palette[&pal][&sprdata.palette][2]);
    }

    pub fn paint_silhouette(&self, pntr: &egui::Painter, o: Vec2, x: isize, y: isize, spr: MarioGraphics, pal: MarioColorPalette) {
        let sprdata = &self.game_graphics[&spr];

        pntr.image(egui::TextureId::from(&sprdata.slices[0]), egui::Rect::from_points(&[
            egui::pos2((x + 2 * sprdata.offset[0]) as f32, (y + 2 * sprdata.offset[1]) as f32),
            egui::pos2((x + 2 * sprdata.offset[0] + 2 * sprdata.size[0] as isize) as f32, (y + 2 * sprdata.offset[1] + 2 * sprdata.size[1] as isize) as f32)
        ]).translate(o), egui::Rect{min: egui::pos2(0.0, 0.0), max: egui::pos2(1.0, 1.0)}, self.game_palette[&pal][&sprdata.palette][0]);

        pntr.image(egui::TextureId::from(&sprdata.slices[1]), egui::Rect::from_points(&[
            egui::pos2((x + 2 * sprdata.offset[0]) as f32, (y + 2 * sprdata.offset[1]) as f32),
            egui::pos2((x + 2 * sprdata.offset[0] + 2 * sprdata.size[0] as isize) as f32, (y + 2 * sprdata.offset[1] + 2 * sprdata.size[1] as isize) as f32)
        ]).translate(o), egui::Rect{min: egui::pos2(0.0, 0.0), max: egui::pos2(1.0, 1.0)}, self.game_palette[&pal][&sprdata.palette][0]);

        pntr.image(egui::TextureId::from(&sprdata.slices[2]), egui::Rect::from_points(&[
            egui::pos2((x + 2 * sprdata.offset[0]) as f32, (y + 2 * sprdata.offset[1]) as f32),
            egui::pos2((x + 2 * sprdata.offset[0] + 2 * sprdata.size[0] as isize) as f32, (y + 2 * sprdata.offset[1] + 2 * sprdata.size[1] as isize) as f32)
        ]).translate(o), egui::Rect{min: egui::pos2(0.0, 0.0), max: egui::pos2(1.0, 1.0)}, self.game_palette[&pal][&sprdata.palette][0]);
    }

    pub fn paint_tag(&self, pntr: &egui::Painter, o: Vec2, x: isize, y: isize, txt: &str, txt2: &str, pal: MarioColorPalette) {
        let mut i = 0;

        pntr.rect_filled(
            egui::Rect::from_points(&[
                egui::pos2(x as f32, y as f32),
                egui::pos2((x + 16 * (txt.len().max(txt2.len()) + 1) as isize) as f32, (y + 32 as isize) as f32)
            ]).translate(o),
            egui::Rounding::ZERO,
            self.game_palette[&pal][&MarioColor::BlackEnemy][2],
        );
        
        for c in txt.to_uppercase().chars() {
            if c != ' ' {
                pntr.image(egui::TextureId::from(&(self.game_font)[&c].slices[0]), egui::Rect::from_points(&[
                    egui::pos2((x + 16 * i) as f32, y as f32),
                    egui::pos2((x + 16 * (i + 1)) as f32, (y + 16) as f32)
                ]).translate(o), egui::Rect{min: egui::pos2(0.0, 0.0), max: egui::pos2(1.0, 1.0)}, self.game_palette[&pal][&MarioColor::BlackEnemy][1]);
            }
            
            i += 1;
        }

        i = 0;
        
        for c in txt2.to_uppercase().chars() {
            if c != ' ' {
                pntr.image(egui::TextureId::from(&(self.game_font)[&c].slices[0]), egui::Rect::from_points(&[
                    egui::pos2((x + 16 * i) as f32, (y + 16) as f32),
                    egui::pos2((x + 16 * (i + 1)) as f32, (y + 32) as f32)
                ]).translate(o), egui::Rect{min: egui::pos2(0.0, 0.0), max: egui::pos2(1.0, 1.0)}, self.game_palette[&pal][&MarioColor::BlackEnemy][1]);
            }

            i += 1;
        }
    }
}
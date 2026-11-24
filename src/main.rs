use macroquad::prelude::*;
use macroquad::rand::gen_range;

const TILE_W: f32 = 64.0;
const TILE_H: f32 = 32.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum TileKind {
    Empty,
    Water,
    Sand,
    Ore,
}

impl TileKind {
    fn base_color(self) -> Color {
        match self {
            TileKind::Empty => Color::from_rgba(110, 75, 48, 255),
            TileKind::Water => Color::from_rgba(55, 120, 195, 255),
            TileKind::Sand => Color::from_rgba(235, 215, 130, 255),
            TileKind::Ore => Color::from_rgba(28, 30, 34, 255),
        }
    }
}

struct Game {
    cursor: (usize, usize),
    selected: Option<(usize, usize)>,
    camera_offset: Vec2,
    last_right_mouse: Option<Vec2>,
    map: Vec<TileKind>,
    width: usize,
    height: usize,
}

impl Game {
    fn new(starting_width: usize, starting_height: usize) -> Self {
        let mut random_map = Vec::with_capacity(starting_width * starting_height);
        for _ in 0..(starting_width * starting_height) {
            let roll = gen_range(0, 100);
            let tile = if roll < 20 {
                TileKind::Water
            } else if roll < 40 {
                TileKind::Sand
            } else if roll < 52 {
                TileKind::Ore
            } else {
                TileKind::Empty
            };
            random_map.push(tile);
        }
        Self {
            cursor: (0, 0),
            selected: None,
            camera_offset: vec2(0.0, 0.0),
            last_right_mouse: None,
            map: random_map,
            width: starting_width,
            height: starting_height,
        }
    }

    fn tile_at(&self, gx: usize, gy: usize) -> Option<TileKind> {
        if let Some(index) = self.grid_index(gx, gy) {
            return Some(self.map[index]);
        }
        None
    }

    fn grid_index(&self, gx: usize, gy: usize) -> Option<usize> {
        if gx < self.width && gy < self.height {
            return Some(gy * self.width + gx);
        }
        None
    }

    fn grid_origin(&self) -> (f32, f32) {
        (
            screen_width() * 0.5 + self.camera_offset.x,
            120.0 + self.camera_offset.y,
        )
    }

    fn grid_to_screen(&self, gx: usize, gy: usize, origin_x: f32, origin_y: f32) -> (f32, f32) {
        let fx = gx as f32;
        let fy = gy as f32;
        let sx = origin_x + (fx - fy) * (TILE_W * 0.5);
        let sy = origin_y + (fx + fy) * (TILE_H * 0.5);
        (sx, sy)
    }

    fn screen_to_grid(&self, sx: f32, sy: f32) -> Option<(usize, usize)> {
        let (origin_x, origin_y) = self.grid_origin();
        let hw = TILE_W * 0.5;
        let hh = TILE_H * 0.5;

        let dx = sx - origin_x;
        let dy = sy - origin_y + hh;

        let fx = (dx / hw + dy / hh) * 0.5;
        let fy = (dy / hh - dx / hw) * 0.5;

        if fx < 0.0 || fy < 0.0 {
            return None;
        }

        let gx = fx.floor() as usize;
        let gy = fy.floor() as usize;

        if gx < self.width && gy < self.height {
            Some((gx, gy))
        } else {
            None
        }
    }

    fn draw_iso_tile(
        &self,
        gx: usize,
        gy: usize,
        origin_x: f32,
        origin_y: f32,
        fill: Color,
        outline: Color,
    ) {
        let (cx, cy) = self.grid_to_screen(gx, gy, origin_x, origin_y);
        let hw = TILE_W * 0.5;
        let hh = TILE_H * 0.5;

        let top = vec2(cx, cy - hh);
        let right = vec2(cx + hw, cy);
        let bottom = vec2(cx, cy + hh);
        let left = vec2(cx - hw, cy);

        draw_triangle(top, right, bottom, fill);
        draw_triangle(top, bottom, left, fill);

        draw_line(top.x, top.y, right.x, right.y, 1.2, outline);
        draw_line(right.x, right.y, bottom.x, bottom.y, 1.2, outline);
        draw_line(bottom.x, bottom.y, left.x, left.y, 1.2, outline);
        draw_line(left.x, left.y, top.x, top.y, 1.2, outline);
    }
    fn update(&mut self) {
        let pan_speed = 320.0 * get_frame_time();
        if is_key_down(KeyCode::D) {
            self.camera_offset.x -= pan_speed;
        }
        if is_key_down(KeyCode::A) {
            self.camera_offset.x += pan_speed;
        }
        if is_key_down(KeyCode::S) {
            self.camera_offset.y -= pan_speed;
        }
        if is_key_down(KeyCode::W) {
            self.camera_offset.y += pan_speed;
        }

        let mouse_pos = vec2(mouse_position().0, mouse_position().1);
        if is_mouse_button_down(MouseButton::Right) {
            if let Some(prev) = self.last_right_mouse {
                self.camera_offset += mouse_pos - prev;
            }
            self.last_right_mouse = Some(mouse_pos);
        } else {
            self.last_right_mouse = None;
        }

        let mut moved_by_key = false;

        if is_key_pressed(KeyCode::Right) {
            self.cursor.0 = (self.cursor.0 + 1).min(self.width - 1);
            moved_by_key = true;
        }
        if is_key_pressed(KeyCode::Left) {
            self.cursor.0 = self.cursor.0.saturating_sub(1);
            moved_by_key = true;
        }
        if is_key_pressed(KeyCode::Down) {
            self.cursor.1 = (self.cursor.1 + 1).min(self.height - 1);
            moved_by_key = true;
        }
        if is_key_pressed(KeyCode::Up) {
            self.cursor.1 = self.cursor.1.saturating_sub(1);
            moved_by_key = true;
        }

        let hovered = self.screen_to_grid(mouse_pos.x, mouse_pos.y);

        if !moved_by_key && !is_mouse_button_down(MouseButton::Right) {
            if let Some(tile) = hovered {
                self.cursor = tile;
            }
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            if let Some(tile) = hovered {
                self.cursor = tile;
                self.selected = Some(tile);
            }
        }

        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            self.selected = Some(self.cursor);
        }
    }

    fn draw(&self) {
        clear_background(Color::from_rgba(24, 28, 36, 255));

        let (origin_x, origin_y) = self.grid_origin();

        for gy in 0..self.height {
            for gx in 0..self.width {
                let is_cursor = self.cursor == (gx, gy);
                let is_selected = self.selected == Some((gx, gy));

                let mut fill = self.tile_at(gx, gy).unwrap_or(TileKind::Empty).base_color();

                if is_cursor {
                    fill.r = (fill.r * 0.82).clamp(0.0, 1.0);
                    fill.g = (fill.g * 0.82).clamp(0.0, 1.0);
                    fill.b = (fill.b * 0.82).clamp(0.0, 1.0);
                } else if is_selected {
                    fill.r = (fill.r + 0.15).clamp(0.0, 1.0);
                    fill.g = (fill.g + 0.15).clamp(0.0, 1.0);
                    fill.b = (fill.b + 0.15).clamp(0.0, 1.0);
                }

                let outline = Color::from_rgba(30, 36, 45, 255);
                self.draw_iso_tile(gx, gy, origin_x, origin_y, fill, outline);
            }
        }

        draw_text(
            &format!(
                "Cursor: ({}, {}) | Selected: {:?}",
                self.cursor.0, self.cursor.1, self.selected
            ),
            16.0,
            28.0,
            24.0,
            WHITE,
        );
        draw_text(
            "Pan: WASD / Right-Drag | Cursor: Arrows / Hover | Click / Enter: Select",
            16.0,
            54.0,
            20.0,
            LIGHTGRAY,
        );
    }
}

#[macroquad::main("Isometric Grid")]
async fn main() {
    let mut game = Game::new(16, 16);

    loop {
        game.update();
        game.draw();
        next_frame().await;
    }
}

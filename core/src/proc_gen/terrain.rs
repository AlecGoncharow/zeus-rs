use pantheon::Color;
use pantheon::Vec3;

use super::color::ColorGenerator;
use super::noise::Perlin;
use crate::entity::plane::Plane;
use crate::entity::terrain::Terrain;
use pantheon::graphics::vertex::ShadedVertex;

pub struct TerrainGenerator {
    perlin_noise: Perlin,
    color_gen: ColorGenerator,
}

impl TerrainGenerator {
    pub fn new(perlin_noise: Perlin, color_gen: ColorGenerator) -> Self {
        Self {
            perlin_noise,
            color_gen,
        }
    }

    pub fn generate(&self, size: usize) -> Terrain {
        println!("[terrain proc gen] generating heights");
        let heights: Vec<f32> = (0..(size + 1).pow(2))
            .map(|i| {
                let x = i % (size + 1);
                let z = i / (size + 1);
                self.perlin_noise.perlin_noise(x as isize, z as isize)
            })
            .collect();

        let amp = self.perlin_noise.amplitude;
        let amp_2 = amp * 2.0;
        //let partitions = self.color_gen.colors.len();
        let clamped_heights: Vec<f32> = heights
            .iter()
            .map(|height| {
                let ratio = (height + amp) / amp_2;
                //let ratio = height / amp;
                (ratio * amp as f32).floor()
            })
            .collect();

        println!("[terrain proc gen] generating colors");
        let colors = self
            .color_gen
            .generate(&heights, self.perlin_noise.amplitude);

        println!("[terrain proc gen] generating mesh");
        //let mesh = Self::create_mesh(&heights, &colors, size + 1);
        let mesh = Self::create_mesh(&clamped_heights, &colors, size + 1);

        println!("[terrain proc gen] generating indices");
        let indices = index_gen::generate_index_buffer(size + 1);
        println!("[terrain proc gen] done");

        //println!("[Mesh] {:#?}", mesh);
        println!("[Mesh] {:#?}", mesh.len());
        println!("[Indices] {:#?}", indices.len());

        Terrain::from_data(mesh, indices)
    }

    fn create_mesh(heights: &Vec<f32>, colors: &Vec<Color>, size: usize) -> Vec<ShadedVertex> {
        let mut buffer = Vec::with_capacity(size * size);
        let mut last_row: Vec<GridSquare> = Vec::with_capacity(size);
        for row in 0..(size - 1) {
            for col in 0..(size - 1) {
                let square = GridSquare::new(row, col, heights, colors, size);
                square.push_vertex_data(&mut buffer);
                if row == size - 2 {
                    last_row.push(square);
                }
            }
        }

        last_row
            .drain(..)
            .for_each(|square| square.push_bottom_data(&mut buffer));

        buffer
    }
}

mod index_gen {
    pub fn generate_index_buffer(vert_count: usize) -> Vec<u32> {
        let mut indices = Vec::with_capacity((vert_count - 1).pow(2) * 6);
        unsafe { indices.set_len((vert_count - 1).pow(2) * 6) }
        let row_len = (vert_count - 1) * 2;
        let mut pointer = store_top_section(&mut indices, row_len, vert_count);
        pointer = store_second_last(&mut indices, pointer, row_len, vert_count);
        store_last(&mut indices, pointer, row_len, vert_count);

        indices
    }

    fn store_top_section(indices: &mut [u32], row_len: usize, vert_count: usize) -> usize {
        let mut pointer = 0;
        for row in 0..(vert_count - 3) {
            for col in 0..(vert_count - 1) {
                let top_left = ((row * row_len) + (col * 2)) as u32;
                let top_right = top_left + 1;
                let bot_left = top_left + row_len as u32;
                let bot_right = bot_left + 1;

                let right_handed = col % 2 != row % 2;
                pointer = store_quad(
                    indices,
                    pointer,
                    right_handed,
                    top_left,
                    top_right,
                    bot_left,
                    bot_right,
                );
            }
        }

        pointer
    }

    fn store_second_last(
        indices: &mut [u32],
        mut pointer: usize,
        row_len: usize,
        vert_count: usize,
    ) -> usize {
        let row = vert_count - 3;
        for col in 0..(vert_count - 1) {
            let top_left = ((row * row_len) + (col * 2)) as u32;
            let top_right = top_left + 1;
            let bot_left = (top_left + row_len as u32) - col as u32;
            let bot_right = bot_left + 1;

            let right_handed = col % 2 != row % 2;
            //println!("[DEBUG] {:#?}", (top_left, top_right, bot_left, bot_right));
            pointer = store_quad(
                indices,
                pointer,
                right_handed,
                top_left,
                top_right,
                bot_left,
                bot_right,
            );
        }
        pointer
    }

    fn store_last(indices: &mut [u32], mut pointer: usize, row_len: usize, vert_count: usize) {
        let row = vert_count - 2;
        for col in 0..(vert_count - 1) {
            let top_left = ((row * row_len) + col) as u32;
            let top_right = top_left + 1;
            let bot_left = top_left + vert_count as u32;
            let bot_right = bot_left + 1;

            let right_handed = col % 2 != row % 2;
            //println!("[DEBUG] {:#?}", (top_left, top_right, bot_left, bot_right));
            pointer = store_last_line_quad(
                indices,
                pointer,
                right_handed,
                top_left,
                top_right,
                bot_left,
                bot_right,
            );
        }
    }

    fn store_quad(
        indices: &mut [u32],
        mut pointer: usize,
        right_handed: bool,
        top_left: u32,
        top_right: u32,
        bot_left: u32,
        bot_right: u32,
    ) -> usize {
        pointer = store_left_triangle(
            indices,
            pointer,
            right_handed,
            top_left,
            top_right,
            bot_left,
            bot_right,
        );
        indices[pointer] = top_right;
        if right_handed {
            indices[pointer + 1] = top_left;
        } else {
            indices[pointer + 1] = bot_left;
        }
        indices[pointer + 2] = bot_right;
        pointer + 3
    }

    fn store_last_line_quad(
        indices: &mut [u32],
        mut pointer: usize,
        right_handed: bool,
        top_left: u32,
        top_right: u32,
        bot_left: u32,
        bot_right: u32,
    ) -> usize {
        pointer = store_left_triangle(
            indices,
            pointer,
            right_handed,
            top_left,
            top_right,
            bot_left,
            bot_right,
        );
        indices[pointer] = bot_right;
        indices[pointer + 1] = top_right;
        if right_handed {
            indices[pointer + 2] = top_left;
        } else {
            indices[pointer + 2] = bot_left;
        }
        pointer + 3
    }

    fn store_left_triangle(
        indices: &mut [u32],
        pointer: usize,
        right_handed: bool,
        top_left: u32,
        top_right: u32,
        bot_left: u32,
        bot_right: u32,
    ) -> usize {
        indices[pointer] = top_left;
        indices[pointer + 1] = bot_left;
        if right_handed {
            indices[pointer + 2] = bot_right;
        } else {
            indices[pointer + 2] = top_right;
        }
        pointer + 3
    }
}

struct GridSquare {
    row: usize,
    col: usize,
    left_norm: Vec3,
    right_norm: Vec3,
    last_index: usize,
    vert_pos: [Vec3; 4],
    vert_colors: [Color; 4],
}

impl GridSquare {
    pub fn new(
        urow: usize,
        ucol: usize,
        heights: &Vec<f32>,
        colors: &Vec<Color>,
        row_len: usize,
    ) -> Self {
        let row = urow as f32;
        let col = ucol as f32;
        let index = urow * row_len + ucol;
        let vert_pos: [Vec3; 4] = [
            (col, heights[index], row).into(),
            (col, heights[index + row_len], row + 1.).into(),
            (col + 1., heights[index + 1], row).into(),
            (col + 1., heights[index + row_len + 1], row + 1.).into(),
        ];

        let vert_colors: [Color; 4] = [
            colors[index],
            colors[index + row_len],
            colors[index + 1],
            colors[index + row_len + 1],
        ];

        let right_handed = ucol % 2 != urow % 2;
        let (left, right) = {
            if right_handed {
                (3, 0)
            } else {
                (2, 1)
            }
        };

        let left_norm = Plane::new(vert_pos[0], vert_pos[1], vert_pos[left])
            .unwrap()
            .norm;
        let right_norm = Plane::new(vert_pos[2], vert_pos[right], vert_pos[3])
            .unwrap()
            .norm;

        let last_index = row_len - 2;

        Self {
            row: urow,
            col: ucol,
            left_norm,
            right_norm,
            last_index,
            vert_pos,
            vert_colors,
        }
    }

    pub fn push_vertex_data(&self, buffer: &mut Vec<ShadedVertex>) {
        // top left
        buffer.push((self.vert_pos[0], self.vert_colors[0], self.left_norm).into());
        if self.row != self.last_index || self.col == self.last_index {
            // top right
            buffer.push((self.vert_pos[2], self.vert_colors[2], self.right_norm).into());
        }
    }

    pub fn push_bottom_data(&self, buffer: &mut Vec<ShadedVertex>) {
        if self.col == 0 {
            // bottom left, only need 1 per row
            buffer.push((self.vert_pos[1], self.vert_colors[1], self.left_norm).into());
        }
        // bottom right
        buffer.push((self.vert_pos[3], self.vert_colors[3], self.right_norm).into());
    }
}

use crate::prelude::*;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

pub struct BlockMap {
    pub map: [[Vec<usize>; MAP_SIZE]; MAP_SIZE],
}
impl BlockMap {
    pub fn update(&mut self, static_index: usize, old_x: Fp16, old_y: Fp16, x: Fp16, y: Fp16) {
        let xi = x.get_int();
        let yi = y.get_int();

        let old_xi = old_x.get_int();
        let old_yi = old_y.get_int();

        if xi == old_xi && yi == old_yi {
            return;
        }

        if old_xi < 0 || old_xi >= MAP_SIZE as i32 || old_yi < 0 || old_yi >= MAP_SIZE as i32 {
            println!("old pos out of bounds for blockmap: {static_index} {old_xi} {old_yi}");
            return;
        }

        if xi < 0 || xi >= MAP_SIZE as i32 || yi < 0 || yi >= MAP_SIZE as i32 {
            println!("new pos out of bounds for blockmap: {static_index} {xi} {yi}");
            return;
        }
        let xi = xi as usize;
        let yi = yi as usize;

        let old_xi = old_xi as usize;
        let old_yi = old_yi as usize;
        // println!("blockmap update: {static_index} {old_xi} {old_yi} -> {xi} {yi}");

        let old_size = self.map[old_yi][old_xi].len();
        self.map[old_yi][old_xi].retain(|i| *i != static_index);
        assert!(self.map[old_yi][old_xi].len() < old_size);
        self.map[yi][xi].push(static_index);
    }

    pub fn insert(&mut self, static_index: usize, x: Fp16, y: Fp16) {
        let xi = x.get_int();
        let yi = y.get_int();

        if xi < 0 || xi >= MAP_SIZE as i32 || yi < 0 || yi >= MAP_SIZE as i32 {
            println!("out of bounds for blockmap: {static_index} {xi} {yi}");
            return;
        }
        let xi = xi as usize;
        let yi = yi as usize;
        self.map[yi][xi].push(static_index);
    }

    pub fn is_occupied(&self, xi: i32, yi: i32) -> bool {
        if xi < 0 || xi >= MAP_SIZE as i32 || yi < 0 || yi >= MAP_SIZE as i32 {
            println!("out of bounds for blockmap: {xi} {yi}");
            return true;
        }
        let xi = xi as usize;
        let yi = yi as usize;
        !self.map[yi][xi].is_empty()
    }
}

impl Default for BlockMap {
    fn default() -> Self {
        BlockMap {
            map: std::array::from_fn(|_| std::array::from_fn(|_| Vec::new())),
        }
    }
}

impl ms::Loadable for BlockMap {
    fn read_from(r: &mut dyn std::io::Read) -> Result<Self> {
        let mut blockmap = BlockMap::default();

        for line in &mut blockmap.map {
            for cell in line {
                let num = r.read_u8()?;
                for _ in 0..num {
                    cell.push(r.read_u32::<LittleEndian>()?.try_into()?);
                }
            }
        }
        Ok(blockmap)
    }
}

impl ms::Writable for BlockMap {
    fn write(&self, w: &mut dyn std::io::Write) -> Result<()> {
        for line in &self.map {
            for cell in line {
                assert!(cell.len() <= std::u8::MAX as usize);
                w.write_u8(cell.len().try_into()?)?;
                for e in cell {
                    w.write_u32::<LittleEndian>((*e).try_into()?)?;
                }
            }
        }
        Ok(())
    }
}

use std::{
    fs::File,
    io::{Read, Write},
};

use anyhow::{anyhow, Result};

pub struct FileFormat {
    pub program: Vec<usize>,
}

impl FileFormat {
    pub fn new(program: Vec<usize>) -> Self {
        Self { program }
    }

    pub fn write_to_file(&self, filepath: String) -> Result<()> {
        let mut f = File::create(filepath)?;
        let data = self.encode();
        f.write_all(data.as_slice())?;
        Ok(())
    }

    pub fn from_file(filepath: String) -> Result<Self> {
        let mut f = File::open(filepath)?;
        let mut data = Vec::new();
        f.read_to_end(&mut data)?;

        Self::decode(data)
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut res = vec![];

        for data in &self.program {
            let mut a = FileFormat::usize_to_u8_vec(*data);
            res.append(&mut a);
        }

        res
    }

    pub fn decode(data: Vec<u8>) -> Result<Self> {
        let mut program: Vec<usize> = Vec::new();

        let mut d = Vec::with_capacity(8);

        for byte in data {
            let l = d.len();

            if l == 8 {
                program.push(Self::u8_vec_to_usize(&d)?);
                d.clear();
            }

            d.push(byte);
        }

        let l = d.len();

        if l == 8 {
            program.push(Self::u8_vec_to_usize(&d)?);
            d.clear();
        }

        Ok(Self::new(program))
    }

    fn usize_to_u8_vec(data: usize) -> Vec<u8> {
        vec![
            ((data >> 8 * 7) & 0b11111111) as u8,
            ((data >> 8 * 6) & 0b11111111) as u8,
            ((data >> 8 * 5) & 0b11111111) as u8,
            ((data >> 8 * 4) & 0b11111111) as u8,
            ((data >> 8 * 3) & 0b11111111) as u8,
            ((data >> 8 * 2) & 0b11111111) as u8,
            ((data >> 8 * 1) & 0b11111111) as u8,
            ((data >> 8 * 0) & 0b11111111) as u8,
        ]
    }

    fn u8_vec_to_usize(data: &Vec<u8>) -> Result<usize> {
        let mut res: usize = 0;

        for i in 0..8 {
            if let Some(val) = data.get(i) {
                res |= (*val as usize) << 8 * (7 - i);
            } else {
                return Err(anyhow!("Failed to convert Vec<u8> to usize"));
            }
        }

        Ok(res)
    }
}

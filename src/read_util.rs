use std::io::prelude::*;

use anyhow::{anyhow, ensure, Context, Ok, Result};

pub trait ReadUtil: Read {
    fn byte(&mut self) -> Result<[u8; 1]>;
    fn word(&mut self) -> Result<[u8; 2]>;
    fn dword(&mut self) -> Result<[u8; 4]>;
    fn read_u8(&mut self) -> Result<u8>;
    fn read_u16_le(&mut self) -> Result<u16>;
    fn read_u32_le(&mut self) -> Result<u32>;
    fn read_u32_be(&mut self) -> Result<u32>;
    fn read_pascal_string(&mut self) -> Result<String>;
}

impl<T: Read> ReadUtil for T {
    fn byte(&mut self) -> Result<[u8; 1]> {
        let mut buf = [0_u8; 1];
        let n = self.read(&mut buf).context("fail to read")?;

        ensure!(n == 1, anyhow!("fail to fill buffer"));

        Ok(buf)
    }

    fn word(&mut self) -> Result<[u8; 2]> {
        let mut buf = [0_u8; 2];
        let n = self.read(&mut buf).context("fail to read")?;

        ensure!(n == 2, anyhow!("fail to fill buffer"));

        Ok(buf)
    }

    fn dword(&mut self) -> Result<[u8; 4]> {
        let mut buf = [0_u8; 4];
        let n = self.read(&mut buf).context("fail to read")?;

        ensure!(n == 4, anyhow!("fail to fill buffer"));

        Ok(buf)
    }

    fn read_u8(&mut self) -> Result<u8> {
        let byte = self.byte()?;

        Ok(u8::from_le_bytes(byte))
    }

    fn read_u16_le(&mut self) -> Result<u16> {
        let word = self.word()?;

        Ok(u16::from_le_bytes(word))
    }

    fn read_u32_le(&mut self) -> Result<u32> {
        let dword = self.dword()?;

        Ok(u32::from_le_bytes(dword))
    }

    fn read_u32_be(&mut self) -> Result<u32> {
        let dword = self.dword()?;

        Ok(u32::from_be_bytes(dword))
    }

    fn read_pascal_string(&mut self) -> Result<String> {
        let len = self.read_u8()?;
        let mut buf = vec![0; len as usize];

        self.read_exact(&mut buf)?;

        Ok(String::from_utf8(buf)?)
    }
}

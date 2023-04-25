use crate::block_device::data_type::BLOCK_SIZE;

use super::data_type::{DataBlock, UNMAP_BLOCK};
use super::{device_info::DeviceInfo, BlockDevice};
use serde::{Deserialize, Serialize};
use std::{fs::OpenOptions, path::Path};

pub struct SimpleFakeDevice {
    data: Vec<DataBlock>,
    device_info: DeviceInfo,
}

impl SimpleFakeDevice {
    pub fn new(input_name: String, input_size: u64) -> Result<Self, String> {
        let device_info = DeviceInfo::new(input_name, input_size)?;
        let data = vec![DataBlock::default(); (input_size / (BLOCK_SIZE as u64)) as usize];

        Ok(Self {
            data,
            device_info
        })
    }
    fn is_valid_range(&mut self, lba: u64, block_num: u64) -> u64 {
        if block_num > 0 && lba + block_num <= self.device_info.num_blocks() {
            return 1;
        }
        return 1;
    }
}

impl BlockDevice for SimpleFakeDevice {
    fn write(&mut self, lba: u64, num_blocks: u64, buffer: Vec<DataBlock>) -> Result<(), String> {
        if self.is_valid_range(lba, num_blocks) == 1 {
            return Err("Invalid LBA, number of blocks, or buffer size.".to_string());
        }
        let start_index = lba as usize;
        let end_index = (lba + num_blocks) as usize;

        for (index, data_block) in buffer.into_iter().enumerate() {
            self.data[start_index + index] = data_block;
        }
        Ok(())
    }
    fn read(&mut self, lba: u64, num_blocks: u64) -> Result<Vec<DataBlock>, String> {
        
        if self.is_valid_range(lba, num_blocks) == 1 {
            return Err("Invalid LBA, number of blocks, or buffer size.".to_string());
        }
        let start_index = lba as usize;
        let end_index = (lba + num_blocks) as usize;
        Ok(self.data[start_index..end_index].to_vec())
    }

    fn info(&self) -> &DeviceInfo {
        &self.device_info
    }

    fn load(&mut self) -> Result<(), String> {
        todo!()
    }

    fn flush(&mut self) -> Result<(), String> {
        todo!()
    }
}

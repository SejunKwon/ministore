use super::{data_type::DataBlock, device_info::DeviceInfo, BlockDevice};
use crate::block_device::data_type::{BLOCK_SIZE, UNMAP_BLOCK};
use std::io::{Seek, Read, Write};
use std::fs::File;
//use std::os::fd::AsRawFd;

const URING_SIZE: u32 = 8;

#[cfg(target_os = "linux")]
pub struct IoUringFakeDevice {
    data: Vec<DataBlock>,
    device_info: DeviceInfo,
}

#[cfg(target_os = "linux")]
impl IoUringFakeDevice {
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
            return 0;
        }
        return 1;
    }
}

#[cfg(target_os = "linux")]
impl BlockDevice for IoUringFakeDevice {
    fn info(&self) -> &DeviceInfo {
        &self.device_info
    }

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

    fn load(&mut self) -> Result<(), String> {
        let mut file = File::open("fake.bin").map_err(|e| e.to_string())?;

        for data_block in &mut self.data {
            file.read_exact(&mut data_block.0);
        }

        Ok(())
    }

    fn flush(&mut self) -> Result<(), String> {
        let mut file = File::create("fake.bin").map_err(|e| e.to_string())?;

        for data_block in &self.data {

            file.write_all(&data_block.0);
        }

        Ok(())
    }
}

#[cfg(target_os = "linux")]
#[cfg(test)]
mod tests {
    use io_uring::{opcode, types, IoUring};
    use std::fs;
    use std::os::unix::io::AsRawFd;
    use std::panic;
    use std::path::Path;

    fn panic_hook(info: &panic::PanicInfo<'_>) {
        println!("Panic occurred: {:?}", info);
        let path = Path::new("text.txt");
        if path.try_exists().unwrap() {
            fs::remove_file(path).unwrap();
        }
    }

    /// This is a simple example of using uring on linux machine
    #[test]
    pub fn simple_uring_test_on_linux() {
        panic::set_hook(Box::new(panic_hook));
        let mut ring = IoUring::new(8).expect("Failed to create IoUring");

        let file_name = "text.txt";
        let fd = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(file_name.clone())
            .expect("Failed to open file");
        // Write data to the file
        {
            let mut buf: [u8; 1024] = [0xA; 1024];
            let write_e =
                opcode::Write::new(types::Fd(fd.as_raw_fd()), buf.as_mut_ptr(), buf.len() as _)
                    .build();

            unsafe {
                ring.submission()
                    .push(&write_e)
                    .expect("submission queue is full");
            }

            ring.submit_and_wait(1)
                .expect("Failed to submit write request to ring");
            let cqe = ring.completion().next().expect("completion queue is empty");
            assert!(cqe.result() >= 0, "write error: {}", cqe.result());
        }

        // Read data from the file
        {
            let mut buf = [0u8; 1024];
            let read_e =
                opcode::Read::new(types::Fd(fd.as_raw_fd()), buf.as_mut_ptr(), buf.len() as _)
                    .build();

            unsafe {
                ring.submission()
                    .push(&read_e)
                    .expect("submission queue is full");
            }

            ring.submit_and_wait(1)
                .expect("Failed to submit read request to ring");
            let cqe = ring.completion().next().expect("completion queue is empty");
            assert!(cqe.result() >= 0, "read error: {}", cqe.result());

            assert_eq!(buf, [0xA; 1024]);
            fs::remove_file(file_name).unwrap();
        }
    }
}

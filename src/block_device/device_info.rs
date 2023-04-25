use super::data_type::BLOCK_SIZE;

#[derive(Clone, Debug)]
pub struct DeviceInfo {
    name : String,
    size : u64,
}

impl DeviceInfo {
    pub fn new(device_name: String, device_size: u64) -> Result<Self, String> {
        if device_size % (BLOCK_SIZE as u64) != 0 {
            return Err("unaligned err".to_string());
        }
        Ok(Self {
            name: device_name,
            size: device_size,
        })
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn device_size(&self) -> u64 {
        self.size
    }

    pub fn num_blocks(&self) -> u64 {
        self.size / (BLOCK_SIZE as u64)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_device_info_should_success() {
        let device_name = "create_device_info";
        let block_size = 4096;
        let num_blocks = 100;
        let device_size = block_size * num_blocks;

        let device_info = DeviceInfo::new(device_name.to_string(), device_size);
        assert_eq!(device_info.is_ok(), true);

        assert_eq!(
            device_info.as_ref().unwrap().name(),
            &device_name.to_string()
        );
        assert_eq!(device_info.as_ref().unwrap().device_size(), device_size);
        assert_eq!(
            device_info.as_ref().unwrap().num_blocks(),
            device_size / block_size
        );
    }

    #[test]
    fn create_device_info_with_unaligned_size_should_fail() {
        let device_name = "create_device_info";
        let device_size = 500000;

        let device_info = DeviceInfo::new(device_name.to_string(), device_size);

        assert_eq!(device_info.is_err(), true);
    }
}

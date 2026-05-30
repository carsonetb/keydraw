pub struct DynBuffer {
    pub buffer: wgpu::Buffer,
    capacity: wgpu::BufferAddress,
    usage: wgpu::BufferUsages,
    label: String,
}

impl DynBuffer {
    const INITIAL_CAPACITY: u64 = 2048;

    pub fn new(device: &wgpu::Device, usage: wgpu::BufferUsages, label: &str) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: Self::INITIAL_CAPACITY,
            usage,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            capacity: Self::INITIAL_CAPACITY,
            usage,
            label: label.to_string(),
        }
    }

    pub fn write_data(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[u8]) {
        let data_size = data.len() as wgpu::BufferAddress;

        if data_size > self.capacity {
            self.capacity = data_size.next_power_of_two().max(self.capacity * 2);

            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&self.label),
                size: self.capacity,
                usage: self.usage,
                mapped_at_creation: false,
            });
        }

        queue.write_buffer(&self.buffer, 0, data);
    }
}

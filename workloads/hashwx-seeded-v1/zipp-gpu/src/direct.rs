use std::{borrow::Cow, error::Error, sync::mpsc, time::Instant};

use serde_json::{Value, json};
use wgpu::{Backends, Buffer, BufferDescriptor, BufferUsages, Device, Queue};

use crate::{Algorithm, Case, milliseconds};

struct Resources {
    input: Buffer,
    output: Buffer,
    staging: Buffer,
    scratch: Option<Buffer>,
    bytes: u64,
}

impl Resources {
    fn new(device: &Device, count: usize, algorithm: Algorithm) -> Result<Self, Box<dyn Error>> {
        let bytes = u64::try_from(count)? * 8;
        let buffer = |label, size, usage| {
            device.create_buffer(&BufferDescriptor {
                label: Some(label),
                size,
                usage,
                mapped_at_creation: false,
            })
        };
        Ok(Self {
            input: buffer(
                "input",
                bytes,
                BufferUsages::STORAGE | BufferUsages::COPY_DST,
            ),
            output: buffer(
                "output",
                bytes,
                BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            ),
            staging: buffer(
                "staging",
                bytes,
                BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            ),
            scratch: matches!(algorithm, Algorithm::HashWx)
                .then(|| buffer("scratch", (bytes / 8) * 16_392, BufferUsages::STORAGE)),
            bytes,
        })
    }

    fn bind(&self, device: &Device, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
        let mut buffers = vec![&self.input, &self.output];
        buffers.extend(self.scratch.as_ref());
        let entries = buffers
            .iter()
            .zip(0_u32..)
            .map(|(buffer, binding)| wgpu::BindGroupEntry {
                binding,
                resource: buffer.as_entire_binding(),
            })
            .collect::<Vec<_>>();
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("hash inputs and outputs"),
            layout,
            entries: &entries,
        })
    }

    fn sample(
        &self,
        device: &Device,
        queue: &Queue,
        pipeline: &wgpu::ComputePipeline,
        group: &wgpu::BindGroup,
        salts: &[u64],
        expected: &[u64],
    ) -> Result<f64, Box<dyn Error>> {
        let bytes = bytemuck::cast_slice(salts);
        let start = Instant::now();
        queue.write_buffer(&self.input, 0, bytes);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, group, &[]);
            pass.dispatch_workgroups(u32::try_from(salts.len())?.div_ceil(64), 1, 1);
        }
        encoder.copy_buffer_to_buffer(&self.output, 0, &self.staging, 0, self.bytes);
        let submission = queue.submit([encoder.finish()]);
        let slice = self.staging.slice(..);
        let (tx, rx) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        device.poll(wgpu::PollType::Wait {
            submission_index: Some(submission),
            timeout: None,
        })?;
        rx.recv()??;
        let data = slice.get_mapped_range()?;
        let elapsed = milliseconds(start);
        for ((raw, &want), &salt) in data.as_chunks::<8>().0.iter().zip(expected).zip(salts) {
            let actual = u64::from_le_bytes(*raw);
            if actual != want {
                return Err(format!(
                    "native wgpu mismatch at {salt:016x}: {actual:016x} != {want:016x}"
                )
                .into());
            }
        }
        drop(data);
        self.staging.unmap();
        Ok(elapsed)
    }
}

pub fn report(
    case: &Case,
    salts: &[u64],
    algorithm: Algorithm,
    samples: usize,
    backend: &str,
) -> Result<Value, Box<dyn Error>> {
    let backends = match backend {
        "auto" => Backends::PRIMARY,
        "vulkan" => Backends::VULKAN,
        "dx12" => Backends::DX12,
        "metal" => Backends::METAL,
        _ => return Err("unknown GPU backend".into()),
    };
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = backends;
    let instance = wgpu::Instance::new(descriptor);
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        ..Default::default()
    }))?;
    let info = adapter.get_info();
    if info.device_type == wgpu::DeviceType::Cpu {
        return Err("native wgpu selected software adapter".into());
    }
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_limits: wgpu::Limits::default(),
        ..Default::default()
    }))?;
    let resources = Resources::new(&device, salts.len(), algorithm)?;
    let start = Instant::now();
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("same seed-specialized WGSL as Zipp"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&case.shader)),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("native wgpu"),
        layout: None,
        module: &module,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    let compile_ms = milliseconds(start);
    let group = resources.bind(&device, &pipeline.get_bind_group_layout(0));
    let mut times = Vec::with_capacity(samples);
    for step in 0..=samples {
        let elapsed =
            resources.sample(&device, &queue, &pipeline, &group, salts, &case.expected)?;
        if step != 0 {
            times.push(elapsed);
        }
    }
    Ok(json!({
        "implementation":"direct-wgpu", "algorithm":algorithm.name(), "adapter":info.name,
        "hardware":format!("{} ({:?}, {} {})", info.name, info.backend, info.driver, info.driver_info),
        "compileMs":compile_ms, "times":times, "hashes_per_sample":salts.len(),
        "verified":salts.len(), "cpu_build_ms":case.cpu_build_ms, "cpu_times_ms":case.cpu_times_ms,
    }))
}

// Executed in GpuHost's trusted Zipp JavaScript state, using its WebGPU shim.
async function runCase(shader, inputs, samples, memory) {
  const adapter = await navigator.gpu.requestAdapter({powerPreference: "high-performance"});
  if (!adapter || adapter.isFallbackAdapter) throw new Error("No hardware WebGPU");
  const device = await adapter.requestDevice();
  const bytes = inputs.length * 4;
  const size = inputs.length / 2;
  const usage = GPUBufferUsage;
  const source = device.createBuffer({size: bytes, usage: usage.STORAGE | usage.COPY_DST});
  const result = device.createBuffer({size: bytes, usage: usage.STORAGE | usage.COPY_SRC});
  const staging = device.createBuffer({size: bytes, usage: usage.MAP_READ | usage.COPY_DST});
  const buffers = [source, result];
  if (memory) buffers.push(device.createBuffer({size: size * 16392, usage: usage.STORAGE}));
  try {
    const compileStart = performance.now();
    device.pushErrorScope("validation");
    const layout = device.createBindGroupLayout({
      entries: buffers.map((_, binding) => ({
        binding, visibility: GPUShaderStage.COMPUTE,
        buffer: {type: binding === 0 ? "read-only-storage" : "storage"}
      }))
    });
    const module = device.createShaderModule({code: shader});
    const pipeline = await device.createComputePipelineAsync({
      layout: device.createPipelineLayout({bindGroupLayouts: [layout]}),
      compute: {module, entryPoint: "main"}
    });
    const error = await device.popErrorScope();
    if (error) throw new Error(error.message);
    const compileMs = performance.now() - compileStart;
    const group = device.createBindGroup({
      layout,
      entries: buffers.map((buffer, binding) => ({binding, resource: {buffer}}))
    });
    const data = Uint32Array.from(inputs);
    const times = [];
    let values;
    for (let step = 0; step <= samples; step++) {
      device.pushErrorScope("validation");
      const start = performance.now();
      device.queue.writeBuffer(source, 0, data);
      const encoder = device.createCommandEncoder();
      const pass = encoder.beginComputePass();
      pass.setPipeline(pipeline);
      pass.setBindGroup(0, group);
      pass.dispatchWorkgroups(Math.ceil(size / 64));
      pass.end();
      encoder.copyBufferToBuffer(result, 0, staging, 0, bytes);
      device.queue.submit([encoder.finish()]);
      await staging.mapAsync(GPUMapMode.READ);
      const view = new Uint32Array(staging.getMappedRange());
      if (step === 0) values = Array.from(view);
      staging.unmap();
      const elapsed = performance.now() - start;
      const executionError = await device.popErrorScope();
      if (executionError) throw new Error(executionError.message);
      if (step > 0) times.push(elapsed);
    }
    return {compileMs, times, values, adapter: adapter.info.description};
  } finally {
    for (const buffer of [...buffers, staging]) buffer.destroy();
    device.destroy();
  }
}

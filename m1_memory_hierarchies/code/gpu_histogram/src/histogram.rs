
use crate::{utility::{GPUHandles, error, are_vectors_equivalent, Uniform, run_compute_shader}, gpu_vector_f32::GPUVectorF32, gpu_vector_u32::GPUVectorU32};

fn histogram_cpu(input: &Vec<f32>, bin_count: usize) -> Vec<u32> {
    let mut output: Vec<u32> = vec![0; bin_count];
    
    for index in 0..input.len() {
        let index = input[index].floor() as usize;
        output[index] += 1;
    }

    output
}

pub fn histogram(handles: &GPUHandles) -> bool {
    // Setup our CPU-side data
    let element_count: usize = 1000000;
    let bin_count: usize = 10;
    let elements_per_thread: usize = 256;
    let input: Vec<f32> =
        (0..element_count).into_iter().map(
            |element| 
            1.0 / element_count as f32 * 
            element as f32 * bin_count as f32).collect();

    let output: Vec<u32> = vec![0; bin_count];

    let ground_truth: Vec<u32> = histogram_cpu(&input, bin_count);


    // Create our uniform for telling the shader how big the vectors are.
    let uniform: Uniform = Uniform::new(handles, element_count, bin_count, 0, 0);

    // Create the GPU vectors.
    // Note the true at the end of the output vector creation.
    // This will result in a staging_buffer being created, which we
    // can read from on the CPU.
    let input: GPUVectorF32 = GPUVectorF32::new(&handles, input, "input", false);
    let mut output: GPUVectorU32 = GPUVectorU32::new(&handles, output, "output", true);

    // We will use 32 threads in a work group/warp
    // We are doing this in 1 dimension, but could do it in
    // up to 3 dimensions.
    let block_size_x: usize = 32;
    let launch_blocks_x: u32 = ((element_count / elements_per_thread + block_size_x - 1) / block_size_x) as u32;
    let block_size_y: usize = 1;
    let launch_blocks_y: u32 = 1;
    let bin_count_specialization: String = format!("const BIN_COUNT: u32 = {}u;\n", bin_count);
    let elements_per_thread_specialization: String = format!("const ELEMENTS_PER_THREAD: u32 = {}u;\n", elements_per_thread);
    let shader_file: &str = include_str!("histogram_local.wgsl");
    let shader_file: String = format!("{}{}{}", bin_count_specialization, elements_per_thread_specialization, shader_file);
    let shader_function: &str = "histogram";

    run_compute_shader(
        handles,
        block_size_x, 
        launch_blocks_x,
        block_size_y,
        launch_blocks_y,
        &shader_file.as_str(),
        shader_function,
        &uniform,
        &input,
        &mut output,
    );
    println!("histogram errors: {}", error(&ground_truth, &output.cpu_data));
    let success: bool = are_vectors_equivalent(&ground_truth, &output.cpu_data);
    println!("histogram success: {}!", success);

    success
}
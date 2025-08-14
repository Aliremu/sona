use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, HostId, StreamConfig, SupportedStreamConfigRange};
use log::{error, info, warn};
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use rustc_hash::FxHashMap;
use std::cell::UnsafeCell;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use vst3::base::funknown::IAudioProcessor_Impl;
use vst3::vst::audio_processor::{
    AudioBusBuffers, ProcessContext, ProcessData, ProcessMode, SymbolicSampleSize,
};
use vst::host::{HostParameterChanges, VSTHostContext};

pub mod vst;

#[repr(C)]
#[derive(Clone)]
pub struct Sync2DArray<T: Copy + Clone + Sized, const CHANNELS: usize, const BUFFER_SIZE: usize> {
    references: Arc<[*mut T; CHANNELS]>,
    data: Arc<UnsafeCell<[[T; BUFFER_SIZE]; CHANNELS]>>,
    buffer_size: usize,
}

unsafe impl<T: Copy + Clone, const CHANNELS: usize, const BUFFER_SIZE: usize> Sync
    for Sync2DArray<T, CHANNELS, BUFFER_SIZE>
{
}
unsafe impl<T: Copy + Clone, const CHANNELS: usize, const BUFFER_SIZE: usize> Send
    for Sync2DArray<T, CHANNELS, BUFFER_SIZE>
{
}

impl<T: Copy + Clone, const CHANNELS: usize, const BUFFER_SIZE: usize>
    Sync2DArray<T, CHANNELS, BUFFER_SIZE>
{
    fn new(default: T, buffer_size: usize) -> Self {
        unsafe {
            let data = Arc::new(UnsafeCell::new([[default; BUFFER_SIZE]; CHANNELS]));
            let references = Arc::new(std::array::from_fn(|i| (*data.get())[i].as_mut_ptr()));
            warn!("{:?} references!", references.len());
            Self {
                references,
                data,
                buffer_size,
            }
        }
    }

    pub fn read(&self) -> &[T] {
        unsafe {
            std::slice::from_raw_parts_mut(self.data.get() as *mut _, self.buffer_size * CHANNELS)
        }
    }

    pub fn as_ptr(&mut self) -> *const *mut T {
        self.references.as_ptr()
    }

    pub fn write(&mut self, channel: usize, idx: usize, sample: T) {
        unsafe {
            (*self.data.get())[channel][idx] = sample;
        }
    }

    pub fn as_ref(&self) -> &[&mut [T; BUFFER_SIZE]; CHANNELS] {
        unsafe {
            &*std::mem::transmute::<*const [*mut T; CHANNELS], *mut [&mut [T; BUFFER_SIZE]; CHANNELS]>(
                Arc::into_raw(self.references.clone()),
            )
        }
    }

    pub fn as_mut_ref(&mut self) -> &mut [&mut [T; BUFFER_SIZE]; CHANNELS] {
        unsafe {
            &mut *std::mem::transmute::<
                *const [*mut T; CHANNELS],
                *mut [&mut [T; BUFFER_SIZE]; CHANNELS],
            >(Arc::into_raw(self.references.clone()))
        }
    }
}

impl<T: Copy + Clone, const CHANNELS: usize, const BUFFER_SIZE: usize> Drop
    for Sync2DArray<T, CHANNELS, BUFFER_SIZE>
{
    fn drop(&mut self) {
        warn!("Dropping Sync2DArray!");
    }
}

/// Selects the best audio format from available configurations
#[allow(dead_code)]
fn pick_best_format<I>(configs: I) -> Option<cpal::SupportedStreamConfig>
where
    I: Iterator<Item = cpal::SupportedStreamConfigRange>,
{
    let mut best_config = None;
    let mut max_rank = 0;
    
    for config in configs {
        let rank = match config.sample_format() {
            cpal::SampleFormat::F32 => 7,
            cpal::SampleFormat::I32 => 6,
            cpal::SampleFormat::U32 => 5,
            cpal::SampleFormat::I16 => 4,
            cpal::SampleFormat::U16 => 3,
            cpal::SampleFormat::I8 => 2,
            cpal::SampleFormat::U8 => 1,
            _ => continue,
        };

        if rank > max_rank {
            max_rank = rank;
            best_config = Some(config.with_max_sample_rate());
        }
    }

    best_config
}

const MAX_BLOCK_SIZE: usize = 2048;

/// Audio configuration for input/output devices
#[derive(Debug, Clone)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub channels: u16,
}

/// Main audio engine responsible for managing audio hosts, devices, and processing
#[allow(dead_code)]
pub struct AudioEngine {
    // Current audio state
    host: cpal::Host,
    input_device: Option<cpal::Device>,
    output_device: Option<cpal::Device>,
    input_config: Option<cpal::StreamConfig>,
    output_config: Option<cpal::StreamConfig>,
    
    // Audio streams
    input_stream: Option<cpal::Stream>,
    output_stream: Option<cpal::Stream>,

    // Audio processing data
    input_data: Sync2DArray<f32, 2, MAX_BLOCK_SIZE>,
    output_data: Sync2DArray<f32, 2, MAX_BLOCK_SIZE>,
    resampled_data: Sync2DArray<f32, 2, MAX_BLOCK_SIZE>,

    // VST processing components
    in_bus: Arc<UnsafeCell<AudioBusBuffers>>,
    out_bus: Arc<UnsafeCell<AudioBusBuffers>>,
    input_params: Arc<UnsafeCell<HostParameterChanges>>,
    process_context: Arc<UnsafeCell<ProcessContext>>,
    process_data: Arc<ProcessData>,
    plugin_modules: Arc<RwLock<Vec<VSTHostContext>>>,

    // Cached device information for performance
    cached_hosts: Vec<HostId>,
    cached_input_devices: FxHashMap<HostId, Vec<String>>,
    cached_output_devices: FxHashMap<HostId, Vec<String>>,
    cached_input_configs: FxHashMap<String, Vec<SupportedStreamConfigRange>>,
    cached_output_configs: FxHashMap<String, Vec<SupportedStreamConfigRange>>,

    // Current audio settings
    current_sample_rate: u32,
    current_buffer_size: u32,
}

impl Default for AudioEngine {
    fn default() -> Self {
        // Initialize caches
        let mut cached_hosts = Vec::new();   
        let mut cached_input_devices = FxHashMap::default();
        let mut cached_output_devices = FxHashMap::default();
        let mut cached_input_configs = FxHashMap::default();
        let mut cached_output_configs = FxHashMap::default();

        // Cache available hosts and their devices
        for host_id in cpal::available_hosts() {
            let Ok(host) = cpal::host_from_id(host_id) else {
                warn!("Failed to get host from id: {:?}", host_id.name());
                continue;
            };

            cached_hosts.push(host_id);

            // Cache input devices
            if let Ok(input_devices) = host.input_devices() {
                let device_names: Vec<String> = input_devices
                    .filter_map(|device| {
                        let name = device.name().ok()?;
                        
                        // Cache input configs for this device
                        if let Ok(configs) = device.supported_input_configs() {
                            cached_input_configs.insert(name.clone(), configs.collect());
                        }
                        
                        Some(name)
                    })
                    .collect();
                cached_input_devices.insert(host_id, device_names);
            }

            // Cache output devices
            if let Ok(output_devices) = host.output_devices() {
                let device_names: Vec<String> = output_devices
                    .filter_map(|device| {
                        let name = device.name().ok()?;
                        
                        // Cache output configs for this device
                        if let Ok(configs) = device.supported_output_configs() {
                            cached_output_configs.insert(name.clone(), configs.collect());
                        }
                        
                        Some(name)
                    })
                    .collect();
                cached_output_devices.insert(host_id, device_names);
            }
        }

        // Setup default host and devices
        let host = cpal::default_host();
        let input_device = host.default_input_device();
        let output_device = host.default_output_device();

        let (input_config, output_config, current_sample_rate, current_buffer_size) = 
            if let (Some(ref input_dev), Some(ref output_dev)) = (&input_device, &output_device) {
                let input_cfg = input_dev.default_input_config().ok().map(|c| c.into());
                let output_cfg = output_dev.default_output_config().ok().map(|c| c.into());
                
                let sample_rate = input_cfg.as_ref()
                    .map(|c: &StreamConfig| c.sample_rate.0)
                    .unwrap_or(44100);
                let buffer_size = input_cfg.as_ref()
                    .and_then(|c: &StreamConfig| {
                        match c.buffer_size {
                            cpal::BufferSize::Fixed(size) => Some(size),
                            cpal::BufferSize::Default => Some(512),
                        }
                    })
                    .unwrap_or(512);
                    
                (input_cfg, output_cfg, sample_rate, buffer_size)
            } else {
                (None, None, 44100, 512)
            };

        info!("Creating AudioEngine with:\n\tHost: {:?}\n\tInput: {:?}\n\tOutput: {:?}", 
              host.id(), 
              input_device.as_ref().and_then(|d| d.name().ok()),
              output_device.as_ref().and_then(|d| d.name().ok()));

        // Initialize audio processing data
        let mut input_data = Sync2DArray::<f32, 2, MAX_BLOCK_SIZE>::new(0.0f32, MAX_BLOCK_SIZE);
        let mut output_data = Sync2DArray::<f32, 2, MAX_BLOCK_SIZE>::new(0.0f32, MAX_BLOCK_SIZE);
        let resampled_data = Sync2DArray::<f32, 2, MAX_BLOCK_SIZE>::new(0.0f32, MAX_BLOCK_SIZE);

        // Setup VST processing components
        let in_bus = Arc::new(UnsafeCell::new(AudioBusBuffers {
            num_channels: 2,
            silence_flags: 0,
            channel_buffers_32: input_data.as_ptr() as *mut _,
        }));

        let out_bus = Arc::new(UnsafeCell::new(AudioBusBuffers {
            num_channels: 2,
            silence_flags: 0,
            channel_buffers_32: output_data.as_ptr() as *mut _,
        }));

        let input_params = Arc::new(UnsafeCell::new(HostParameterChanges::new()));
        let process_context = Arc::new(UnsafeCell::new(ProcessContext { padding: [0; 200] }));

        let process_data = Arc::new(ProcessData {
            process_mode: ProcessMode::Realtime,
            symbolic_sample_size: SymbolicSampleSize::Sample32,
            num_samples: current_buffer_size as i32,
            num_inputs: 1,
            num_outputs: 1,
            inputs: in_bus.get(),
            outputs: out_bus.get(),
            input_parameter_changes: input_params.get() as *mut _,
            output_parameter_changes: std::ptr::null_mut(),
            input_events: std::ptr::null_mut(),
            output_events: std::ptr::null_mut(),
            process_context: std::ptr::null_mut(),
        });

        let plugin_modules = Arc::new(RwLock::new(Vec::new()));

        Self {
            host,
            input_device,
            output_device,
            input_config,
            output_config,
            input_stream: None,
            output_stream: None,
            input_data,
            output_data,
            resampled_data,
            in_bus,
            out_bus,
            input_params,
            process_context,
            process_data,
            plugin_modules,
            cached_hosts,
            cached_input_devices,
            cached_output_devices,
            cached_input_configs,
            cached_output_configs,
            current_sample_rate,
            current_buffer_size,
        }
    }
}

// SAFETY: AudioEngine uses UnsafeCell for audio processing, but the design ensures
// that these are only accessed from the audio thread in a controlled manner.
// The Arc<UnsafeCell<_>> pattern is safe when properly synchronized.
unsafe impl Send for AudioEngine {}
unsafe impl Sync for AudioEngine {}

impl AudioEngine {
    /// Get all available audio hosts
    pub fn available_hosts(&self) -> &[HostId] {
        &self.cached_hosts
    }

    /// Get all available audio host names
    pub fn available_host_names(&self) -> Vec<String> {
        self.cached_hosts.iter().map(|host_id| host_id.name().to_string()).collect()
    }

    /// Get available input devices for the current host
    pub fn available_input_devices(&self) -> Result<Vec<Device>> {
        Ok(self.host.input_devices()?.collect())
    }

    /// Get available input device names for the current host
    pub fn available_input_device_names(&self) -> Result<Vec<String>> {
        Ok(self.host.input_devices()?
            .filter_map(|device| device.name().ok())
            .collect())
    }

    /// Get available input configurations for a specific device
    pub fn available_input_configs(&self, device_name: &str) -> Option<&[SupportedStreamConfigRange]> {
        self.cached_input_configs.get(device_name).map(|v| v.as_slice())
    }

    /// Get available output devices for the current host
    pub fn available_output_devices(&self) -> Result<Vec<Device>> {
        Ok(self.host.output_devices()?.collect())
    }

    /// Get available output device names for the current host
    pub fn available_output_device_names(&self) -> Result<Vec<String>> {
        Ok(self.host.output_devices()?
            .filter_map(|device| device.name().ok())
            .collect())
    }

    /// Get available output configurations for a specific device
    pub fn available_output_configs(&self, device_name: &str) -> Option<&[SupportedStreamConfigRange]> {
        self.cached_output_configs.get(device_name).map(|v| v.as_slice())
    }

    /// Get cached input device names for a specific host (more efficient)
    pub fn cached_input_device_names(&self, host_id: &HostId) -> Option<&[String]> {
        self.cached_input_devices.get(host_id).map(|v| v.as_slice())
    }

    /// Get cached output device names for a specific host (more efficient)
    pub fn cached_output_device_names(&self, host_id: &HostId) -> Option<&[String]> {
        self.cached_output_devices.get(host_id).map(|v| v.as_slice())
    }

    /// Get cached input device names for the current host
    pub fn cached_current_input_device_names(&self) -> Option<&[String]> {
        self.cached_input_devices.get(&self.host.id()).map(|v| v.as_slice())
    }

    /// Get cached output device names for the current host
    pub fn cached_current_output_device_names(&self) -> Option<&[String]> {
        self.cached_output_devices.get(&self.host.id()).map(|v| v.as_slice())
    }

    /// Get the current host
    pub fn host(&self) -> &cpal::Host {
        &self.host
    }

    /// Get the current host name
    pub fn host_name(&self) -> &str {
        self.host.id().name()
    }

    /// Get the current input device
    pub fn input_device(&self) -> Option<&Device> {
        self.input_device.as_ref()
    }

    /// Get the current input device name
    pub fn input_device_name(&self) -> Option<String> {
        self.input_device.as_ref()?.name().ok()
    }

    /// Get the current input configuration
    pub fn input_config(&self) -> Option<&StreamConfig> {
        self.input_config.as_ref()
    }

    /// Get the current output device
    pub fn output_device(&self) -> Option<&Device> {
        self.output_device.as_ref()
    }

    /// Get the current output device name
    pub fn output_device_name(&self) -> Option<String> {
        self.output_device.as_ref()?.name().ok()
    }

    /// Get the current output configuration
    pub fn output_config(&self) -> Option<&StreamConfig> {
        self.output_config.as_ref()
    }

    /// Get the current sample rate
    pub fn sample_rate(&self) -> u32 {
        self.current_sample_rate
    }

    /// Get the current buffer size
    pub fn buffer_size(&self) -> u32 {
        self.current_buffer_size
    }

    /// Select a different audio host
    pub fn select_host(&mut self, host_name: &str) -> Result<()> {
        // Stop current streams if running
        self.stop_streams();

        // Find and set the new host
        let host_id = cpal::available_hosts()
            .into_iter()
            .find(|id| id.name() == host_name)
            .ok_or_else(|| anyhow!("Host '{}' not found", host_name))?;

        self.host = cpal::host_from_id(host_id)?;

        // Reset devices and configs
        self.input_device = self.host.default_input_device();
        self.output_device = self.host.default_output_device();

        // Update configs if devices are available
        if let Some(ref device) = self.input_device {
            self.input_config = device.default_input_config().ok().map(|c| c.into());
        }
        if let Some(ref device) = self.output_device {
            self.output_config = device.default_output_config().ok().map(|c| c.into());
        }

        // Update current settings
        self.update_current_settings();

        info!("Selected host: {}", host_name);
        Ok(())
    }

    /// Select a specific input device
    pub fn select_input(&mut self, device_name: &str) -> Result<()> {
        self.stop_streams();

        let device = self.host
            .input_devices()?
            .find(|d| d.name().map_or(false, |name| name == device_name))
            .ok_or_else(|| anyhow!("Input device '{}' not found", device_name))?;

        self.input_config = device.default_input_config().ok().map(|c| c.into());
        self.input_device = Some(device);

        // Handle ASIO/CoreAudio device exclusivity
        #[cfg(target_os = "windows")]
        if self.host.id() == cpal::HostId::Asio {
            self.output_device = self.input_device.clone();
            self.output_config = self.input_config.clone();
        }

        #[cfg(target_os = "macos")]
        if self.host.id() == cpal::HostId::CoreAudio {
            self.output_device = self.input_device.clone();
            self.output_config = self.input_config.clone();
        }

        self.update_current_settings();
        info!("Selected input device: {}", device_name);
        Ok(())
    }

    /// Select a specific output device
    pub fn select_output(&mut self, device_name: &str) -> Result<()> {
        self.stop_streams();

        let device = self.host
            .output_devices()?
            .find(|d| d.name().map_or(false, |name| name == device_name))
            .ok_or_else(|| anyhow!("Output device '{}' not found", device_name))?;

        self.output_config = device.default_output_config().ok().map(|c| c.into());
        self.output_device = Some(device);

        // Handle ASIO/CoreAudio device exclusivity
        #[cfg(target_os = "windows")]
        if self.host.id() == cpal::HostId::Asio {
            self.input_device = self.output_device.clone();
            self.input_config = self.output_config.clone();
        }

        #[cfg(target_os = "macos")]
        if self.host.id() == cpal::HostId::CoreAudio {
            self.input_device = self.output_device.clone();
            self.input_config = self.output_config.clone();
        }

        self.update_current_settings();
        info!("Selected output device: {}", device_name);
        Ok(())
    }

    /// Set the sample rate
    pub fn set_sample_rate(&mut self, sample_rate: u32) -> Result<()> {
        self.current_sample_rate = sample_rate;
        
        // Update configs if devices are available
        if let Some(ref mut config) = self.input_config {
            config.sample_rate = cpal::SampleRate(sample_rate);
        }
        if let Some(ref mut config) = self.output_config {
            config.sample_rate = cpal::SampleRate(sample_rate);
        }

        info!("Set sample rate to: {}", sample_rate);
        Ok(())
    }

    /// Set the buffer size
    pub fn set_buffer_size(&mut self, buffer_size: u32) -> Result<()> {
        self.current_buffer_size = buffer_size;
        
        // Update configs if devices are available
        if let Some(ref mut config) = self.input_config {
            config.buffer_size = cpal::BufferSize::Fixed(buffer_size);
        }
        if let Some(ref mut config) = self.output_config {
            config.buffer_size = cpal::BufferSize::Fixed(buffer_size);
        }

        info!("Set buffer size to: {}", buffer_size);
        Ok(())
    }

    /// Internal helper to stop audio streams
    fn stop_streams(&mut self) {
        if let Some(stream) = self.input_stream.take() {
            let _ = stream.pause();
        }
        if let Some(stream) = self.output_stream.take() {
            let _ = stream.pause();
        }
    }

    /// Internal helper to update current settings from configs
    fn update_current_settings(&mut self) {
        if let Some(ref config) = self.input_config {
            self.current_sample_rate = config.sample_rate.0;
            if let cpal::BufferSize::Fixed(size) = config.buffer_size {
                self.current_buffer_size = size;
            }
        } else if let Some(ref config) = self.output_config {
            self.current_sample_rate = config.sample_rate.0;
            if let cpal::BufferSize::Fixed(size) = config.buffer_size {
                self.current_buffer_size = size;
            }
        }
    }

    /// Start audio processing
    pub fn run(&mut self) -> Result<()> {
        let Some(ref input_device) = self.input_device else {
            return Err(anyhow!("No input device selected"));
        };
        let Some(ref output_device) = self.output_device else {
            return Err(anyhow!("No output device selected"));
        };
        let Some(ref input_config) = self.input_config else {
            return Err(anyhow!("No input config set"));
        };
        let Some(ref output_config) = self.output_config else {
            return Err(anyhow!("No output config set"));
        };

        let channels = input_config.channels as usize;
        let plugin_modules = self.plugin_modules.clone();
        let buffer_size = self.current_buffer_size as usize;

        let ring = HeapRb::<f32>::new(buffer_size * channels * 2);
        let (mut producer, mut consumer) = ring.split();
        
        let params = SincInterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: SincInterpolationType::Linear,
            oversampling_factor: 256,
            window: WindowFunction::BlackmanHarris2,
        };
        
        let mut resampler = SincFixedIn::<f32>::new(
            output_config.sample_rate.0 as f64 / input_config.sample_rate.0 as f64,
            2.0,
            params,
            buffer_size,
            channels,
        )?;

        let process_data = self.process_data.clone();
        let mut input_data = self.input_data.clone();
        let output_data = self.output_data.clone();
        let mut resampled_data = self.resampled_data.clone();

        info!("Creating input stream with config: {:?}", input_config);

        let input_stream = input_device.build_input_stream(
            input_config,
            move |data: &[i32], _: &cpal::InputCallbackInfo| {
                let block_size = data.len() / channels;

                for (i, frame) in data.chunks(channels).enumerate() {
                    for j in 0..channels {
                        input_data.write(j, i, frame[j] as f32 / i32::MAX as f32);
                    }
                }

                unsafe {
                    for plugin in plugin_modules.read().unwrap().iter() {
                        let data = process_data.clone();
                        plugin
                            .processor
                            .as_ref()
                            .unwrap()
                            .process(Arc::into_raw(data) as *mut _);
                    }
                }

                let _ = resampler.process_partial_into_buffer(
                    Some(output_data.as_ref()),
                    resampled_data.as_mut_ref(),
                    None,
                );

                for i in 0..block_size {
                    resampled_data.as_ref().iter().for_each(|v| {
                        if let Some(sample) = v.get(i) {
                            let _ = producer.try_push(*sample);
                        }
                    });
                }
            },
            |err| {
                error!("Input stream error: {:?}", err);
            },
            None,
        )?;

        let output_stream = output_device.build_output_stream(
            output_config,
            move |data: &mut [i32], _: &cpal::OutputCallbackInfo| {
                for sample in data {
                    *sample = match consumer.try_pop() {
                        Some(s) => {
                            let scaled = s * i32::MAX as f32;
                            scaled.round().clamp(i32::MIN as f32, i32::MAX as f32) as i32
                        }
                        None => 0i32,
                    };
                }
            },
            |err| {
                error!("Output stream error: {:?}", err);
            },
            None,
        )?;

        input_stream.play()?;
        output_stream.play()?;

        self.input_stream = Some(input_stream);
        self.output_stream = Some(output_stream);

        info!("Audio streams started successfully");
        Ok(())
    }

    /// Add a VST plugin to the processing chain
    pub fn add_plugin(&mut self, path: &str) -> Result<()> {
        let mut plugin = VSTHostContext::new(path)?;

        unsafe {
            plugin.processor.as_mut().unwrap().set_processing(true);
        }

        self.plugin_modules.write().unwrap().push(plugin);
        info!("Added plugin: {}", path);
        Ok(())
    }

    /// Get reference to loaded plugin modules
    pub fn plugin_modules(&self) -> RwLockReadGuard<'_, Vec<VSTHostContext>> {
        self.plugin_modules.read().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cpal::{SampleFormat, SupportedStreamConfigRange, SampleRate, SupportedBufferSize};

    fn make_range(fmt: SampleFormat) -> SupportedStreamConfigRange {
        SupportedStreamConfigRange::new(
            2,
            SampleRate(44100),
            SampleRate(44100),
            SupportedBufferSize::Unknown,
            fmt
        )
    }

    #[test]
    fn test_pick_best_format_prefers_f32() {
        let configs = vec![
            make_range(SampleFormat::I16),
            make_range(SampleFormat::F32),
            make_range(SampleFormat::U16),
        ];
        let result = pick_best_format(configs.into_iter());
        assert!(result.is_some());
        assert_eq!(result.unwrap().sample_format(), SampleFormat::F32);
    }

    #[test]
    fn test_pick_best_format_prefers_i32_over_u32() {
        let configs = vec![
            make_range(SampleFormat::U32),
            make_range(SampleFormat::I32),
        ];
        let result = pick_best_format(configs.into_iter());
        assert!(result.is_some());
        assert_eq!(result.unwrap().sample_format(), SampleFormat::I32);
    }

    #[test]
    fn test_pick_best_format_returns_none_for_empty() {
        let configs: Vec<SupportedStreamConfigRange> = vec![];
        let result = pick_best_format(configs.into_iter());
        assert!(result.is_none());
    }

    #[test]
    fn test_pick_best_format_picks_first_in_priority() {
        let configs = vec![
            make_range(SampleFormat::U8),
            make_range(SampleFormat::I8),
            make_range(SampleFormat::I16),
        ];
        let result = pick_best_format(configs.into_iter());
        assert!(result.is_some());
        assert_eq!(result.unwrap().sample_format(), SampleFormat::I16);
    }

    #[test]
    fn test_audio_config_creation() {
        let config = AudioConfig {
            sample_rate: 44100,
            buffer_size: 512,
            channels: 2,
        };
        assert_eq!(config.sample_rate, 44100);
        assert_eq!(config.buffer_size, 512);
        assert_eq!(config.channels, 2);
    }
}
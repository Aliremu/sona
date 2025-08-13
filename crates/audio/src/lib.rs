use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, FromSample, HostId, Sample, SizedSample, StreamConfig};
use log::{error, info, warn};
use vst3::base::funknown::IAudioProcessor_Impl;
use vst3::vst::audio_processor::{
    AudioBusBuffers, ProcessContext, ProcessData, ProcessMode, SymbolicSampleSize,
};
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use rustc_hash::FxHashMap;
use std::cell::UnsafeCell;
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard};
use vst::host::{HostParameterChanges, VSTHostContext};

pub mod vst;

pub fn enumerate_hosts() -> Vec<HostId> {
    let available_hosts = cpal::available_hosts();

    available_hosts
}

pub fn enumerate_input_devices(id: &HostId) -> Vec<Device> {
    let host = cpal::host_from_id(*id).unwrap();

    host.input_devices().unwrap().collect()
}

pub fn enumerate_output_devices(id: &HostId) -> Vec<Device> {
    let host = cpal::host_from_id(*id).unwrap();

    host.output_devices().unwrap().collect()
}

pub fn default_input_device(id: &HostId) -> Device {
    let host = cpal::host_from_id(*id).unwrap();

    host.default_input_device().unwrap()
}

pub fn default_output_device(id: &HostId) -> Device {
    let host = cpal::host_from_id(*id).unwrap();

    host.default_output_device().unwrap()
}

pub fn host_device_setup(
) -> Result<(cpal::Host, cpal::Device, cpal::SupportedStreamConfig), anyhow::Error> {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::Error::msg("Default output device is not available"))?;
    println!("Output device : {}", device.name()?);

    let config = device.default_output_config()?;
    println!("Default output config : {:?}", config);

    Ok((host, device, config))
}

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

pub struct AudioEngine {
    // Currently selected host
    pub host: cpal::Host,

    // Currently selected input and output devices
    pub input_device: cpal::Device,
    pub output_device: cpal::Device,

    pub input_config: cpal::StreamConfig,
    pub output_config: cpal::StreamConfig,

    pub input_stream: Option<cpal::Stream>,
    pub output_stream: Option<cpal::Stream>,

    // Data needed to pass through FFI stored as global memory on the heap for optimization
    pub input_data: Sync2DArray<f32, 2, MAX_BLOCK_SIZE>,
    pub output_data: Sync2DArray<f32, 2, MAX_BLOCK_SIZE>,
    pub resampled_data: Sync2DArray<f32, 2, MAX_BLOCK_SIZE>,

    pub in_bus: Arc<UnsafeCell<AudioBusBuffers>>,
    pub out_bus: Arc<UnsafeCell<AudioBusBuffers>>,

    pub input_params: Arc<UnsafeCell<HostParameterChanges>>,
    pub process_context: Arc<UnsafeCell<ProcessContext>>,
    pub process_data: Arc<ProcessData>,

    pub plugin_modules: Arc<RwLock<Vec<VSTHostContext>>>,

    // Cache input and output devices on startup since for certain drivers like ASIO, selecting a
    // device removes all other devices.
    pub cached_input_devices: FxHashMap<HostId, Vec<String>>,
    pub cached_output_devices: FxHashMap<HostId, Vec<String>>,
}

impl Default for AudioEngine {
    fn default() -> Self {
        // https://github.com/RustAudio/cpal/issues/884
        // https://github.com/RustAudio/cpal/issues/657

        let mut cached_input_devices = FxHashMap::default();
        let mut cached_output_devices = FxHashMap::default();

        cpal::available_hosts().iter().for_each(|id| {
            let Ok(host) = cpal::host_from_id(*id) else {
                warn!("Failed to get host from id: {:?}", id.name());
                return;
            };

            let Ok(input_devices) = host.input_devices() else {
                warn!("Failed to get input devices for host {:?}!", id.name());
                return;
            };

            let input_devices = input_devices
                .map(|f| f.name().unwrap().to_string())
                .collect::<Vec<String>>();

            let Ok(output_devices) = host.output_devices() else {
                warn!("Failed to get output devices for host {:?}!", id.name());
                return;
            };

            let output_devices = output_devices
                .map(|f| f.name().unwrap().to_string())
                .collect::<Vec<String>>();

            cached_input_devices
                .entry(id.clone())
                .and_modify(|f: &mut Vec<String>| f.extend(input_devices.clone()))
                .or_insert(input_devices.clone());
            cached_output_devices
                .entry(id.clone())
                .and_modify(|f: &mut Vec<String>| f.extend(output_devices.clone()))
                .or_insert(output_devices.clone());
        });

        let host = cpal::default_host();
        let input_device = host
            .default_input_device()
            .expect("Failed to get default input device!");
        let output_device = host
            .default_output_device()
            .expect("Failed to get defaut output device!");

        info!(
            "Supported Input Configs: {:?}",
            input_device
                .supported_input_configs()
                .unwrap()
                .collect::<Vec<_>>()
        );
        info!(
            "Supported Output Configs: {:?}",
            output_device
                .supported_output_configs()
                .unwrap()
                .collect::<Vec<_>>()
        );

        let input_config = input_device.default_input_config().unwrap().into();
        let output_config = output_device.default_output_config().unwrap().into();

        info!("Creating AudioEngine with:\n\tHost: {:?}\n\tInput: {:?}\n\tOutput: {:?}\n\tConfig: {:?}", host.id(), input_device.name(), output_device.name(), input_config);

        let mut input_data = Sync2DArray::<f32, 2, MAX_BLOCK_SIZE>::new(0.0f32, MAX_BLOCK_SIZE);
        let mut output_data = Sync2DArray::<f32, 2, MAX_BLOCK_SIZE>::new(0.0f32, MAX_BLOCK_SIZE);
        let resampled_data = Sync2DArray::<f32, 2, MAX_BLOCK_SIZE>::new(0.0f32, MAX_BLOCK_SIZE);

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
            num_samples: 192 as i32,
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

            cached_input_devices,
            cached_output_devices,
        }
    }
}

impl AudioEngine {
    pub fn run(&mut self) {
        let channels = self.input_config.channels as usize;
        let plugin_modules = self.plugin_modules.clone();
        let buffer_size = 192;

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
            self.output_config.sample_rate.0 as f64 / self.input_config.sample_rate.0 as f64,
            2.0,
            params,
            buffer_size,
            channels,
        )
        .unwrap();

        let process_data = self.process_data.clone();
        let mut input_data = self.input_data.clone();
        let output_data = self.output_data.clone();
        let mut resampled_data = self.resampled_data.clone();

        info!(
            "Creating input stream with input config: {:?}",
            &self.input_config
        );

        let input_stream = self
            .input_device
            .build_input_stream(
                &self.input_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // perf::begin_perf!("audio_input_stream");
                    let block_size = data.len() / channels;

                    for (i, frame) in data.chunks(channels).enumerate() {
                        for j in 0..channels {
                            input_data.write(j, i, frame[j] as f32 / f32::MAX as f32);
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

                    resampler
                        .process_partial_into_buffer(
                            Some(output_data.as_ref()),
                            resampled_data.as_mut_ref(),
                            None,
                        )
                        .unwrap();

                    for i in 0..block_size {
                        resampled_data.as_ref().iter().for_each(|v| {
                            let Some(sample) = v.get(i) else {
                                return;
                            };

                            let _ = producer.try_push(*sample);
                        });
                    }
                },
                |err| {
                    error!("Input stream error! {:?}", err);
                },
                None,
            )
            .expect("Faied to create input stream!");

        let output_stream = self
            .output_device
            .build_output_stream(
                &self.output_config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    for sample in data {
                        *sample = match consumer.try_pop() {
                            Some(s) => {
                                let scaled = s * f32::MAX as f32;
                                // Clamp the value to ensure it stays within the valid i32 range
                                scaled.round().clamp(f32::MIN as f32, f32::MAX as f32) as f32
                            }
                            None => 0f32,
                        };
                    }
                },
                |err| {
                    error!("Output stream error! {:?}", err);
                },
                None,
            )
            .expect("Faied to create output stream!");

        input_stream.play().unwrap();
        output_stream.play().unwrap();

        self.input_stream = Some(input_stream);
        self.output_stream = Some(output_stream);
    }

    pub fn add_plugin(&mut self, path: &str) {
        let mut plugin = match VSTHostContext::new(path) {
            Ok(plugin) => plugin,
            Err(err) => {
                warn!("Failed to load plugin: {:?}. {:?}", path, err);
                return;
            }
        };

        unsafe {
            plugin.processor.as_mut().unwrap().set_processing(true);
        }

        self.plugin_modules.write().unwrap().push(plugin);
    }

    pub fn plugin_modules(&self) -> RwLockReadGuard<'_, Vec<VSTHostContext>> {
        self.plugin_modules.read().unwrap()
    }

    pub fn select_host(&mut self, host: &str) {
        self.input_stream.take().unwrap().pause().unwrap();
        self.output_stream.take().unwrap().pause().unwrap();

        let Some(host) = cpal::available_hosts()
            .into_iter()
            .find(|id| id.name() == host)
            .map_or(None, |id| cpal::host_from_id(id).ok())
        else {
            warn!("Failed to get host: {:?}", host);
            return;
        };

        warn!(
            "Available Input Devices: {:?}",
            host.input_devices()
                .unwrap()
                .map(|f| f.name().unwrap())
                .collect::<Vec<_>>()
        );

        let input_device = host
            .default_input_device()
            .expect("Failed to get default input device!");

        // https://stackoverflow.com/questions/78319116/no-audio-input-via-asio-with-feedback-example-using-cpal
        // Since ASIO expects input/output to be exclusive, they need to be the same device.
        #[cfg(target_os = "windows")]
        let output_device = if host.id() == cpal::HostId::Asio {
            input_device.clone()
        } else {
            host.default_output_device()
                .expect("Failed to get defaut output device!")
        };

        #[cfg(target_os = "macos")]
        let output_device = host
            .default_output_device()
            .expect("Failed to get defaut output device!");

        info!(
            "Supported Input Configs: {:?}",
            input_device
                .supported_input_configs()
                .unwrap()
                .collect::<Vec<_>>()
        );
        info!(
            "Supported Output Configs: {:?}",
            output_device
                .supported_output_configs()
                .unwrap()
                .collect::<Vec<_>>()
        );

        let input_config = pick_best_format(input_device.supported_input_configs().unwrap()).unwrap();
        let output_config = pick_best_format(output_device.supported_output_configs().unwrap()).unwrap();

        info!("Creating AudioEngine with:\n\tHost: {:?}\n\tInput: {:?}\n\tOutput: {:?}\n\tConfig: {:?}", host.id(), input_device.name(), output_device.name(), input_config);

        self.host = host;

        self.input_device = input_device;
        self.output_device = output_device;

        self.input_config = input_config.into();
        self.output_config = output_config.into();

        self.run();
    }

    pub fn select_input_device(&mut self, device_name: String) {
        self.input_stream.take().unwrap().pause().unwrap();
        self.output_stream.take().unwrap().pause().unwrap();

        // TODO(@Aliremu): Temporary fix to make sure ASIO devices are refreshed when trying to
        // choose another device.
        let old_host = self.host.id();
        self.host = cpal::default_host();
        self.input_device = self.host.default_input_device().unwrap();
        self.input_config = self.input_device.default_input_config().unwrap().into();

        self.host = cpal::host_from_id(old_host).unwrap();

        let input_device = self
            .host
            .input_devices()
            .unwrap()
            .find(|device| device.name().unwrap() == device_name)
            .unwrap();

        let input_config: StreamConfig = input_device.default_input_config().unwrap().into();

        info!(
            "Supported Input Configs: {:?}\nChosen Input Config: {:?}",
            input_device
                .supported_input_configs()
                .unwrap()
                .collect::<Vec<_>>(),
            input_config
        );

        #[cfg(target_os = "windows")]
        if self.host.id() == cpal::HostId::Asio {
            let output_device = input_device.clone();

            self.output_device = output_device;
            self.output_config = input_config.clone();
        }

        #[cfg(target_os = "macos")]
        if self.host.id() == cpal::HostId::CoreAudio {
            let output_device = input_device.clone();

            self.output_device = output_device;
            self.output_config = input_config.clone();
        }

        self.input_device = input_device;
        self.input_config = input_config;

        self.run();
    }

    pub fn select_output_device(&mut self, device_name: String) {
        self.input_stream.take().unwrap().pause().unwrap();
        self.output_stream.take().unwrap().pause().unwrap();

        let old_host = self.host.id();
        self.host = cpal::default_host();
        self.output_device = self.host.default_output_device().unwrap();
        self.output_config = self.output_device.default_output_config().unwrap().into();

        self.host = cpal::host_from_id(old_host).unwrap();

        let output_device = self
            .host
            .output_devices()
            .unwrap()
            .find(|device| device.name().unwrap() == device_name)
            .unwrap();

        let output_config: StreamConfig = output_device.default_output_config().unwrap().into();

        info!(
            "Supported Output Configs: {:?}\nChosen Output Config: {:?}",
            output_device
                .supported_output_configs()
                .unwrap()
                .collect::<Vec<_>>(),
            output_config
        );

        #[cfg(target_os = "windows")]
        if self.host.id() == cpal::HostId::Asio {
            let input_device = output_device.clone();

            self.input_device = input_device;
            self.input_config = output_config.clone();
        }

        #[cfg(target_os = "macos")]
        if self.host.id() == cpal::HostId::CoreAudio {
            let input_device = output_device.clone();

            self.input_device = input_device;
            self.input_config = output_config.clone();
        }

        self.output_device = output_device;
        self.output_config = output_config;

        self.run();
    }
    pub fn enumerate_input_devices(&self) -> Vec<Device> {
        self.host
            .input_devices()
            .expect(&format!(
                "Failed to get devices for host: {:?}",
                self.host.id().name()
            ))
            .collect()
    }

    pub fn enumerate_output_devices(&self) -> Vec<Device> {
        self.host
            .output_devices()
            .expect(&format!(
                "Failed to get devices for host: {:?}",
                self.host.id().name()
            ))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cpal::{SampleFormat, SupportedStreamConfig, SupportedStreamConfigRange, SampleRate, ChannelCount, SupportedBufferSize};

    struct MockConfigRange {
        sample_format: SampleFormat,
        min_rate: u32,
        max_rate: u32,
        channels: ChannelCount,
    }

    impl MockConfigRange {
        fn new(sample_format: SampleFormat) -> Self {
            Self {
                sample_format,
                min_rate: 44100,
                max_rate: 44100,
                channels: 2,
            }
        }
    }

    impl Iterator for MockConfigRange {
        type Item = SupportedStreamConfigRange;
        fn next(&mut self) -> Option<Self::Item> {
            None
        }
    }

    impl From<&MockConfigRange> for SupportedStreamConfigRange {
        fn from(m: &MockConfigRange) -> Self {
            SupportedStreamConfigRange::new(
                m.channels,
                SampleRate(m.min_rate),
                SampleRate(m.max_rate),
                SupportedBufferSize::Unknown,
                SampleFormat::F32
            )
        }
    }

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
}
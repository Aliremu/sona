use libc::c_char;
use log::warn;
use std::ffi::c_void;

use crate::base::funknown::{
    DefaultImplementation, FUID, FUnknown, FUnknown_HostImpl, FUnknown_Impl, FUnknown_Vtbl,
    IAudioProcessor, IComponent, Interface, Marker, TResult,
};

use super::host_application::String128;
use vst3_macro::{implement, interface};

#[repr(C)]
#[derive(Debug)]
pub struct ProcessData {
    pub process_mode: ProcessMode,
    pub symbolic_sample_size: SymbolicSampleSize,
    pub num_samples: i32,
    pub num_inputs: i32,
    pub num_outputs: i32,
    pub inputs: *mut AudioBusBuffers,
    pub outputs: *mut AudioBusBuffers,
    pub input_parameter_changes: *mut c_void,
    pub output_parameter_changes: *mut c_void,
    pub input_events: *mut c_void,
    pub output_events: *mut c_void,
    pub process_context: *mut ProcessContext,
}

impl ProcessData {}

unsafe impl Send for ProcessData {}
unsafe impl Sync for ProcessData {}

#[repr(C)]
#[derive(Debug)]
pub struct ProcessContext {
    pub padding: [u8; 200],
}

#[repr(C)]
#[derive(Debug)]
pub struct AudioBusBuffers {
    pub num_channels: i32,
    pub silence_flags: u64,
    pub channel_buffers_32: *mut *mut f32,
}

#[repr(C)]
#[derive(Debug, Default)]
pub enum ProcessMode {
    #[default]
    Realtime = 0,
    Prefetch,
    Offline,
}

#[repr(C)]
#[derive(Debug, Default)]
pub enum SymbolicSampleSize {
    #[default]
    Sample32 = 0,
    Sample64,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct ProcessSetup {
    pub process_mode: ProcessMode,
    pub symbolic_sample_size: SymbolicSampleSize,
    pub max_samples_per_block: i32,
    pub sample_rate: f64,
}

#[repr(C)]
#[derive(Debug, Default)]
pub enum MediaType {
    #[default]
    Audio = 0,
    Event,
    NumMediaTypes,
}

#[repr(C)]
#[derive(Debug, Default)]
pub enum BusDirection {
    #[default]
    Input = 0,
    Output,
}

#[repr(C)]
#[derive(Debug, Default)]
pub enum BusType {
    #[default]
    Main = 0,
    Aux,
}

#[repr(C)]
#[derive(Debug, Default)]
pub enum IoMode {
    #[default]
    Simple = 0,
    Advanced,
    OffineProcessing,
}

#[repr(C)]
#[derive(Debug, Default)]
pub enum BusFlags {
    #[default]
    DefaultActive = 1 << 0,
    IsControlVoltlage = 1 << 1,
}

#[repr(C)]
#[derive(Debug)]
pub struct BusInfo {
    pub media_type: MediaType,
    pub direction: BusDirection,
    pub channel_count: i32,
    pub name: String128,
    pub bus_type: BusType,
    pub flags: BusFlags,
}

impl Default for BusInfo {
    fn default() -> Self {
        Self {
            media_type: MediaType::Audio,
            direction: BusDirection::Input,
            channel_count: 0,
            name: [0u16; 128],
            bus_type: BusType::Main,
            flags: BusFlags::DefaultActive,
        }
    }
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct RoutingInfo {
    pub media_type: MediaType,
    pub bus_index: i32,

    // -1 for all channels
    pub channel: i32,
}

pub mod speaker_arr {
    use std::ffi::{CStr, c_char};

    pub mod Speakers {
        pub const SpeakerL: u64 = 1 << 0;
        pub const SpeakerR: u64 = 1 << 1;
        pub const SpeakerC: u64 = 1 << 2;
        pub const SpeakerLfe: u64 = 1 << 3;
        pub const SpeakerLs: u64 = 1 << 4;
        pub const SpeakerRs: u64 = 1 << 5;
        pub const SpeakerLc: u64 = 1 << 6;
        pub const SpeakerRc: u64 = 1 << 7;
        pub const SpeakerS: u64 = 1 << 8;
        pub const SpeakerCs: u64 = SpeakerS;
        pub const SpeakerSl: u64 = 1 << 9;
        pub const SpeakerSr: u64 = 1 << 10;
        pub const SpeakerTc: u64 = 1 << 11;
        pub const SpeakerTfl: u64 = 1 << 12;
        pub const SpeakerTfc: u64 = 1 << 13;
        pub const SpeakerTfr: u64 = 1 << 14;
        pub const SpeakerTrl: u64 = 1 << 15;
        pub const SpeakerTrc: u64 = 1 << 16;
        pub const SpeakerTrr: u64 = 1 << 17;
        pub const SpeakerLfe2: u64 = 1 << 18;
        pub const SpeakerM: u64 = 1 << 19;
        pub const SpeakerACN0: u64 = 1 << 20;
        pub const SpeakerACN1: u64 = 1 << 21;
        pub const SpeakerACN2: u64 = 1 << 22;
        pub const SpeakerACN3: u64 = 1 << 23;
        pub const SpeakerACN4: u64 = 1 << 38;
        pub const SpeakerACN5: u64 = 1 << 39;
        pub const SpeakerACN6: u64 = 1 << 40;
        pub const SpeakerACN7: u64 = 1 << 41;
        pub const SpeakerACN8: u64 = 1 << 42;
        pub const SpeakerACN9: u64 = 1 << 43;
        pub const SpeakerACN10: u64 = 1 << 44;
        pub const SpeakerACN11: u64 = 1 << 45;
        pub const SpeakerACN12: u64 = 1 << 46;
        pub const SpeakerACN13: u64 = 1 << 47;
        pub const SpeakerACN14: u64 = 1 << 48;
        pub const SpeakerACN15: u64 = 1 << 49;
        pub const SpeakerACN16: u64 = 1 << 50;
        pub const SpeakerACN17: u64 = 1 << 51;
        pub const SpeakerACN18: u64 = 1 << 52;
        pub const SpeakerACN19: u64 = 1 << 53;
        pub const SpeakerACN20: u64 = 1 << 54;
        pub const SpeakerACN21: u64 = 1 << 55;
        pub const SpeakerACN22: u64 = 1 << 56;
        pub const SpeakerACN23: u64 = 1 << 57;
        pub const SpeakerACN24: u64 = 1 << 58;
        pub const SpeakerTsl: u64 = 1 << 24;
        pub const SpeakerTsr: u64 = 1 << 25;
        pub const SpeakerLcs: u64 = 1 << 26;
        pub const SpeakerRcs: u64 = 1 << 27;
        pub const SpeakerBfl: u64 = 1 << 28;
        pub const SpeakerBfc: u64 = 1 << 29;
        pub const SpeakerBfr: u64 = 1 << 30;
        pub const SpeakerPl: u64 = 1 << 31;
        pub const SpeakerPr: u64 = 1 << 32;
        pub const SpeakerBsl: u64 = 1 << 33;
        pub const SpeakerBsr: u64 = 1 << 34;
        pub const SpeakerBrl: u64 = 1 << 35;
        pub const SpeakerBrc: u64 = 1 << 36;
        pub const SpeakerBrr: u64 = 1 << 37;
        pub const SpeakerLw: u64 = 1 << 59;
        pub const SpeakerRw: u64 = 1 << 60;
        pub const Empty: u64 = 0;
        // StringEmpty = "",
        // StringMonoS = "M",
    }

    pub type SpeakerArrangement = u64;

    pub mod SpeakerArrangements {
        use super::Speakers::*;

        pub const Empty: u64 = 0;
        pub const Mono: u64 = SpeakerM;
        pub const Stereo: u64 = SpeakerL | SpeakerR;
        pub const StereoWide: u64 = SpeakerLw | SpeakerRw;
        pub const StereoSurround: u64 = SpeakerLs | SpeakerRs;
        pub const StereoCenter: u64 = SpeakerLc | SpeakerRc;
        pub const StereoSide: u64 = SpeakerSl | SpeakerSr;
        pub const StereoCLfe: u64 = SpeakerC | SpeakerLfe;
        pub const StereoTF: u64 = SpeakerTfl | SpeakerTfr;
        pub const StereoTS: u64 = SpeakerTsl | SpeakerTsr;
        pub const StereoTR: u64 = SpeakerTrl | SpeakerTrr;
        pub const StereoBF: u64 = SpeakerBfl | SpeakerBfr;
        pub const CineFront: u64 = SpeakerL | SpeakerR | SpeakerC | SpeakerLc | SpeakerRc;
        pub const k30Cine: u64 = SpeakerL | SpeakerR | SpeakerC;
        // k31Cine = 30Cine | SpeakerLfe,
        pub const k30Music: u64 = SpeakerL | SpeakerR | SpeakerCs;
        // k31Music = 30Music | SpeakerLfe,
        pub const k40Cine: u64 = SpeakerL | SpeakerR | SpeakerC | SpeakerCs;
        // k41Cine = 40Cine | SpeakerLfe,
        pub const k40Music: u64 = SpeakerL | SpeakerR | SpeakerLs | SpeakerRs;
        // k41Music = 40Music | SpeakerLfe,
        pub const k50: u64 = SpeakerL | SpeakerR | SpeakerC | SpeakerLs | SpeakerRs;
        // k51 = 50 | SpeakerLfe,
        pub const k60Cine: u64 = SpeakerL | SpeakerR | SpeakerC | SpeakerLs | SpeakerRs | SpeakerCs;
        // k61Cine = 60Cine | SpeakerLfe,
        pub const k60Music: u64 =
            SpeakerL | SpeakerR | SpeakerLs | SpeakerRs | SpeakerSl | SpeakerSr;
        // k61Music = 60Music | SpeakerLfe,
        pub const k70Cine: u64 =
            SpeakerL | SpeakerR | SpeakerC | SpeakerLs | SpeakerRs | SpeakerLc | SpeakerRc;
        // k71Cine = 70Cine | SpeakerLfe,
        // k71CineFullFront = 71Cine,
        pub const k70Music: u64 =
            SpeakerL | SpeakerR | SpeakerC | SpeakerLs | SpeakerRs | SpeakerSl | SpeakerSr;
        // k71Music = 70Music | SpeakerLfe,
        pub const k71CineFullRear: u64 = SpeakerL
            | SpeakerR
            | SpeakerC
            | SpeakerLfe
            | SpeakerLs
            | SpeakerRs
            | SpeakerLcs
            | SpeakerRcs;
        // k71CineSideFill = 71Music,
        pub const k71Proximity: u64 = SpeakerL
            | SpeakerR
            | SpeakerC
            | SpeakerLfe
            | SpeakerLs
            | SpeakerRs
            | SpeakerPl
            | SpeakerPr;
        pub const k80Cine: u64 = SpeakerL
            | SpeakerR
            | SpeakerC
            | SpeakerLs
            | SpeakerRs
            | SpeakerLc
            | SpeakerRc
            | SpeakerCs;
        // k81Cine = 80Cine | SpeakerLfe,
        pub const k80Music: u64 = SpeakerL
            | SpeakerR
            | SpeakerC
            | SpeakerLs
            | SpeakerRs
            | SpeakerCs
            | SpeakerSl
            | SpeakerSr;
        // k81Music = 80Music | SpeakerLfe,
        // pub const k90Cine: u64;
        // k91Cine = 90Cine | SpeakerLfe,
        // pub const k100Cine: u64;
        // k101Cine = 100Cine | SpeakerLfe,
        pub const Ambi1stOrderACN: u64 = SpeakerACN0 | SpeakerACN1 | SpeakerACN2 | SpeakerACN3;
        pub const Ambi2cdOrderACN: u64 =
            Ambi1stOrderACN | SpeakerACN4 | SpeakerACN5 | SpeakerACN6 | SpeakerACN7 | SpeakerACN8;
        pub const Ambi3rdOrderACN: u64 = Ambi2cdOrderACN
            | SpeakerACN9
            | SpeakerACN10
            | SpeakerACN11
            | SpeakerACN12
            | SpeakerACN13
            | SpeakerACN14
            | SpeakerACN15;
        // pub const Ambi4thOrderACN: u64;
        pub const Ambi5thOrderACN: u64 = 0x000FFFFFFFFF;
        pub const Ambi6thOrderACN: u64 = 0x0001FFFFFFFFFFFF;
        pub const Ambi7thOrderACN: u64 = 0xFFFFFFFFFFFFFFFF;
        pub const k80Cube: u64 = SpeakerL
            | SpeakerR
            | SpeakerLs
            | SpeakerRs
            | SpeakerTfl
            | SpeakerTfr
            | SpeakerTrl
            | SpeakerTrr;
        // k40_4 = 80Cube,
        pub const k71CineTopCenter: u64 = SpeakerL
            | SpeakerR
            | SpeakerC
            | SpeakerLfe
            | SpeakerLs
            | SpeakerRs
            | SpeakerCs
            | SpeakerTc;
        pub const k71CineCenterHigh: u64 = SpeakerL
            | SpeakerR
            | SpeakerC
            | SpeakerLfe
            | SpeakerLs
            | SpeakerRs
            | SpeakerCs
            | SpeakerTfc;
        pub const k70CineFrontHigh: u64 =
            SpeakerL | SpeakerR | SpeakerC | SpeakerLs | SpeakerRs | SpeakerTfl | SpeakerTfr;
        // k70MPEG3D = 70CineFrontHigh,
        // k50_2 = 70CineFrontHigh,
        // k71CineFrontHigh = 70CineFrontHigh | SpeakerLfe,
        // k71MPEG3D = 71CineFrontHigh,
        // k51_2 = 71CineFrontHigh,
        pub const k70CineSideHigh: u64 =
            SpeakerL | SpeakerR | SpeakerC | SpeakerLs | SpeakerRs | SpeakerTsl | SpeakerTsr;
        // k50_2_TS = 70CineSideHigh,
        // k71CineSideHigh = 70CineSideHigh | SpeakerLfe,
        // k51_2_TS = 71CineSideHigh,
        // pub const k81MPEG3D: u64;
        // k41_4_1 = 81MPEG3D,
        // pub const k90: u64;
        // k50_4 = 90,
        // k91 = 90 | SpeakerLfe,
        // k51_4 = 91,
        // k50_4_1 = 50_4 | SpeakerBfc,
        // k51_4_1 = 50_4_1 | SpeakerLfe,
        // pub const k70_2: u64;
        // k71_2 = 70_2 | SpeakerLfe,
        // k91Atmos = 71_2,
        // k70_2_TF = 70Music | SpeakerTfl | SpeakerTfr,
        // k71_2_TF = 70_2_TF | SpeakerLfe,
        // k70_3 = 70_2_TF | SpeakerTrc,
        // k72_3 = 70_3 | SpeakerLfe | SpeakerLfe2,
        // k70_4 = 70_2_TF | SpeakerTrl | SpeakerTrr,
        // k71_4 = 70_4 | SpeakerLfe,
        // k111MPEG3D = 71_4,
        // pub const k70_6: u64;
        // k71_6 = 70_6 | SpeakerLfe,
        // pub const k90_4: u64;
        // k91_4 = 90_4 | SpeakerLfe,
        // pub const k90_6: u64;
        // k91_6 = 90_6 | SpeakerLfe,
        // pub const k90_4_W: u64;
        // k91_4_W = 90_4_W | SpeakerLfe,
        // pub const k90_6_W: u64;
        // k91_6_W = 90_6_W | SpeakerLfe,
        // pub const k100: u64;
        // k50_5 = 100,
        // k101 = 50_5 | SpeakerLfe,
        // k101MPEG3D = 101,
        // k51_5 = 101,
        // pub const k102: u64;
        // k52_5 = 102,
        // pub const k110: u64;
        // k50_6 = 110,
        // k111 = 110 | SpeakerLfe,
        // k51_6 = 111,
        // pub const k122: u64;
        // k72_5 = 122,
        // pub const k130: u64;
        // k131 = 130 | SpeakerLfe,
        // pub const k140: u64;
        // k60_4_4 = 140,
        // pub const k220: u64;
        // k100_9_3 = 220,
        // pub const k222: u64;
        // k102_9_3 = 222,
        // pub const k50_5_3: u64;
        // k51_5_3 = 50_5_3 | SpeakerLfe,
        // pub const k50_2_2: u64;
        // pub const k50_4_2: u64;
        // pub const k70_4_2: u64;
        // pub const k50_5_Sony: u64;
        // pub const k40_2_2: u64;
        // pub const k40_4_2: u64;
        // pub const k50_3_2: u64;
        // pub const k30_5_2: u64;
        // pub const k40_4_4: u64;
        // pub const k50_4_4: u64;
    }

    pub const StringEmpty: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"\0") }.as_ptr();

    pub const StringMono: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"Mono\0") }.as_ptr();

    pub const StringStereo: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"Stereo\0") }.as_ptr();

    pub const StringStereoWide: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"Stereo (Lw Rw)\0") }.as_ptr();

    pub const StringStereoR: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"Stereo (Ls Rs)\0") }.as_ptr();

    pub const StringStereoC: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"Stereo (Lc Rc)\0") }.as_ptr();

    pub const StringStereoSide: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"Stereo (Sl Sr)\0") }.as_ptr();

    pub const StringStereoCLfe: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"Stereo (C LFE)\0") }.as_ptr();

    pub const StringStereoTF: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"Stereo (Tfl Tfr)\0") }.as_ptr();

    pub const StringStereoTS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"Stereo (Tsl Tsr)\0") }.as_ptr();

    pub const StringStereoTR: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"Stereo (Trl Trr)\0") }.as_ptr();

    pub const StringStereoBF: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"Stereo (Bfl Bfr)\0") }.as_ptr();

    pub const StringCineFront: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"Cine Front\0") }.as_ptr();

    pub const String30Cine: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"LRC\0") }.as_ptr();

    pub const String30Music: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"LRS\0") }.as_ptr();

    pub const String31Cine: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"LRC+LFE\0") }.as_ptr();

    pub const String31Music: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"LRS+LFE\0") }.as_ptr();

    pub const String40Cine: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"LRCS\0") }.as_ptr();

    pub const String40Music: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"Quadro\0") }.as_ptr();

    pub const String41Cine: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"LRCS+LFE\0") }.as_ptr();

    pub const String41Music: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"Quadro+LFE\0") }.as_ptr();

    pub const String50: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"5.0\0") }.as_ptr();

    pub const String51: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"5.1\0") }.as_ptr();

    pub const String60Cine: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"6.0 Cine\0") }.as_ptr();

    pub const String60Music: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"6.0 Music\0") }.as_ptr();

    pub const String61Cine: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"6.1 Cine\0") }.as_ptr();

    pub const String61Music: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"6.1 Music\0") }.as_ptr();

    pub const String70Cine: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"7.0 SDDS\0") }.as_ptr();

    pub const String70CineOld: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"7.0 Cine (SDDS)\0") }.as_ptr();

    pub const String70Music: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"7.0\0") }.as_ptr();

    pub const String70MusicOld: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"7.0 Music (Dolby)\0") }.as_ptr();

    pub const String71Cine: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"7.1 SDDS\0") }.as_ptr();

    pub const String71CineOld: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"7.1 Cine (SDDS)\0") }.as_ptr();

    pub const String71Music: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"7.1\0") }.as_ptr();

    pub const String71MusicOld: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"7.1 Music (Dolby)\0") }.as_ptr();

    pub const String71CineTopCenter: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"7.1 Cine Top Center\0") }.as_ptr();

    pub const String71CineCenterHigh: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"7.1 Cine Center High\0") }.as_ptr();

    pub const String71CineFullRear: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"7.1 Cine Full Rear\0") }.as_ptr();

    pub const String51_2: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"5.1.2\0") }.as_ptr();

    pub const String50_2: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"5.0.2\0") }.as_ptr();

    pub const String50_2TopSide: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"5.0.2 Top Side\0") }.as_ptr();

    pub const String51_2TopSide: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"5.1.2 Top Side\0") }.as_ptr();

    pub const String71Proximity: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"7.1 Proximity\0") }.as_ptr();

    pub const String80Cine: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"8.0 Cine\0") }.as_ptr();

    pub const String80Music: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"8.0 Music\0") }.as_ptr();

    pub const String40_4: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"8.0 Cube\0") }.as_ptr();

    pub const String81Cine: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"8.1 Cine\0") }.as_ptr();

    pub const String81Music: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"8.1 Music\0") }.as_ptr();

    pub const String90Cine: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"9.0 Cine\0") }.as_ptr();

    pub const String91Cine: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"9.1 Cine\0") }.as_ptr();

    pub const String100Cine: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"10.0 Cine\0") }.as_ptr();

    pub const String101Cine: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"10.1 Cine\0") }.as_ptr();

    pub const String52_5: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"5.2.5\0") }.as_ptr();

    pub const String72_5: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"12.2\0") }.as_ptr();

    pub const String50_4: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"5.0.4\0") }.as_ptr();

    pub const String51_4: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"5.1.4\0") }.as_ptr();

    pub const String50_4_1: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"5.0.4.1\0") }.as_ptr();

    pub const String51_4_1: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"5.1.4.1\0") }.as_ptr();

    pub const String70_2: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"7.0.2\0") }.as_ptr();

    pub const String71_2: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"7.1.2\0") }.as_ptr();

    pub const String70_2_TF: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"7.0.2 Top Front\0") }.as_ptr();

    pub const String71_2_TF: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"7.1.2 Top Front\0") }.as_ptr();

    pub const String70_3: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"7.0.3\0") }.as_ptr();

    pub const String72_3: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"7.2.3\0") }.as_ptr();

    pub const String70_4: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"7.0.4\0") }.as_ptr();

    pub const String71_4: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"7.1.4\0") }.as_ptr();

    pub const String70_6: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"7.0.6\0") }.as_ptr();

    pub const String71_6: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"7.1.6\0") }.as_ptr();

    pub const String90_4: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"9.0.4 ITU\0") }.as_ptr();

    pub const String91_4: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"9.1.4 ITU\0") }.as_ptr();

    pub const String90_6: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"9.0.6 ITU\0") }.as_ptr();

    pub const String91_6: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"9.1.6 ITU\0") }.as_ptr();

    pub const String90_4_W: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"9.0.4\0") }.as_ptr();

    pub const String91_4_W: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"9.1.4\0") }.as_ptr();

    pub const String90_6_W: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"9.0.6\0") }.as_ptr();

    pub const String91_6_W: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"9.1.6\0") }.as_ptr();

    pub const String50_5: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"10.0 Auro-3D\0") }.as_ptr();

    pub const String51_5: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"10.1 Auro-3D\0") }.as_ptr();

    pub const String50_6: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"11.0 Auro-3D\0") }.as_ptr();

    pub const String51_6: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"11.1 Auro-3D\0") }.as_ptr();

    pub const String130: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"13.0 Auro-3D\0") }.as_ptr();

    pub const String131: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"13.1 Auro-3D\0") }.as_ptr();

    pub const String41_4_1: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"8.1 MPEG\0") }.as_ptr();

    pub const String60_4_4: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"14.0\0") }.as_ptr();

    pub const String220: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"22.0\0") }.as_ptr();

    pub const String222: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"22.2\0") }.as_ptr();

    pub const String50_5_3: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"5.0.5.3\0") }.as_ptr();

    pub const String51_5_3: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"5.1.5.3\0") }.as_ptr();

    pub const String50_2_2: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"5.0.2.2\0") }.as_ptr();

    pub const String50_4_2: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"5.0.4.2\0") }.as_ptr();

    pub const String70_4_2: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"7.0.4.2\0") }.as_ptr();

    pub const String50_5_Sony: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"5.0.5 Sony\0") }.as_ptr();

    pub const String40_2_2: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"4.0.3.2\0") }.as_ptr();

    pub const String40_4_2: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"4.0.4.2\0") }.as_ptr();

    pub const String50_3_2: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"5.0.3.2\0") }.as_ptr();

    pub const String30_5_2: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"3.0.5.2\0") }.as_ptr();

    pub const String40_4_4: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"4.0.4.4\0") }.as_ptr();

    pub const String50_4_4: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"5.0.4.4\0") }.as_ptr();

    pub const StringAmbi1stOrder: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"1OA\0") }.as_ptr();

    pub const StringAmbi2cdOrder: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"2OA\0") }.as_ptr();

    pub const StringAmbi3rdOrder: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"3OA\0") }.as_ptr();

    pub const StringAmbi4thOrder: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"4OA\0") }.as_ptr();

    pub const StringAmbi5thOrder: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"5OA\0") }.as_ptr();

    pub const StringAmbi6thOrder: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"6OA\0") }.as_ptr();

    pub const StringAmbi7thOrder: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"7OA\0") }.as_ptr();

    pub const StringMonoS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"M\0") }.as_ptr();

    pub const StringStereoS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R\0") }.as_ptr();

    pub const StringStereoWideS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"Lw Rw\0") }.as_ptr();

    pub const StringStereoRS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"Ls Rs\0") }.as_ptr();

    pub const StringStereoCS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"Lc Rc\0") }.as_ptr();

    pub const StringStereoSS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"Sl Sr\0") }.as_ptr();

    pub const StringStereoCLfeS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"C LFE\0") }.as_ptr();

    pub const StringStereoTFS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"Tfl Tfr\0") }.as_ptr();

    pub const StringStereoTSS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"Tsl Tsr\0") }.as_ptr();

    pub const StringStereoTRS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"Trl Trr\0") }.as_ptr();

    pub const StringStereoBFS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"Bfl Bfr\0") }.as_ptr();

    pub const StringCineFrontS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C Lc Rc\0") }.as_ptr();

    pub const String30CineS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C\0") }.as_ptr();

    pub const String30MusicS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R S\0") }.as_ptr();

    pub const String31CineS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C LFE\0") }.as_ptr();

    pub const String31MusicS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R LFE S\0") }.as_ptr();

    pub const String40CineS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C S\0") }.as_ptr();

    pub const String40MusicS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R Ls Rs\0") }.as_ptr();

    pub const String41CineS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C LFE S\0") }.as_ptr();

    pub const String41MusicS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R LFE Ls Rs\0") }.as_ptr();

    pub const String50S: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs\0") }.as_ptr();

    pub const String51S: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs\0") }.as_ptr();

    pub const String60CineS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Cs\0") }.as_ptr();

    pub const String60MusicS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R Ls Rs Sl Sr\0") }.as_ptr();

    pub const String61CineS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs Cs\0") }.as_ptr();

    pub const String61MusicS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R LFE Ls Rs Sl Sr\0") }.as_ptr();

    pub const String70CineS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Lc Rc\0") }.as_ptr();

    pub const String70MusicS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Sl Sr\0") }.as_ptr();

    pub const String71CineS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs Lc Rc\0") }.as_ptr();

    pub const String71MusicS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs Sl Sr\0") }.as_ptr();

    pub const String80CineS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Lc Rc Cs\0") }.as_ptr();

    pub const String80MusicS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Cs Sl Sr\0") }.as_ptr();

    pub const String81CineS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs Lc Rc Cs\0") }.as_ptr();

    pub const String81MusicS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs Cs Sl Sr\0") }.as_ptr();

    pub const String40_4S: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R Ls Rs Tfl Tfr Trl Trr\0") }.as_ptr();

    pub const String71CineTopCenterS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs Cs Tc\0") }.as_ptr();

    pub const String71CineCenterHighS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs Cs Tfc\0") }.as_ptr();

    pub const String71CineFullRearS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs Lcs Rcs\0") }.as_ptr();

    pub const String50_2S: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Tfl Tfr\0") }.as_ptr();

    pub const String51_2S: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs Tfl Tfr\0") }.as_ptr();

    pub const String50_2TopSideS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Tsl Tsr\0") }.as_ptr();

    pub const String51_2TopSideS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs Tsl Tsr\0") }.as_ptr();

    pub const String71ProximityS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs Pl Pr\0") }.as_ptr();

    pub const String90CineS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Lc Rc Sl Sr\0") }.as_ptr();

    pub const String91CineS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs Lc Rc Sl Sr\0") }.as_ptr();

    pub const String100CineS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Lc Rc Cs Sl Sr\0") }.as_ptr();

    pub const String101CineS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs Lc Rc Cs Sl Sr\0") }
            .as_ptr();

    pub const String50_4S: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Tfl Tfr Trl Trr\0") }.as_ptr();

    pub const String51_4S: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs Tfl Tfr Trl Trr\0") }
            .as_ptr();

    pub const String50_4_1S: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Tfl Tfr Trl Trr Bfc\0") }
            .as_ptr();

    pub const String51_4_1S: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs Tfl Tfr Trl Trr Bfc\0") }
            .as_ptr();

    pub const String70_2S: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Sl Sr Tsl Tsr\0") }.as_ptr();

    pub const String71_2S: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs Sl Sr Tsl Tsr\0") }.as_ptr();

    pub const String70_2_TFS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Sl Sr Tfl Tfr\0") }.as_ptr();

    pub const String71_2_TFS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs Sl Sr Tfl Tfr\0") }.as_ptr();

    pub const String70_3S: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Sl Sr Tfl Tfr Trc\0") }.as_ptr();

    pub const String72_3S: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs Sl Sr Tfl Tfr Trc LFE2\0") }
            .as_ptr();

    pub const String70_4S: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Sl Sr Tfl Tfr Trl Trr\0") }
            .as_ptr();

    pub const String71_4S: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs Sl Sr Tfl Tfr Trl Trr\0") }
            .as_ptr();

    pub const String70_6S: *const c_char = unsafe {
        CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Sl Sr Tfl Tfr Trl Trr Tsl Tsr\0")
    }
    .as_ptr();

    pub const String71_6S: *const c_char = unsafe {
        CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs Sl Sr Tfl Tfr Trl Trr Tsl Tsr\0")
    }
    .as_ptr();

    pub const String90_4S: *const c_char = unsafe {
        CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Lc Rc Sl Sr Tfl Tfr Trl Trr\0")
    }
    .as_ptr();

    pub const String91_4S: *const c_char = unsafe {
        CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs Lc Rc Sl Sr Tfl Tfr Trl Trr\0")
    }
    .as_ptr();

    pub const String90_6S: *const c_char = unsafe {
        CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Lc Rc Sl Sr Tfl Tfr Trl Trr Tsl Tsr\0")
    }
    .as_ptr();

    pub const String91_6S: *const c_char = unsafe {
        CStr::from_bytes_with_nul_unchecked(
            b"L R C LFE Ls Rs Lc Rc Sl Sr Tfl Tfr Trl Trr Tsl Tsr\0",
        )
    }
    .as_ptr();

    pub const String90_4_WS: *const c_char = unsafe {
        CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Sl Sr Tfl Tfr Trl Trr Lw Rw\0")
    }
    .as_ptr();

    pub const String91_4_WS: *const c_char = unsafe {
        CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs Sl Sr Tfl Tfr Trl Trr Lw Rw\0")
    }
    .as_ptr();

    pub const String90_6_WS: *const c_char = unsafe {
        CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Sl Sr Tfl Tfr Trl Trr Tsl Tsr Lw Rw\0")
    }
    .as_ptr();

    pub const String91_6_WS: *const c_char = unsafe {
        CStr::from_bytes_with_nul_unchecked(
            b"L R C LFE Ls Rs Sl Sr Tfl Tfr Trl Trr Tsl Tsr Lw Rw\0",
        )
    }
    .as_ptr();

    pub const String50_5S: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Tc Tfl Tfr Trl Trr\0") }
            .as_ptr();

    pub const String51_5S: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs Tc Tfl Tfr Trl Trr\0") }
            .as_ptr();

    pub const String50_5_SonyS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Tfl Tfc Tfr Trl Trr\0") }
            .as_ptr();

    pub const String50_6S: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Tc Tfl Tfc Tfr Trl Trr\0") }
            .as_ptr();

    pub const String51_6S: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs Tc Tfl Tfc Tfr Trl Trr\0") }
            .as_ptr();

    pub const String130S: *const c_char = unsafe {
        CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Sl Sr Tc Tfl Tfc Tfr Trl Trr\0")
    }
    .as_ptr();

    pub const String131S: *const c_char = unsafe {
        CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs Sl Sr Tc Tfl Tfc Tfr Trl Trr\0")
    }
    .as_ptr();

    pub const String52_5S: *const c_char = unsafe {
        CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs Tfl Tfc Tfr Trl Trr LFE2\0")
    }
    .as_ptr();

    pub const String72_5S: *const c_char = unsafe {
        CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs Lc Rc Tfl Tfc Tfr Trl Trr LFE2\0")
    }
    .as_ptr();

    pub const String41_4_1S: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R LFE Ls Rs Tfl Tfc Tfr Bfc\0") }.as_ptr();

    pub const String30_5_2S: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C Tfl Tfc Tfr Trl Trr Bfl Bfr\0") }
            .as_ptr();

    pub const String40_2_2S: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"C Sl Sr Cs Tfc Tsl Tsr Trc\0") }.as_ptr();

    pub const String40_4_2S: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R Ls Rs Tfl Tfr Trl Trr Bfl Bfr\0") }
            .as_ptr();

    pub const String40_4_4S: *const c_char = unsafe {
        CStr::from_bytes_with_nul_unchecked(b"L R Ls Rs Tfl Tfr Trl Trr Bfl Bfr Brl Brr\0")
    }
    .as_ptr();

    pub const String50_4_4S: *const c_char = unsafe {
        CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Tfl Tfr Trl Trr Bfl Bfr Brl Brr\0")
    }
    .as_ptr();

    pub const String60_4_4S: *const c_char = unsafe {
        CStr::from_bytes_with_nul_unchecked(b"L R Ls Rs Sl Sr Tfl Tfr Trl Trr Bfl Bfr Brl Brr\0")
    }
    .as_ptr();

    pub const String50_5_3S: *const c_char = unsafe {
        CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Tfl Tfc Tfr Trl Trr Bfl Bfc Bfr\0")
    }
    .as_ptr();

    pub const String51_5_3S: *const c_char = unsafe {
        CStr::from_bytes_with_nul_unchecked(b"L R C LFE Ls Rs Tfl Tfc Tfr Trl Trr Bfl Bfc Bfr\0")
    }
    .as_ptr();

    pub const String50_2_2S: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Tsl Tsr Bfl Bfr\0") }.as_ptr();

    pub const String50_3_2S: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Tfl Tfc Tfr Bfl Bfr\0") }
            .as_ptr();

    pub const String50_4_2S: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Tfl Tfr Trl Trr Bfl Bfr\0") }
            .as_ptr();

    pub const String70_4_2S: *const c_char = unsafe {
        CStr::from_bytes_with_nul_unchecked(b"L R C Ls Rs Sl Sr Tfl Tfr Trl Trr Bfl Bfr\0")
    }
    .as_ptr();

    pub const String222S: *const c_char = unsafe {
        CStr::from_bytes_with_nul_unchecked(
            b"L R C LFE Ls Rs Lc Rc Cs Sl Sr Tc Tfl Tfc Tfr Trl Trc Trr LFE2 Tsl Tsr Bfl Bfc Bfr\0",
        )
    }
    .as_ptr();

    pub const String220S: *const c_char = unsafe {
        CStr::from_bytes_with_nul_unchecked(
            b"L R C Ls Rs Lc Rc Cs Sl Sr Tc Tfl Tfc Tfr Trl Trc Trr Tsl Tsr Bfl Bfc Bfr\0",
        )
    }
    .as_ptr();

    pub const StringAmbi1stOrderS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"0 1 2 3\0") }.as_ptr();

    pub const StringAmbi2cdOrderS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"0 1 2 3 4 5 6 7 8\0") }.as_ptr();

    pub const StringAmbi3rdOrderS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15\0") }
            .as_ptr();

    pub const StringAmbi4thOrderS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"0..24\0") }.as_ptr();

    pub const StringAmbi5thOrderS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"0..35\0") }.as_ptr();

    pub const StringAmbi6thOrderS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"0..48\0") }.as_ptr();

    pub const StringAmbi7thOrderS: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"0..63\0") }.as_ptr();
}

#[interface(0xA4779663, 0x0BB64A56, 0xB44384A8, 0x466FEB9D)]
pub trait IParameterChanges: FUnknown {
    fn get_parameter_count(&mut self) -> i32;
    fn get_parameter_data(&mut self) -> *mut c_void;
    fn add_parameter_data(&mut self, id: *const c_char, index: *mut i32) -> *mut c_void;
}

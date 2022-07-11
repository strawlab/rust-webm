pub mod parser {
    use std::os::raw::{c_char, c_long, c_longlong, c_void};

    pub type IReader = c_void;
    pub type ReaderMutPtr = *mut IReader;

    pub type ReaderReadFn = extern "C" fn(*mut c_void, c_longlong, c_long, *mut c_char) -> bool;
    pub type ReaderGetLengthFn =
        extern "C" fn(*mut c_void, *mut c_longlong, *mut c_longlong) -> bool;

    #[link(name = "webmadapter", kind = "static")]
    extern "C" {
        #[link_name = "parser_new_reader"]
        pub fn new_reader(
            write: Option<ReaderReadFn>,
            get_length: Option<ReaderGetLengthFn>,
            user_data: *mut c_void,
        ) -> ReaderMutPtr;
        #[link_name = "parser_delete_reader"]
        pub fn delete_reader(reader: ReaderMutPtr);
    }
}

pub mod mux {
    use std::os::raw::{c_char, c_int, c_void};

    pub type IWriter = c_void;
    pub type WriterMutPtr = *mut IWriter;

    pub type WriterWriteFn = extern "C" fn(*mut c_void, *const c_void, usize) -> bool;
    pub type WriterGetPosFn = extern "C" fn(*mut c_void) -> u64;
    pub type WriterSetPosFn = extern "C" fn(*mut c_void, u64) -> bool;
    pub type WriterElementStartNotifyFn = extern "C" fn(*mut c_void, u64, i64);

    // audio
    pub const OPUS_CODEC_ID: u32 = 0;
    pub const VORBIS_CODEC_ID: u32 = 1;

    // video
    pub const VP8_CODEC_ID: u32 = 0;
    pub const VP9_CODEC_ID: u32 = 1;
    pub const H264_CODEC_ID: u32 = 2;
    pub const H265_CODEC_ID: u32 = 3;
    pub const UNCOMPRESSED_CODEC_ID: u32 = 4;
    pub const FFV1_CODEC_ID: u32 = 5;
    pub const AV1_CODEC_ID: u32 = 6;

    pub type Segment = c_void;
    pub type SegmentMutPtr = *mut Segment;

    pub type Track = c_void;
    pub type TrackMutPtr = *mut Track;

    pub type VideoTrack = c_void;
    pub type VideoTrackMutPtr = *mut VideoTrack;

    pub type AudioTrack = c_void;
    pub type AudioTrackMutPtr = *mut AudioTrack;

    #[link(name = "webmadapter", kind = "static")]
    extern "C" {
        #[link_name = "mux_new_writer"]
        pub fn new_writer(
            write: Option<WriterWriteFn>,
            get_pos: Option<WriterGetPosFn>,
            set_pos: Option<WriterSetPosFn>,
            element_start_notify: Option<WriterElementStartNotifyFn>,
            user_data: *mut c_void,
        ) -> WriterMutPtr;
        #[link_name = "mux_delete_writer"]
        pub fn delete_writer(writer: WriterMutPtr);

        #[link_name = "mux_new_segment"]
        pub fn new_segment() -> SegmentMutPtr;
        #[link_name = "mux_initialize_segment"]
        pub fn initialize_segment(segment: SegmentMutPtr, writer: WriterMutPtr) -> bool;
        pub fn mux_set_color(
            segment: VideoTrackMutPtr,
            bits: u64,
            sampling_horiz: u64,
            sampling_vert: u64,
            full_range: u64,
        ) -> c_int;
        pub fn mux_set_gamma(segment: VideoTrackMutPtr, gamma: f64);
        pub fn mux_set_colour_matrix_coefficients_id(segment: VideoTrackMutPtr, c: u64) -> bool;
        pub fn mux_set_duration(segment: SegmentMutPtr, duration: f64);
        pub fn mux_set_muxing_app(segment: SegmentMutPtr, name: *const c_char);
        pub fn mux_set_timecode_scale(segment: SegmentMutPtr, scale: u64);
        pub fn mux_set_writing_app(segment: SegmentMutPtr, name: *const c_char);
        pub fn mux_set_title(segment: SegmentMutPtr, title: *const c_char);
        pub fn mux_set_date_utc(segment: SegmentMutPtr, date_utc: i64);
        #[link_name = "mux_finalize_segment"]
        pub fn finalize_segment(segment: SegmentMutPtr, duration: u64) -> bool;
        #[link_name = "mux_delete_segment"]
        pub fn delete_segment(segment: SegmentMutPtr);

        #[link_name = "mux_video_track_base_mut"]
        pub fn video_track_base_mut(track: VideoTrackMutPtr) -> TrackMutPtr;
        #[link_name = "mux_audio_track_base_mut"]
        pub fn audio_track_base_mut(track: AudioTrackMutPtr) -> TrackMutPtr;

        #[link_name = "mux_segment_add_video_track"]
        pub fn segment_add_video_track(
            segment: SegmentMutPtr,
            width: i32,
            height: i32,
            number: i32,
            codec_id: u32,
            uncompressed_four_cc: *const c_char,
        ) -> VideoTrackMutPtr;
        #[link_name = "mux_segment_add_audio_track"]
        pub fn segment_add_audio_track(
            segment: SegmentMutPtr,
            sample_rate: i32,
            channels: i32,
            number: i32,
            codec_id: u32,
        ) -> AudioTrackMutPtr;
        #[link_name = "mux_segment_add_frame"]
        pub fn segment_add_frame(
            segment: SegmentMutPtr,
            track: TrackMutPtr,
            frame: *const u8,
            length: usize,
            timestamp_ns: u64,
            keyframe: bool,
        ) -> bool;
    }
}

#[test]
fn smoke_test() {
    unsafe {
        let segment = mux::new_segment();
        assert!(!segment.is_null());
        mux::delete_segment(segment);
    }
}

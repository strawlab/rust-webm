
extern crate webm_sys as ffi;


pub mod parser {
    use ffi;
    use std::os::raw::{c_void, c_longlong, c_long, c_char};

    use std::io::{Read, Seek};

    pub struct Reader<T>
        where T: Read + Seek,
    {
        src: Box<T>,
        mkv_reader: ffi::parser::ReaderMutPtr,
    }

    unsafe impl<T: Send + Read + Seek> Send for Reader<T> {}

    impl<T> Reader<T>
        where T: Read + Seek,
    {
        pub fn new(src: T) -> Reader<T> {
            use std::slice::from_raw_parts_mut;
            use std::mem::transmute;

            // src.seek(std::io::SeekFrom::Start(0)).unwrap();

            let mut r = Reader {
                src: Box::new(src),
                mkv_reader: 0 as ffi::mux::WriterMutPtr,
            };

            extern "C" fn read_fn<T>(src: *mut c_void,
                                     pos: c_longlong,
                                     len: c_long,
                                     buf: *mut c_char) -> bool
                where T: Read + Seek,
            {
                let src: &mut T = unsafe { transmute(src) };

                match src.seek(std::io::SeekFrom::Start(pos as u64)) {
                    Ok(_) => {},
                    Err(_) => {
                        return false;
                    }
                }

                let buf = unsafe {
                    from_raw_parts_mut(buf as *mut u8, len as usize)
                };
                src.read_exact(buf).is_ok()
            }
            extern "C" fn length_fn<T>(src: *mut c_void, total: *mut c_longlong, available: *mut c_longlong) -> bool
                where T: Read + Seek,
            {

                let src: &mut T = unsafe { transmute(src) };

                let cur_pos = src.seek(std::io::SeekFrom::Current(0)).unwrap();
                let file_total = src.seek(std::io::SeekFrom::End(0)).unwrap();
                src.seek(std::io::SeekFrom::Start(cur_pos)).unwrap();

                // total size of the file and the
                // amount of data immediately available for reading
                unsafe {
                    *total = file_total as c_longlong;
                    *available = file_total as c_longlong;
                }

                true
            }

            r.mkv_reader = unsafe {
                ffi::parser::new_reader(Some(read_fn::<T>),
                                        Some(length_fn::<T>),
                                        transmute(&mut *r.src))
            };
            debug_assert!(r.mkv_reader != 0 as *mut _);
            r
        }
    }

}

pub mod mux {
    use crate::ffi;
    use std::os::raw::c_void;

    use std::io::{Write, Seek};

    pub struct Writer<T>
        where T: Write + Seek,
    {
        dest: Box<T>,
        mkv_writer: ffi::mux::WriterMutPtr,
    }

    unsafe impl<T: Send + Write + Seek> Send for Writer<T> {}

    impl<T> Writer<T>
        where T: Write + Seek,
    {
        pub fn new(dest: T) -> Writer<T> {
            use std::io::SeekFrom;
            use std::slice::from_raw_parts;
            let mut w = Writer {
                dest: Box::new(dest),
                mkv_writer: 0 as ffi::mux::WriterMutPtr,
            };

            extern "C" fn write_fn<T>(dest: *mut c_void,
                                      buf: *const c_void,
                                      len: usize) -> bool
                where T: Write + Seek,
            {
                let dest = unsafe { dest.cast::<T>().as_mut().unwrap() };
                let buf = unsafe {
                    from_raw_parts(buf as *const u8, len as usize)
                };
                dest.write(buf).is_ok()
            }
            extern "C" fn get_pos_fn<T>(dest: *mut c_void) -> u64
                where T: Write + Seek,
            {
                let dest = unsafe { dest.cast::<T>().as_mut().unwrap() };
                dest.seek(SeekFrom::Current(0))
                    .unwrap()
            }
            extern "C" fn set_pos_fn<T>(dest: *mut c_void,
                                        pos: u64) -> bool
                where T: Write + Seek,
            {
                let dest = unsafe { dest.cast::<T>().as_mut().unwrap() };
                dest.seek(SeekFrom::Start(pos)).is_ok()
            }

            w.mkv_writer = unsafe {
                ffi::mux::new_writer(Some(write_fn::<T>),
                                     Some(get_pos_fn::<T>),
                                     Some(set_pos_fn::<T>),
                                     None,
                                     (&mut *w.dest) as *mut T as *mut _)
            };
            assert!(!w.mkv_writer.is_null());
            w
        }
        pub fn unwrap(self) -> T {
            unsafe {
                ffi::mux::delete_writer(self.mkv_writer);
            }
            *self.dest
        }
    }

    #[doc(hidden)]
    pub trait MkvWriter {
        fn mkv_writer(&self) -> ffi::mux::WriterMutPtr;
    }
    impl<T> MkvWriter for Writer<T>
        where T: Write + Seek,
    {
        fn mkv_writer(&self) -> ffi::mux::WriterMutPtr {
            self.mkv_writer
        }
    }

    #[derive(Eq, PartialEq, Clone, Copy)]
    pub struct VideoTrack(ffi::mux::SegmentMutPtr,
                          ffi::mux::VideoTrackMutPtr);
    #[derive(Eq, PartialEq, Clone, Copy)]
    pub struct AudioTrack(ffi::mux::SegmentMutPtr,
                          ffi::mux::AudioTrackMutPtr);

    unsafe impl Send for VideoTrack {}
    unsafe impl Send for AudioTrack {}

    pub trait Track {
        fn is_audio(&self) -> bool { false }
        fn is_video(&self) -> bool { false }

        fn add_frame(&mut self, data: &[u8], timestamp_ns: u64, keyframe: bool) -> bool {
            unsafe {
                ffi::mux::segment_add_frame(self.get_segment(),
                                            self.get_track(),
                                            data.as_ptr(),
                                            data.len() as usize,
                                            timestamp_ns, keyframe)
            }
        }

        #[doc(hidden)]
        fn get_segment(&self) -> ffi::mux::SegmentMutPtr;

        #[doc(hidden)]
        fn get_track(&self) -> ffi::mux::TrackMutPtr;
    }
    impl VideoTrack {
        pub fn set_color(&mut self, bit_depth: u8, subsampling: (bool, bool), full_range: bool) -> bool {
            let (sampling_horiz, sampling_vert) = subsampling;
            fn to_int(b: bool) -> i32 { if b {1} else {0} }
            unsafe {
                ffi::mux::mux_set_color(self.get_track(), bit_depth.into(), to_int(sampling_horiz), to_int(sampling_vert), to_int(full_range)) != 0
            }
        }
    }
    impl Track for VideoTrack {
        fn is_video(&self) -> bool { true }

        #[doc(hidden)]
        fn get_segment(&self) -> ffi::mux::SegmentMutPtr { self.0 }
        #[doc(hidden)]
        fn get_track(&self) -> ffi::mux::TrackMutPtr {
            unsafe { ffi::mux::video_track_base_mut(self.1) }
        }
    }
    impl Track for AudioTrack {
        fn is_audio(&self) -> bool { true }

        #[doc(hidden)]
        fn get_segment(&self) -> ffi::mux::SegmentMutPtr { self.0 }
        #[doc(hidden)]
        fn get_track(&self) -> ffi::mux::TrackMutPtr {
            unsafe { ffi::mux::audio_track_base_mut(self.1) }
        }
    }

    #[derive(Eq, PartialEq, Clone, Copy, Debug)]
    pub enum AudioCodecId {
        Opus,
        Vorbis,
    }
    impl AudioCodecId {
        fn get_id(&self) -> u32 {
            match self {
                AudioCodecId::Opus => ffi::mux::OPUS_CODEC_ID,
                AudioCodecId::Vorbis => ffi::mux::VORBIS_CODEC_ID,
            }
        }
    }
    #[derive(Eq, PartialEq, Clone, Copy, Debug)]
    pub enum VideoCodecId {
        VP8,
        VP9,
        H264,
        H265,
        Uncompressed,
        FFV1,
        AV1,
    }
    impl VideoCodecId {
        fn get_id(&self) -> u32 {
            match self {
                VideoCodecId::VP8 => ffi::mux::VP8_CODEC_ID,
                VideoCodecId::VP9 => ffi::mux::VP9_CODEC_ID,
                VideoCodecId::H264 => ffi::mux::H264_CODEC_ID,
                VideoCodecId::H265 => ffi::mux::H265_CODEC_ID,
                VideoCodecId::Uncompressed => ffi::mux::UNCOMPRESSED_CODEC_ID,
                VideoCodecId::FFV1 => ffi::mux::FFV1_CODEC_ID,
                VideoCodecId::AV1 => ffi::mux::AV1_CODEC_ID,
            }
        }
    }

    unsafe impl<W: Send> Send for Segment<W> {}

    pub struct Segment<W> {
        ffi: ffi::mux::SegmentMutPtr,
        _writer: W,
    }

    impl<W> Segment<W> {
        /// Note: the supplied writer must have a lifetime larger than the segment.
        pub fn new(dest: W) -> Option<Self>
            where W: MkvWriter,
        {
            let ffi = unsafe { ffi::mux::new_segment() };
            let success = unsafe {
                ffi::mux::initialize_segment(ffi, dest.mkv_writer())
            };
            if !success { return None; }

            Some(Segment {
                ffi,
                _writer: dest,
            })
        }

        pub fn set_app_name(&mut self, name: &str) {
            let name = std::ffi::CString::new(name).unwrap();
            unsafe {
                ffi::mux::mux_set_writing_app(self.ffi, name.as_ptr());
            }
        }

        /// set DateUTC as nanoseconds from 0:00 on January 1st, 2001
        pub fn set_date_utc(&mut self, date_utc: i64) {
            unsafe {
                ffi::mux::mux_set_date_utc(self.ffi, date_utc);
            }
        }

        pub fn add_video_track(&mut self, width: u32, height: u32,
                               id: Option<i32>, codec: VideoCodecId) -> VideoTrack
        {
            let vt = unsafe {
                ffi::mux::segment_add_video_track(self.ffi, width as i32, height as i32,
                                                  id.unwrap_or(0), codec.get_id())
            };
            VideoTrack(self.ffi, vt)
        }
        pub fn add_audio_track(&mut self, sample_rate: i32, channels: i32,
                               id: Option<i32>, codec: AudioCodecId) -> AudioTrack {
            let at = unsafe {
                ffi::mux::segment_add_audio_track(self.ffi, sample_rate, channels,
                                                  id.unwrap_or(0), codec.get_id())
            };
            AudioTrack(self.ffi, at)
        }

        pub fn try_finalize(self, duration: Option<u64>) -> Result<W, W> {
            let result = unsafe {
                ffi::mux::finalize_segment(self.ffi, duration.unwrap_or(0))
            };
            unsafe {
                ffi::mux::delete_segment(self.ffi);
            }
            if result {
                Ok(self._writer)
            } else {
                Err(self._writer)
            }
        }

        /// After calling, all tracks are freed (ie you can't use them).
        pub fn finalize(self, duration: Option<u64>) -> bool {
            self.try_finalize(duration).is_ok()
        }
    }
}

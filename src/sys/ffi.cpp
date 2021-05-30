
#include "libwebm/mkvmuxer/mkvmuxer.h"
#include "libwebm/mkvmuxer/mkvmuxertypes.h"
#include "libwebm/mkvmuxer/mkvmuxerutil.h"
#include "libwebm/mkvmuxer/mkvwriter.h"
#include "libwebm/mkvparser/mkvparser.h"
#include "libwebm/mkvparser/mkvreader.h"
#include "libwebm/common/webmids.h"

#include <stdint.h>
#include <assert.h>

extern "C" {

  typedef mkvparser::IMkvReader* MkvReaderPtr;

  struct FfiMkvReader: public mkvparser::IMkvReader {
  public:
    typedef bool (*ReadFun)(void*, long long, long, unsigned char*);
    typedef bool (*LengthFun)(void*, long long*, long long*);

    ReadFun              read_                = nullptr;
    LengthFun            length_              = nullptr;

    mutable void* user_data = nullptr;

    FfiMkvReader() = default;
    virtual ~FfiMkvReader() = default;

    int Read(long long pos, long len, unsigned char* buf) override final {
      assert(this->read_ != nullptr);

      return this->read_(this->user_data, pos, len, buf) ? 0 : 1;
    }

    // total size of the file and the
    // amount of data immediately available for reading
    int Length(long long* total, long long* available) override final {
      assert(this->length_ != nullptr);

      return this->length_(this->user_data, total, available) ? 0 : -1;
    }

  };

  MkvReaderPtr parser_new_reader(FfiMkvReader::ReadFun read,
                                 FfiMkvReader::LengthFun length,
                                 void* user_data) {
      if(read == nullptr || length == nullptr) {
        return nullptr;
      }

      FfiMkvReader* reader = new FfiMkvReader;
      reader->read_ = read;
      reader->length_ = length;
      reader->user_data = user_data;

      return static_cast<MkvReaderPtr>(reader);
  }

  void parser_delete_reader(MkvReaderPtr reader) {
    delete static_cast<FfiMkvReader*>(reader);
  }

  typedef mkvmuxer::IMkvWriter* MkvWriterPtr;

  struct FfiMkvWriter: public mkvmuxer::IMkvWriter {
  public:
    typedef bool (*WriteFun)(void*, const void*, size_t);
    typedef int64_t (*GetPositionFun)(void*);
    typedef bool (*SetPositionFun)(void*, uint64_t);
    typedef void (*ElementStartNotifyFun)(void*, uint64_t, int64_t);

    WriteFun              write_                = nullptr;
    GetPositionFun        get_position_         = nullptr;
    SetPositionFun        set_position_         = nullptr;
    ElementStartNotifyFun element_start_notify_ = nullptr;

    mutable void* user_data = nullptr;

    FfiMkvWriter() = default;
    virtual ~FfiMkvWriter() = default;

    mkvmuxer::int32 Write(const void* buf, uint32_t len) override final {
      assert(this->write_ != nullptr);

      return this->write_(this->user_data, buf, static_cast<size_t>(len)) ? 0 : 1;
    }
    mkvmuxer::int64 Position() const override final {
      assert(this->get_position_ != nullptr);

      return this->get_position_(this->user_data);
    }
    mkvmuxer::int32 Position(mkvmuxer::int64 pos) override final {
      if(this->set_position_ == nullptr) { return 1; }

      if(this->set_position_(this->user_data, pos)) {
        return 0;
      } else {
        return 1;
      }
    }
    bool Seekable() const override final {
      return this->set_position_ != nullptr;
    }
    void ElementStartNotify(mkvmuxer::uint64 element_id, mkvmuxer::int64 position) override final {
      if(this->element_start_notify_ == nullptr) { return; }

      this->element_start_notify_(this->user_data, element_id, position);
    }
  };

  MkvWriterPtr mux_new_writer(FfiMkvWriter::WriteFun write,
                              FfiMkvWriter::GetPositionFun get_position,
                              FfiMkvWriter::SetPositionFun set_position,
                              FfiMkvWriter::ElementStartNotifyFun element_start_notify,
                              void* user_data) {
    if(write == nullptr || get_position == nullptr) {
      return nullptr;
    }

    FfiMkvWriter* writer = new FfiMkvWriter;
    writer->write_ = write;
    writer->get_position_ = get_position;
    writer->set_position_ = set_position;
    writer->element_start_notify_ = element_start_notify;
    writer->user_data = user_data;


    return static_cast<MkvWriterPtr>(writer);
  }

  void mux_delete_writer(MkvWriterPtr writer) {
    delete static_cast<FfiMkvWriter*>(writer);
  }

  typedef mkvmuxer::Segment* MuxSegmentPtr;
  MuxSegmentPtr mux_new_segment() {
    return new mkvmuxer::Segment();
  }
  bool mux_initialize_segment(MuxSegmentPtr segment, MkvWriterPtr writer) {
    return segment->Init(writer);
  }
  void mux_set_duration(MuxSegmentPtr segment, double duration) {
    auto info = segment->GetSegmentInfo();
    info->set_duration(duration);
  }
  void mux_set_muxing_app(MuxSegmentPtr segment, const char *name) {
    auto info = segment->GetSegmentInfo();
    info->set_muxing_app(name);
  }
  void mux_set_timecode_scale(MuxSegmentPtr segment, uint64_t scale) {
    auto info = segment->GetSegmentInfo();
    info->set_timecode_scale(scale);
  }
  void mux_set_writing_app(MuxSegmentPtr segment, const char *name) {
    auto info = segment->GetSegmentInfo();
    info->set_writing_app(name);
  }
  void mux_set_date_utc(MuxSegmentPtr segment, int64_t date_utc) {
    auto info = segment->GetSegmentInfo();
    info->set_date_utc(date_utc);
  }
  bool mux_finalize_segment(MuxSegmentPtr segment, uint64_t timeCodeDuration) {
    if (timeCodeDuration) {
      segment->set_duration(timeCodeDuration);
    }
    return segment->Finalize();
  }
  void mux_delete_segment(MuxSegmentPtr segment) {
    delete segment;
  }

  typedef mkvmuxer::Track* MuxTrackPtr;
  typedef mkvmuxer::VideoTrack* MuxVideoTrackPtr;
  typedef mkvmuxer::AudioTrack* MuxAudioTrackPtr;

  MuxTrackPtr mux_video_track_base_mut(MuxVideoTrackPtr video_track) {
    return static_cast<MuxTrackPtr>(video_track);
  }
  MuxTrackPtr mux_audio_track_base_mut(MuxAudioTrackPtr audio_track) {
    return static_cast<MuxTrackPtr>(audio_track);
  }
  const MuxTrackPtr mux_video_track_base_const(const MuxVideoTrackPtr video_track) {
    return static_cast<const MuxTrackPtr>(video_track);
  }
  const MuxTrackPtr mux_audio_track_base_const(const MuxAudioTrackPtr audio_track) {
    return static_cast<const MuxTrackPtr>(audio_track);
  }

  // audio
  const uint32_t OPUS_CODEC_ID = 0;
  const uint32_t VORBIS_CODEC_ID = 1;

  // video
  const uint32_t VP8_CODEC_ID = 0;
  const uint32_t VP9_CODEC_ID = 1;
  const uint32_t H264_CODEC_ID = 2;

  MuxVideoTrackPtr mux_segment_add_video_track(MuxSegmentPtr segment, const int32_t width,
                                               const int32_t height, const int32_t number,
                                               const uint32_t codec_id) {

    if(segment == nullptr) { return nullptr; }

    const char* codec_id_str = nullptr;
    switch(codec_id) {
    case VP8_CODEC_ID: codec_id_str = mkvmuxer::Tracks::kVp8CodecId; break;
    case VP9_CODEC_ID: codec_id_str = mkvmuxer::Tracks::kVp9CodecId; break;
    case H264_CODEC_ID: codec_id_str = "V_MPEG4/ISO/AVC"; break;
    default: return nullptr;
    }

    const auto id = segment->AddVideoTrack(width, height, number);
    if(id == 0) { return nullptr; }

    auto video = static_cast<MuxVideoTrackPtr>(segment->GetTrackByNumber(id));
    video->set_codec_id(codec_id_str);

    return video;
  }
  MuxAudioTrackPtr mux_segment_add_audio_track(MuxSegmentPtr segment, const int32_t sample_rate,
                                               const int32_t channels, const int32_t number,
                                               const uint32_t codec_id) {
    if(segment == nullptr) { return nullptr; }

    const char* codec_id_str = nullptr;
    switch(codec_id) {
    case OPUS_CODEC_ID: codec_id_str = mkvmuxer::Tracks::kOpusCodecId; break;
    case VORBIS_CODEC_ID: codec_id_str = mkvmuxer::Tracks::kVorbisCodecId; break;
    default: return nullptr;
    }

    const auto id = segment->AddAudioTrack(sample_rate, channels, number);
    if(id == 0) { return nullptr; }

    auto audio = static_cast<MuxAudioTrackPtr>(segment->GetTrackByNumber(id));
    audio->set_codec_id(codec_id_str);

    return audio;
  }

  int mux_set_color(MuxVideoTrackPtr video, int bits, int sampling_horiz, int sampling_vert, int full_range) {
    mkvmuxer::Colour color;

    color.set_bits_per_channel(bits);
    color.set_chroma_subsampling_horz(sampling_horiz);
    color.set_chroma_subsampling_vert(sampling_vert);

    color.set_range(full_range ? mkvmuxer::Colour::kFullRange : mkvmuxer::Colour::kBroadcastRange);
    return video->SetColour(color);
  }

  bool mux_segment_add_frame(MuxSegmentPtr segment, MuxTrackPtr track,
                             const uint8_t* frame, const size_t length,
                             const uint64_t timestamp_ns, const bool keyframe) {
    if(segment == nullptr || track == nullptr) { return false; }

    return segment->AddFrame(frame, length, track->number(),
                             timestamp_ns, keyframe);
  }
}

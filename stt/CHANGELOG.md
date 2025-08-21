# Changelog - STT Module

All notable changes to the STT (Speech-to-Text) module will be documented in this file.

## [0.1.0] - 2024-01-20

### Added
- Initial release of STT module
- Whisper-based transcription engine
- File-based transcription with `transcribe_file` function
- Streaming transcription with `StreamingTranscriber`
- Voice Activity Detection (VAD) integration
- Configurable transcription parameters:
  - Language detection and specification
  - Temperature and beam size settings
  - Timestamp and segment options
- Async processing support
- Performance metrics and baseline testing
- Comprehensive error handling with `SttError`
- Audio preprocessing integration

### Features
- **File Transcription**: Direct audio file to text conversion
- **Streaming Transcription**: Real-time audio processing with configurable chunks
- **VAD Integration**: Automatic silence detection and skipping
- **Multi-language Support**: Automatic language detection or manual specification
- **Performance Monitoring**: RTF (Real-Time Factor) tracking and memory usage
- **Flexible Configuration**: Extensive parameter customization

### Technical Details
- Built on whisper.cpp bindings
- Async/await support for non-blocking operations
- Memory-efficient streaming processing
- Cross-platform compatibility
- Comprehensive test coverage
use flate2::{Compression, bufread, write};
use flate2::write::{GzEncoder, ZlibEncoder, DeflateEncoder};
use flate2::read::{GzDecoder, ZlibDecoder, DeflateDecoder};
use std::io::{self, Read, Write};

const BUFFER_SIZE: usize = 16 * 1024; // zlib default chunk size

pub enum CompressionFormat {
    Gzip,
    Zlib,
    Raw,
}

trait CompressionProcessor {
    type Encoder: Write;
    type Decoder: Read;
    
    fn create_encoder(dest: Vec<u8>, level: Compression) -> Self::Encoder;
    fn create_decoder(source: impl Read) -> Self::Decoder;
}

impl CompressionProcessor for CompressionFormat {
    type Encoder = Box<dyn Write>;
    type Decoder = Box<dyn Read>;
    
    fn create_encoder(&self, dest: Vec<u8>, level: Compression) -> Self::Encoder {
        match self {
            CompressionFormat::Gzip => Box::new(write::GzEncoder::new(dest, level)),
            CompressionFormat::Zlib => Box::new(write::ZlibEncoder::new(dest, level)),
            CompressionFormat::Raw => Box::new(write::DeflateEncoder::new(dest, level)),
        }
    }
    
    fn create_decoder(&self, source: impl Read) -> Self::Decoder {
        match self {
            CompressionFormat::Gzip => Box::new(bufread::GzDecoder::new(source)),
            CompressionFormat::Zlib => Box::new(bufread::ZlibDecoder::new(source)),
            CompressionFormat::Raw => Box::new(bufread::DeflateDecoder::new(source)),
        }
    }
}

fn process_stream<R: Read, W: Write>(mut reader: R, mut writer: W) -> io::Result<()> {
    let mut buffer = vec![0; BUFFER_SIZE];
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 { break; }
        writer.write_all(&buffer[..bytes_read])?;
    }
    Ok(())
}

pub fn compress<R: Read>(reader: R, format: CompressionFormat) -> io::Result<Vec<u8>> {
    let mut result = Vec::new();
    let mut encoder = format.create_encoder(result, Compression::default());
    process_stream(reader, &mut encoder)?;
    Ok(encoder.finish()?.into_inner())
}

pub fn decompress<R: Read>(reader: R, format: CompressionFormat) -> io::Result<Vec<u8>> {
    let mut result = Vec::new();
    let mut decoder = format.create_decoder(reader);
    process_stream(decoder, &mut result)?;
    Ok(result)
}
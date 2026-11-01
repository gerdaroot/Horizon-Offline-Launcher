//! Packet framing (Varint21FrameDecoder / Varint21LengthFieldPrepender) and
//! zlib compression above a threshold (CompressionEncoder / CompressionDecoder).
use crate::{
    Error, Result,
    buf::{Reader, write_varint},
};
use flate2::{Compression, Decompress, FlushDecompress, Status, write::ZlibEncoder};
use std::io::Write;

/// Frames are at most three VarInt bytes long (21 bits).
pub const MAX_FRAME: usize = (1 << 21) - 1;
/// CompressionDecoder.MAXIMUM_UNCOMPRESSED_LENGTH.
pub const MAX_UNCOMPRESSED: usize = 8_388_608;

/// Build the frame for one packet body (`id` + payload already concatenated):
/// length prefix, and with compression enabled the uncompressed-length
/// field (0 when below the threshold).
pub fn encode(body: &[u8], threshold: Option<usize>) -> Result<Vec<u8>> {
    Ok(prefixed(&contents(body, threshold)?))
}

/// What [`encode`] puts after the length prefix: the same for every
/// connection with the same threshold.
pub fn contents(body: &[u8], threshold: Option<usize>) -> Result<Vec<u8>> {
    let inner = match threshold {
        None => body.to_vec(),
        Some(t) if body.len() < t => {
            let mut v = Vec::with_capacity(body.len() + 1);
            v.push(0);
            v.extend_from_slice(body);
            v
        }
        Some(_) => {
            let mut v = Vec::with_capacity(body.len() / 2 + 8);
            write_varint(&mut v, body.len() as i32);
            let mut z = ZlibEncoder::new(v, Compression::default());
            z.write_all(body)?;
            z.finish()?
        }
    };
    if inner.len() > MAX_FRAME {
        return Err(Error::new(format!(
            "packet too large: {} bytes",
            inner.len()
        )));
    }
    Ok(inner)
}

/// A frame: the length prefix, then the [`contents`].
pub fn prefixed(contents: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(contents.len() + 3);
    write_varint(&mut out, contents.len() as i32);
    out.extend_from_slice(contents);
    out
}

/// Split one complete frame off the front of `buffer`, if available.
/// Returns the frame contents (without the length prefix).
pub fn split(buffer: &mut Vec<u8>) -> Result<Option<Vec<u8>>> {
    let mut len = 0usize;
    for i in 0..3 {
        let Some(&b) = buffer.get(i) else {
            return Ok(None);
        };
        len |= usize::from(b & 0x7F) << (7 * i);
        if b & 0x80 == 0 {
            if len == 0 {
                // Varint21FrameDecoder: a frame with no bytes is a
                // corrupted frame, not an empty packet.
                return Err(Error::new("Frame length cannot be zero"));
            }
            let start = i + 1;
            if buffer.len() < start + len {
                return Ok(None);
            }
            let frame = buffer[start..start + len].to_vec();
            buffer.drain(..start + len);
            return Ok(Some(frame));
        }
    }
    Err(Error::new("length wider than 21 bits"))
}

/// Undo compression on a frame, giving the packet body (id + payload).
pub fn decompress(frame: Vec<u8>, threshold: Option<usize>) -> Result<Vec<u8>> {
    let Some(threshold) = threshold else {
        return Ok(frame);
    };
    let mut r = Reader::new(&frame);
    let size = r.varint()?;
    if size == 0 {
        return Ok(frame[r.position()..].to_vec());
    }
    let size = usize::try_from(size).map_err(|_| Error::new("negative uncompressed length"))?;
    if size < threshold {
        return Err(Error::new(format!(
            "Badly compressed packet - size of {size} is below server threshold of {threshold}"
        )));
    }
    if size > MAX_UNCOMPRESSED {
        return Err(Error::new(format!(
            "Badly compressed packet - size of {size} is larger than protocol maximum"
        )));
    }
    // Where the VarInt ended, not its shortest length: some proxies pad it (`a4 a0 00`), and
    // vanilla's readVarInt takes that.
    let start = r.position();
    // Like vanilla's Inflater: exactly the declared size, and no more. Vanilla never reads the
    // stream's end (its final block and checksum), which some servers' proxies leave out.
    let mut out = vec![0u8; size];
    let mut z = Decompress::new(true);
    let input = &frame[start..];
    loop {
        let (read, written) = (z.total_in() as usize, z.total_out() as usize);
        if written == size {
            break;
        }
        let status = z
            .decompress(&input[read..], &mut out[written..], FlushDecompress::None)
            .map_err(|e| Error::new(format!("Badly compressed packet - {e}")))?;
        let stuck = z.total_in() as usize == read && z.total_out() as usize == written;
        if status == Status::StreamEnd || stuck {
            break;
        }
    }
    let written = z.total_out() as usize;
    if written != size {
        return Err(Error::new(format!(
            "Badly compressed packet - actual length {written} does not match declared size {size}"
        )));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uncompressed_frames_round_trip() {
        let body = vec![0x00, 1, 2, 3];
        let framed = encode(&body, None).unwrap();
        assert_eq!(framed, [4, 0, 1, 2, 3]);
        let mut buffer = framed.clone();
        buffer.extend_from_slice(&framed[..2]);
        assert_eq!(split(&mut buffer).unwrap(), Some(body));
        assert_eq!(split(&mut buffer).unwrap(), None);
        assert_eq!(buffer, [4, 0]);
    }

    #[test]
    fn compression_threshold_like_vanilla() {
        // Below the threshold: a zero data length, then the raw body.
        let small = vec![7u8; 10];
        let framed = encode(&small, Some(256)).unwrap();
        assert_eq!(&framed[..2], &[11, 0]);
        let mut buffer = framed;
        let frame = split(&mut buffer).unwrap().unwrap();
        assert_eq!(decompress(frame, Some(256)).unwrap(), small);
        // At or above: the uncompressed size and a zlib stream.
        let big: Vec<u8> = (0..5000u32).map(|i| (i % 7) as u8).collect();
        let framed = encode(&big, Some(256)).unwrap();
        assert!(framed.len() < big.len());
        let mut buffer = framed;
        let frame = split(&mut buffer).unwrap().unwrap();
        assert_eq!(Reader::new(&frame).varint().unwrap(), 5000);
        assert_eq!(decompress(frame, Some(256)).unwrap(), big);
    }

    #[test]
    fn rejects_bad_compressed_lengths() {
        // Declared size below the threshold is a protocol error.
        let mut frame = vec![10];
        let mut z = ZlibEncoder::new(Vec::new(), Compression::default());
        z.write_all(&[1; 10]).unwrap();
        frame.extend(z.finish().unwrap());
        assert!(decompress(frame, Some(256)).is_err());
        // Declared size that does not match the stream.
        let mut frame = Vec::new();
        write_varint(&mut frame, 300);
        let mut z = ZlibEncoder::new(Vec::new(), Compression::default());
        z.write_all(&[1; 299]).unwrap();
        frame.extend(z.finish().unwrap());
        assert!(decompress(frame, Some(256)).is_err());
        // Bytes that aren't zlib at all.
        let mut frame = Vec::new();
        write_varint(&mut frame, 300);
        frame.extend([0xFF; 40]);
        assert!(decompress(frame, Some(256)).is_err());
    }

    #[test]
    fn padded_size_varints_decode_like_vanilla() {
        let body: Vec<u8> = (0..4132u32).map(|i| (i % 13) as u8).collect();
        let mut z = ZlibEncoder::new(Vec::new(), Compression::default());
        z.write_all(&body).unwrap();
        // 4132 in three bytes, as FunTime's proxy writes it.
        let mut frame = vec![0xa4, 0xa0, 0x00];
        frame.extend(z.finish().unwrap());
        assert_eq!(decompress(frame, Some(256)).unwrap(), body);
    }

    #[test]
    fn streams_without_their_end_decode_like_vanilla() {
        let body: Vec<u8> = (0..3000u32).map(|i| (i % 11) as u8).collect();
        let stream = |level| {
            let mut z = ZlibEncoder::new(Vec::new(), Compression::new(level));
            z.write_all(&body).unwrap();
            z.finish().unwrap()
        };
        let framed = |stream: &[u8]| {
            let mut frame = Vec::new();
            write_varint(&mut frame, body.len() as i32);
            frame.extend_from_slice(stream);
            frame
        };
        // Without the Adler-32 checksum, as some proxies send them: vanilla's Inflater stops
        // at the declared size and never reads it.
        let whole = stream(6);
        let unchecked = &whole[..whole.len() - 4];
        assert_eq!(decompress(framed(unchecked), Some(256)).unwrap(), body);
        // Flushed but never finished (no final block).
        let mut z = flate2::Compress::new(Compression::new(6), true);
        let mut open = Vec::with_capacity(body.len() + 64);
        z.compress_vec(&body, &mut open, flate2::FlushCompress::Sync)
            .unwrap();
        assert_eq!(decompress(framed(&open), Some(256)).unwrap(), body);
        // Trailing bytes after the stream are left alone, as vanilla leaves them.
        let mut trailing = whole.clone();
        trailing.extend([1, 2, 3]);
        assert_eq!(decompress(framed(&trailing), Some(256)).unwrap(), body);
    }

    #[test]
    fn a_noncanonical_length_prefix_still_reads() {
        // The data-length VarInt is read, not measured: zero as `0x80 0x00`.
        assert_eq!(
            decompress(vec![0x80, 0x00, 7, 1, 2], Some(256)).unwrap(),
            [7, 1, 2]
        );
        // And a compressed packet whose size was written wide: 300.
        let body = vec![9; 300];
        let mut frame = vec![0xAC, 0x82, 0x00];
        let mut z = ZlibEncoder::new(Vec::new(), Compression::default());
        z.write_all(&body).unwrap();
        frame.extend(z.finish().unwrap());
        assert_eq!(decompress(frame, Some(256)).unwrap(), body);
    }
}

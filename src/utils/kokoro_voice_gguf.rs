//! Reader for the pinned cstr/kokoro-voices-GGUF F32 voice-pack format.
//! Only the style tensor is loaded; inference stays in the existing ONNX runtime.

use std::io::{Cursor, Read};

use ndarray::Array3;

struct Reader<'a>(Cursor<&'a [u8]>);

impl Reader<'_> {
    fn bytes<const N: usize>(&mut self) -> Result<[u8; N], String> {
        let mut value = [0; N];
        self.0.read_exact(&mut value).map_err(|e| e.to_string())?;
        Ok(value)
    }

    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.bytes()?))
    }

    fn u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(self.bytes()?))
    }

    fn string(&mut self) -> Result<String, String> {
        let length = self.u64()?;
        if length > 4096 {
            return Err("GGUF string exceeds voice-pack limit".into());
        }
        let mut value = vec![0; length as usize];
        self.0.read_exact(&mut value).map_err(|e| e.to_string())?;
        String::from_utf8(value).map_err(|e| e.to_string())
    }
}

pub fn read_voice_pack(bytes: &[u8]) -> Result<Array3<f32>, String> {
    let mut reader = Reader(Cursor::new(bytes));
    if reader.bytes::<4>()? != *b"GGUF" || reader.u32()? != 3 || reader.u64()? != 1 {
        return Err("expected GGUF v3 with one voice tensor".into());
    }
    let metadata_count = reader.u64()?;
    if metadata_count > 64 {
        return Err("too many GGUF voice metadata entries".into());
    }
    let mut alignment = 32u64;
    let mut architecture = None;
    for _ in 0..metadata_count {
        let key = reader.string()?;
        match reader.u32()? {
            8 => {
                let value = reader.string()?;
                if key == "general.architecture" {
                    architecture = Some(value);
                }
            }
            4 => {
                let value = reader.u32()?;
                if key == "general.alignment" {
                    alignment = u64::from(value);
                }
            }
            _ => return Err("unsupported GGUF voice metadata type".into()),
        }
    }
    if architecture.as_deref() != Some("kokoro-voice")
        || !alignment.is_power_of_two()
        || alignment > 4096
    {
        return Err("invalid GGUF voice architecture or alignment".into());
    }
    if reader.string()? != "voice.pack"
        || reader.u32()? != 3
        || reader.u64()? != 256
        || reader.u64()? != 1
    {
        return Err("expected voice.pack with shape [rows, 1, 256]".into());
    }
    // GGUF stores dimensions in reverse order. Kikiri has 510 rows; Tundragoon has 512.
    let rows = reader.u64()?;
    if !matches!(rows, 510 | 512) || reader.u32()? != 0 {
        // GGML_TYPE_F32
        return Err("expected F32 voice.pack with 510 or 512 rows".into());
    }
    let offset = reader.u64()?;
    let data_start = reader.0.position().div_ceil(alignment) * alignment;
    let start = data_start
        .checked_add(offset)
        .ok_or("GGUF offset overflow")?;
    let end = start
        .checked_add(rows * 256 * 4)
        .ok_or("GGUF size overflow")?;
    let data = bytes
        .get(
            usize::try_from(start).map_err(|e| e.to_string())?
                ..usize::try_from(end).map_err(|e| e.to_string())?,
        )
        .ok_or("truncated GGUF voice tensor")?;
    let values: Vec<f32> = data
        .chunks_exact(4)
        .map(|v| f32::from_le_bytes([v[0], v[1], v[2], v[3]]))
        .collect();
    if values.iter().any(|v| !v.is_finite()) {
        return Err("GGUF voice tensor contains non-finite values".into());
    }
    Array3::from_shape_vec((rows as usize, 1, 256), values).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::read_voice_pack;

    fn string(out: &mut Vec<u8>, value: &str) {
        out.extend((value.len() as u64).to_le_bytes());
        out.extend(value.as_bytes());
    }

    fn fixture(rows: u64, tensor_type: u32, offset: u64, value: f32) -> Vec<u8> {
        let mut data = b"GGUF".to_vec();
        data.extend(3u32.to_le_bytes());
        data.extend(1u64.to_le_bytes());
        data.extend(1u64.to_le_bytes());
        string(&mut data, "general.architecture");
        data.extend(8u32.to_le_bytes());
        string(&mut data, "kokoro-voice");
        string(&mut data, "voice.pack");
        data.extend(3u32.to_le_bytes());
        for dim in [256u64, 1, rows] {
            data.extend(dim.to_le_bytes());
        }
        data.extend(tensor_type.to_le_bytes());
        data.extend(offset.to_le_bytes());
        data.resize(data.len().div_ceil(32) * 32, 0);
        for _ in 0..rows * 256 {
            data.extend(value.to_le_bytes());
        }
        data
    }

    #[test]
    fn extracts_float_styles_with_gguf_alignment() {
        let styles = read_voice_pack(&fixture(510, 0, 0, 0.125)).expect("voice styles");
        assert_eq!(styles.shape(), &[510, 1, 256]);
        assert_eq!(styles[[0, 0, 0]], 0.125);
        assert_eq!(styles[[509, 0, 255]], 0.125);
    }

    #[test]
    fn accepts_eva_and_bernd_style_dimensions() {
        let styles = read_voice_pack(&fixture(512, 0, 0, 0.25)).expect("512-row voice styles");
        assert_eq!(styles.shape(), &[512, 1, 256]);
        assert_eq!(styles[[511, 0, 255]], 0.25);
        assert!(read_voice_pack(&fixture(509, 0, 0, 0.25)).is_err());
    }

    #[test]
    fn rejects_unsupported_and_corrupt_voice_packs() {
        for (kind, offset, value) in [(1, 0, 0.0), (0, u64::MAX, 0.0), (0, 0, f32::NAN)] {
            assert!(read_voice_pack(&fixture(510, kind, offset, value)).is_err());
        }
        let bytes = fixture(510, 0, 0, 0.125);
        for end in [0, 4, 24, 100, bytes.len() - 1] {
            assert!(read_voice_pack(&bytes[..end]).is_err());
        }
    }
}

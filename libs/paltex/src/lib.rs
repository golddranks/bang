mod common;
#[cfg(any(feature = "decoding", test))]
mod decoding;
#[cfg(any(feature = "encoding", test))]
mod encoding;

#[cfg(any(feature = "decoding", test))]
pub use decoding::decode;

#[cfg(any(feature = "encoding", test))]
pub use encoding::encode;

pub use common::{Color, PalTex};

#[cfg(test)]
mod tests {
    use crate::common::Header;
    use crate::decoding::*;
    use crate::encoding::*;

    #[test]
    fn test_op_roundtrip() {
        fn test(op: u8) {
            let decoded = decode_op(op);
            assert_eq!(encode_op(decoded.0, decoded.1, decoded.2), op);
        }

        for i in 0..=255 {
            test(i);
        }
    }

    #[test]
    fn test_op_clamp() {
        assert_eq!(decode_op(encode_op(0, 0, 0)), (0, 0, 1));
        assert_eq!(decode_op(encode_op(1, 0, 0)), (1, 0, 1));
        assert_eq!(decode_op(encode_op(15, 0, 1)), (15, 0, 1));
        assert_eq!(decode_op(encode_op(16, 0, 1)), (15, 0, 1));
        assert_eq!(decode_op(encode_op(255, 1, 0)), (15, 1, 1));
        assert_eq!(decode_op(encode_op(255, 0, 1)), (15, 0, 1));
        assert_eq!(decode_op(encode_op(255, 1, 1)), (15, 1, 1));
        assert_eq!(decode_op(encode_op(255, 0, 255)), (15, 0, 8));
        assert_eq!(decode_op(encode_op(255, 1, 255)), (15, 1, 8));
        assert_eq!(decode_op(encode_op(255, 255, 255)), (15, 1, 8));
    }

    #[test]
    fn test_main_encode() {
        let mut output = Vec::new();
        encode_main(&[0, 0, 0, 0, 0], 5, &mut output);
        assert_eq!(output, &[encode_op(0, 0, 5)]);
        output.clear();
        encode_main(&[1, 1, 1, 0, 0], 5, &mut output);
        assert_eq!(output, &[encode_op(1, 0, 3), encode_op(0, 0, 2)]);
        output.clear();
        encode_main(&[2, 0, 0, 0, 0, 2, 1, 1, 1, 1], 5, &mut output);
        assert_eq!(
            output,
            &[encode_op(2, 1, 2), encode_op(0, 0, 4), encode_op(1, 0, 4)]
        );
    }

    #[test]
    fn test_main_roundtrip() {
        let input = &[
            0, 1, 0, 0, 0, //
            0, 0, 1, 0, 0, //
            0, 2, 0, 1, 0, //
            0, 2, 0, 0, 1, //
            0, 0, 0, 0, 0, //
        ];
        let mut output = Vec::new();
        encode_main(input, 5, &mut output);
        let header = Header {
            width: 5,
            height: 5,
            pal_len: 15,
        };
        let decoded = decode_main(&header, &output);
        assert_eq!(decoded, input);
    }
}

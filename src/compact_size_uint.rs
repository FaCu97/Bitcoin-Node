#[derive(Clone, Debug)]
pub struct CompactSizeUint {
    value: u128,
}

impl CompactSizeUint {
    pub fn new(value: u128) -> Self {
        CompactSizeUint {
            value: Self::generate_compact_size_uint(value),
        }
    }

    fn generate_compact_size_uint(value: u128) -> u128 {
        if 253 <= value && value <= 0xffff {
            return Self::get_compact_size_uint(0xfd, 13, value);
        }
        if 0x10000 <= value && value <= 0xffffffff {
            return Self::get_compact_size_uint(0xfe, 11, value);
        }
        if 0x100000000 <= value && value <= 0xffffffffffffffff {
            return Self::get_compact_size_uint(0xff, 7, value);
        }
        value
    }

    pub fn value(&self) -> u128 {
        self.value
    }

    fn get_compact_size_uint(first_byte: u8, shift: usize, value: u128) -> u128 {
        let mut bytes: [u8; 16] = [0; 16];
        bytes[shift] = first_byte;
        let aux_bytes: [u8; 16] = value.to_le_bytes();
        let start = shift + 1;
        for x in start..16 {
            bytes[x] = aux_bytes[x - start];
        }
        let compact_size_uint = u128::from_be_bytes(bytes);
        compact_size_uint
    }
}

#[cfg(test)]
mod test {
    use crate::compact_size_uint::CompactSizeUint;

    #[test]
    fn test_el_numero_200_se_representa_como_0x_c8() {
        let valor: u128 = 200;
        let valor_retornado: CompactSizeUint = CompactSizeUint::new(valor);
        let valor_esperado: u128 = 0xC8;
        assert_eq!(valor_retornado.value(), valor_esperado);
    }

    #[test]
    fn test_el_numero_505_se_representa_como_0x_fd_f9_01() {
        let valor: u128 = 505;
        let valor_retornado: CompactSizeUint = CompactSizeUint::new(valor);
        let valor_esperado: u128 = 0xFDF901;
        assert_eq!(valor_retornado.value(), valor_esperado);
    }

    #[test]
    fn test_el_numero_100000_se_representa_como_0x_fe_a0_86_01_00() {
        let valor: u128 = 100000;
        let valor_retornado: CompactSizeUint = CompactSizeUint::new(valor);
        let valor_esperado: u128 = 0xFEA0860100;
        assert_eq!(valor_retornado.value(), valor_esperado);
    }

    #[test]
    fn test_el_numero_5000000000_se_representa_como_0x_ff_00_f2_05_2a_01_00_00_00() {
        let valor: u128 = 5000000000;
        let valor_retornado: CompactSizeUint = CompactSizeUint::new(valor);
        let valor_esperado: u128 = 0xFF00F2052A01000000;
        assert_eq!(valor_retornado.value(), valor_esperado);
    }
}

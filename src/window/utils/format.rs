// SPDX-License-Identifier: GPL-3.0-or-later

pub fn fmt_bytes(bytes: u64) -> String {
    const GIB: u64 = 1_073_741_824;
    const MIB: u64 = 1_048_576;
    if bytes >= GIB {
        format!("{:.1} GB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.0} MB", bytes as f64 / MIB as f64)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_gib() {
        assert_eq!(fmt_bytes(2_147_483_648), "2.0 GB");
    }

    #[test]
    fn formats_mib() {
        assert_eq!(fmt_bytes(10_485_760), "10 MB");
    }

    #[test]
    fn formats_bytes_below_mib() {
        assert_eq!(fmt_bytes(512), "512 B");
    }

    #[test]
    fn formats_zero() {
        assert_eq!(fmt_bytes(0), "0 B");
    }
}

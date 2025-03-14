pub(crate) fn level_table_lookup(level: u32) -> u32 {
    match level {
        51 => 120,
        52 => 125,
        53 => 130,
        54 => 133,
        55 => 136,
        56 => 139,
        57 => 142,
        58 => 145,
        59 => 148,
        60 => 150,
        61 => 260,
        62 => 265,
        63 => 270,
        64 => 273,
        65 => 276,
        66 => 279,
        67 => 282,
        68 => 285,
        69 => 288,
        70 => 290,
        71 => 390,
        72 => 395,
        73 => 400,
        74 => 403,
        75 => 406,
        76 => 409,
        77 => 412,
        78 => 415,
        79 => 418,
        80 => 420,
        81 => 517,
        82 => 520,
        83 => 525,
        84 => 530,
        85 => 535,
        86 => 540,
        87 => 545,
        88 => 550,
        89 => 555,
        90 => 560,
        _ => level,
    }
}

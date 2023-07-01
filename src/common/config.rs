use phf::phf_map;

pub static TIMINGS: phf::Map<i32, i64> = phf_map! {
  0i32 => 1,
  1i32 => 12,
  2i32 => 24,
  3i32 => 36,
  4i32 => 48,
  5i32 => 72,
};

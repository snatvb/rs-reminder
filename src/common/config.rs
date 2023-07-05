use phf::phf_map;

#[cfg(release)]
static ONE_HOUR: i64 = 1 * 60 * 60;

#[cfg(release)]
pub static TIMINGS: phf::Map<i32, i64> = phf_map! {
  0i32 => ONE_HOUR,
  1i32 => ONE_HOUR * 12,
  2i32 => ONE_HOUR * 24,
  3i32 => ONE_HOUR * 32,
  4i32 => ONE_HOUR * 48,
  5i32 => ONE_HOUR * 72,
};

#[cfg(debug_assertions)]
pub static TIMINGS: phf::Map<i32, i64> = phf_map! {
  0i32 => 1 * 30,
  1i32 => 2 * 30,
  2i32 => 3 * 30,
  3i32 => 4 * 30,
  4i32 => 5 * 30,
  5i32 => 6 * 30,
};

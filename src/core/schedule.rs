pub struct Rotation {
    // UNIX timestamp for the start
    pub start: i64,
    // UNIX timestamp for the end
    pub end: i64,
    // Map IDs for Turf
    pub regular: Vec<i32>,
    // Map IDs for Ranked
    pub gachi: Vec<i32>,
    // Regular isn't hardcoded to Turf War for some reason
    pub regular_rule: String,
    // Ranked Mode (cVar, cVlf, cVgl)
    pub gachi_rule: String,
}

pub struct Schedule {
    pub phases: Vec<Rotation>,
}

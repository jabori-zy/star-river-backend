use star_river_core::custom_type::CycleId;



#[derive(Debug, Clone)]
pub enum Cycle {
    Id(CycleId),
    Reset,
}

impl Cycle {

    pub fn new() -> Self {
        Cycle::Reset
    }


    pub fn id(&self) -> CycleId {
        match self {
            Cycle::Id(id) => *id,
            Cycle::Reset => 0,
        }
    }


}
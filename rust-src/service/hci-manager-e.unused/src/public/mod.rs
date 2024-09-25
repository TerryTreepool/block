
pub mod status;
pub mod brand;
pub mod product;
pub mod thing;
pub mod group;

pub trait CheckTrait{
    fn check_status(&self) -> near_base::NearResult<()>;
}


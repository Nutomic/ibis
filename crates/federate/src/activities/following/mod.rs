use crate::objects::{instance::InstanceWrapper, user::PersonWrapper};
use either::Either;

pub mod accept;
pub mod follow;
pub mod undo_follow;

type InstanceOrPerson = Either<InstanceWrapper, PersonWrapper>;

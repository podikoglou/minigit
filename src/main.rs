use crate::{
    cli::{Options, options},
    object::ObjectType,
};

pub mod hash;
pub mod object;

mod cli;

fn main() {
    let opts = options().run();

    match opts {
        Options::HashObjectCommand { r#type } => {
            let r#type = r#type.unwrap_or(ObjectType::Blob);
        }
    }
}

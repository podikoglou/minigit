use bpaf::Bpaf;

use crate::object::ObjectType;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub enum Options {
    #[bpaf(command("hash-object"))]
    /// Compute object ID and optionally create an object from a file
    HashObjectCommand {
        /// Specify the type of the object to be created (default: "blob").
        /// Possible values are blob, tree.
        #[bpaf(long("type"), short('t'))]
        r#type: Option<ObjectType>,
    },
}

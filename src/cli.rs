use bpaf::Bpaf;

use crate::{object::ObjectType, object::hash::HashPrefix};

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

        /// Read the object from the standard input instead of from a file.
        #[bpaf(flag(true, false))]
        stdin: bool,

        /// Actually write the object into the object database.
        #[bpaf(short('w'))]
        write: bool,

        #[bpaf(positional("file"))]
        files: Vec<String>,
    },

    /// List buckets in the repository
    #[bpaf(command("ls-buckets"))]
    LsBuckets {},

    /// List objects in the repository
    #[bpaf(command("ls-objects"))]
    LsObjectsCommand {
        #[bpaf(positional("prefix-dirs"))]
        prefix_dirs: Vec<HashPrefix>,
    },
}

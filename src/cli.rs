use bpaf::Bpaf;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(command("hash-object"))]
/// Compute object ID and optionally create an object from a file
pub struct HashObjectCommand {}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {}

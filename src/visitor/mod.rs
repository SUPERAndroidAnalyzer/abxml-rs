use chunks::Chunk;

pub trait ChunkVisitor {
    fn visit(&mut self, chunk: Chunk) {}

    fn visit_string_table(&mut self, chunk: Chunk) {}
}

use kasl::{KaslCompiler, scope_manager::IOBlueprint};
use knodiq_engine::node::Node;
use std::path::PathBuf;

#[derive(Default)]
pub struct KaslNode {
    compiler: KaslCompiler,
    blueprint: Option<IOBlueprint>,
    search_paths: Vec<String>,
}

impl KaslNode {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_search_paths(&mut self, paths: Vec<String>) {
        self.search_paths = paths;
    }

    pub fn compile(&mut self, code: &str) -> Result<(), Vec<kasl::error::ErrorRecord>> {
        // Add the search paths to the compiler
        self.compiler
            .set_search_paths(self.search_paths.iter().map(PathBuf::from).collect());

        // Parse, build and compile the source codes
        self.compiler.parse(code).map_err(|e| vec![*e])?;
        let blueprint = self.compiler.build()?;
        self.compiler.compile_buffer(&blueprint)?;

        // Set the blueprint
        self.blueprint = Some(blueprint);
        Ok(())
    }
}

impl Node for KaslNode {
    fn prepare(&mut self, _audio_ctx: &knodiq_engine::audio_context::AudioContext) {}

    fn process(
        &mut self,
        inputs: &[*const u8],
        outputs: &[*mut u8],
        audio_ctx: &knodiq_engine::audio_context::AudioContext,
    ) {
    }
}

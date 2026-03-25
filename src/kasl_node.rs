use kasl::{KaslCompiler, scope_manager::IOBlueprint};
use knodiq_engine::{
    data_types::{AudioContext, TypeInfo},
    node::Node,
};
use std::path::PathBuf;

#[derive(Default)]
pub struct KaslNode {
    compiler: KaslCompiler,
    blueprint: Option<IOBlueprint>,
    search_paths: Vec<String>,
    code: Option<String>,

    input_types: Vec<TypeInfo>,
    output_types: Vec<TypeInfo>,

    states: Vec<*mut ()>,
    is_first_process: bool,
}

impl KaslNode {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_search_paths(&mut self, paths: Vec<String>) {
        self.search_paths = paths;
    }

    pub fn set_code(&mut self, code: String) {
        self.code = Some(code);
    }

    pub fn compile(&mut self) -> Result<(), Vec<kasl::error::ErrorRecord>> {
        // Add the search paths to the compiler
        self.compiler
            .set_search_paths(self.search_paths.iter().map(PathBuf::from).collect());

        // Parse, build and compile the source codes
        self.compiler
            .parse(self.code.as_ref().unwrap_or(&String::default()))
            .map_err(|e| vec![*e])?;
        let blueprint = self.compiler.build()?;

        // Allocate the state memory based of the blueprint
        for state_item in blueprint.get_states() {
            let layout = std::alloc::Layout::from_size_align(
                state_item.actual_size,
                state_item.align as usize,
            )
            .unwrap();
            let ptr = unsafe { std::alloc::alloc_zeroed(layout) as *mut () };
            self.states.push(ptr);
        }

        // Compile the program
        self.compiler.compile_buffer(&blueprint)?;

        // Set the blueprint
        self.blueprint = Some(blueprint);

        // Update the types
        self.update_type_infos();

        Ok(())
    }

    fn update_type_infos(&mut self) {
        // Create TypeInfo for input types and output types
        self.input_types = self
            .blueprint
            .as_ref()
            .map(|blueprint| {
                blueprint
                    .get_inputs()
                    .iter()
                    .map(|item| TypeInfo::new(item.actual_size, item.align as usize))
                    .collect()
            })
            .unwrap_or_default();
        self.output_types = self
            .blueprint
            .as_ref()
            .map(|blueprint| {
                blueprint
                    .get_outputs()
                    .iter()
                    .map(|item| TypeInfo::new(item.actual_size, item.align as usize))
                    .collect()
            })
            .unwrap_or_default();
    }
}

impl Node for KaslNode {
    fn clone_box(&self) -> Box<dyn Node> {
        Box::new(self.clone())
    }

    fn get_input_names(&self) -> Vec<String> {
        self.blueprint
            .as_ref()
            .map(|blueprint| {
                blueprint
                    .get_inputs()
                    .iter()
                    .map(|i| i.name.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn get_output_names(&self) -> Vec<String> {
        self.blueprint
            .as_ref()
            .map(|blueprint| {
                blueprint
                    .get_outputs()
                    .iter()
                    .map(|i| i.name.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn get_input_len(&self) -> usize {
        self.blueprint
            .as_ref()
            .map(|blueprint| blueprint.get_inputs().len())
            .unwrap_or_default()
    }

    fn get_output_len(&self) -> usize {
        self.blueprint
            .as_ref()
            .map(|blueprint| blueprint.get_outputs().len())
            .unwrap_or_default()
    }

    fn get_input_type(&self, index: usize) -> Option<&TypeInfo> {
        self.input_types.get(index)
    }

    fn get_output_type(&self, index: usize) -> Option<&TypeInfo> {
        self.output_types.get(index)
    }

    fn update(&mut self, _audio_ctx: &AudioContext) {}

    fn prepare(&mut self) {
        self.is_first_process = true;
    }

    fn process(&mut self, inputs: &[*const u8], outputs: &[*mut u8], audio_ctx: &AudioContext) {
        let inputs: Vec<*const ()> = inputs.iter().map(|p| *p as *const ()).collect();
        let outputs: Vec<*mut ()> = outputs.iter().map(|p| *p as *mut ()).collect();

        match self.compiler.run_buffer(
            &inputs,
            &outputs,
            &self.states,
            if self.is_first_process { 1 } else { 0 },
            audio_ctx.buffer_size as i32,
        ) {
            Ok(()) => (),
            Err(err) => eprintln!("An error occured while processing KaslNode: {}", err),
        };

        self.is_first_process = false;
    }
}

impl Clone for KaslNode {
    fn clone(&self) -> Self {
        Self {
            compiler: KaslCompiler::default(),
            blueprint: None,
            search_paths: self.search_paths.clone(),
            code: self.code.clone(),
            input_types: self.input_types.clone(),
            output_types: self.output_types.clone(),
            states: self.states.clone(),
            is_first_process: false,
        }
    }
}

unsafe impl Send for KaslNode {}

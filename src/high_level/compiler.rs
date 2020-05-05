use crate::high_level::problem::CompileProblem;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::time::Instant;

pub const FAKE_BUILTIN_SOURCE: &str = r#"
DataType Auto;
DataType Bool;
DataType Int;
DataType Float;
DataType Void;
"#;

#[derive(Default)]
pub struct PerformanceCounter {
    time: u128,
    num_invocations: u32,
}

impl Display for PerformanceCounter {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}ms ({} invocations)",
            self.time, self.num_invocations
        )
    }
}

#[derive(Default)]
pub struct PerformanceCounters {
    ast: PerformanceCounter,
    vague: PerformanceCounter,
    resolved: PerformanceCounter,
    trivial: PerformanceCounter,
    llvmir: PerformanceCounter,
}

impl Display for PerformanceCounters {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        writeln!(formatter, "          Performance")?;
        writeln!(formatter, "     ast: {}", self.ast)?;
        writeln!(formatter, "   vague: {}", self.vague)?;
        writeln!(formatter, "resolved: {}", self.resolved)?;
        writeln!(formatter, " trivial: {}", self.trivial)?;
        write!(formatter, "  llvmir: {}", self.llvmir)
    }
}

pub struct Compiler {
    sources: Vec<(String, String)>,
    source_indices: HashMap<String, usize>,
    performance_counters: PerformanceCounters,
    error_width: usize,
}

impl Compiler {
    pub fn new() -> Self {
        let mut new = Self {
            sources: Vec::new(),
            source_indices: HashMap::new(),
            performance_counters: Default::default(),
            error_width: 80,
        };
        new.add_source(
            "(internal code) builtins".to_owned(),
            FAKE_BUILTIN_SOURCE.to_owned(),
        );
        new
    }

    pub fn set_error_width(&mut self, width: usize) {
        self.error_width = width;
    }

    pub fn add_source(&mut self, name: String, content: String) {
        self.source_indices.insert(name.clone(), self.sources.len());
        self.sources.push((name, content));
    }

    pub fn add_source_from_file(&mut self, file_path: String) -> std::io::Result<()> {
        let file_content = std::fs::read_to_string(&file_path)?;
        self.add_source(file_path, file_content);
        Ok(())
    }

    pub fn borrow_performance_counters(&self) -> &PerformanceCounters {
        &self.performance_counters
    }

    pub(crate) fn find_source(&self, name: &str) -> Option<usize> {
        self.source_indices.get(name).cloned()
    }

    pub(crate) fn borrow_source(&self, index: usize) -> &(String, String) {
        &self.sources[index]
    }

    pub(crate) fn parse_ast_and_count_performance<'a, 's>(
        &'s mut self,
        source: &'a str,
    ) -> Result<crate::ast::structure::Program<'a>, CompileProblem> {
        let start_time = Instant::now();
        let result = crate::ast::ingest(source);
        self.performance_counters.ast.time += start_time.elapsed().as_millis();
        self.performance_counters.ast.num_invocations += 1;
        result
    }

    fn impl_ast<'a>(
        pc: &mut PerformanceCounters,
        source: &'a str,
    ) -> Result<crate::ast::structure::Program<'a>, CompileProblem> {
        let timer = Instant::now();
        let result = crate::ast::ingest(source);
        pc.ast.time += timer.elapsed().as_millis();
        pc.ast.num_invocations += 1;
        result
    }

    fn impl_vague(
        pc: &mut PerformanceCounters,
        source: &str,
    ) -> Result<crate::vague::structure::Program, CompileProblem> {
        let mut source = Self::impl_ast(pc, source)?;
        let timer = Instant::now();
        let result = crate::vague::ingest(&mut source);
        pc.vague.time += timer.elapsed().as_millis();
        pc.vague.num_invocations += 1;
        result
    }

    fn impl_resolved(
        pc: &mut PerformanceCounters,
        source: &str,
    ) -> Result<crate::resolved::structure::Program, CompileProblem> {
        let mut source = Self::impl_vague(pc, source)?;
        let timer = Instant::now();
        let result = crate::resolved::ingest(&mut source);
        pc.resolved.time += timer.elapsed().as_millis();
        pc.resolved.num_invocations += 1;
        result
    }

    fn impl_trivial(
        pc: &mut PerformanceCounters,
        source: &str,
    ) -> Result<crate::trivial::structure::Program, CompileProblem> {
        let source = Self::impl_resolved(pc, source)?;
        let timer = Instant::now();
        let result = crate::trivial::ingest(&source);
        pc.trivial.time += timer.elapsed().as_millis();
        pc.trivial.num_invocations += 1;
        result
    }

    fn impl_llvmir(
        pc: &mut PerformanceCounters,
        source: &str,
    ) -> Result<crate::llvmir::structure::Program, CompileProblem> {
        let source = Self::impl_trivial(pc, source)?;
        let timer = Instant::now();
        let result = crate::llvmir::ingest(&source);
        pc.llvmir.time += timer.elapsed().as_millis();
        pc.llvmir.num_invocations += 1;
        Ok(result)
    }

    fn format_error<T>(&self, result: Result<T, CompileProblem>) -> Result<T, String> {
        result.map_err(|e| e.format(self.error_width, self))
    }

    pub fn compile_to_ast<'a>(
        &'a mut self,
        source_name: &str,
    ) -> Result<crate::ast::structure::Program<'a>, String> {
        let source = if let Some(index) = self.find_source(source_name) {
            &self.sources[index].1
        } else {
            return Err(format!("Could not find a source named {}.", source_name));
        };
        let result = Self::impl_ast(&mut self.performance_counters, source);
        self.format_error(result)
    }

    pub fn compile_to_vague(
        &mut self,
        source_name: &str,
    ) -> Result<crate::vague::structure::Program, String> {
        let source = if let Some(index) = self.find_source(source_name) {
            &self.sources[index].1
        } else {
            return Err(format!("Could not find a source named {}.", source_name));
        };
        let result = Self::impl_vague(&mut self.performance_counters, source);
        self.format_error(result)
    }

    pub fn compile_to_resolved(
        &mut self,
        source_name: &str,
    ) -> Result<crate::resolved::structure::Program, String> {
        let source = if let Some(index) = self.find_source(source_name) {
            &self.sources[index].1
        } else {
            return Err(format!("Could not find a source named {}.", source_name));
        };
        let result = Self::impl_resolved(&mut self.performance_counters, source);
        self.format_error(result)
    }

    pub fn compile_to_trivial(
        &mut self,
        source_name: &str,
    ) -> Result<crate::trivial::structure::Program, String> {
        let source = if let Some(index) = self.find_source(source_name) {
            &self.sources[index].1
        } else {
            return Err(format!("Could not find a source named {}.", source_name));
        };
        let result = Self::impl_trivial(&mut self.performance_counters, source);
        self.format_error(result)
    }

    pub fn compile_to_llvmir(
        &mut self,
        source_name: &str,
    ) -> Result<crate::llvmir::structure::Program, String> {
        let source = if let Some(index) = self.find_source(source_name) {
            &self.sources[index].1
        } else {
            return Err(format!("Could not find a source named {}.", source_name));
        };
        let result = Self::impl_llvmir(&mut self.performance_counters, source);
        self.format_error(result)
    }

    pub fn compile(
        &mut self,
        source_name: &str,
    ) -> Result<crate::llvmir::structure::Program, String> {
        self.compile_to_llvmir(source_name)
    }
}

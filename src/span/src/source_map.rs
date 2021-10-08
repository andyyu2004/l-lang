use crate::{Span, Symbol};
use codespan_reporting::files::{line_starts, Files};
use index::{newtype_index, IndexVec};
use std::ffi::OsString;
use std::ops::{Deref, Index};
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct SourceMap {
    modules: IndexVec<FileIdx, SourceFile>,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ModuleKind {
    File,
    Dir,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ModuleFile {
    pub path: PathBuf,
    pub kind: ModuleKind,
}

impl ModuleFile {
    pub fn new(path: impl AsRef<Path>, kind: ModuleKind) -> Self {
        let path = path.as_ref().canonicalize().unwrap();
        Self { path, kind }
    }
}

impl SourceMap {
    pub fn add_src_file(&mut self, module_file: ModuleFile) -> FileIdx {
        let src_file = SourceFile::new(module_file);
        self.modules.push(src_file)
    }

    pub fn add_module(&mut self, current_file: FileIdx, sym: Symbol) -> Option<FileIdx> {
        self.find_module_file(current_file, sym).map(|path| self.add_src_file(path))
    }

    fn find_module_file(&mut self, current_file: FileIdx, sym: Symbol) -> Option<ModuleFile> {
        let path = self.path_of(current_file).parent().unwrap();

        let check_path = |p: &Path| p.is_file() && p.extension() == Some(&OsString::from("l"));

        // suppose we are in a module at `/path/to/file.l`
        // and we see a `mod foo` in `file.l`,
        // then we check both `/path/to/foo.l` and `path/to/foo/foo.l`
        // we call these `module_file_path` and `module_dir_path` respectively
        // for the `foo` module file
        let module_file_path = path.join(format!("{}.l", sym));
        if check_path(&module_file_path) {
            return Some(ModuleFile::new(module_file_path, ModuleKind::File));
        } else {
            let module_dir_path = path.join(format!("{}/{}.l", sym, sym));
            if check_path(&module_dir_path) {
                return Some(ModuleFile::new(module_dir_path, ModuleKind::Dir));
            }
        };
        None
    }

    pub fn dir_of(&self, file: FileIdx) -> &Path {
        self.path_of(file).parent().unwrap()
    }

    pub fn path_of(&self, idx: FileIdx) -> &Path {
        &self.get(idx).file.path
    }

    pub fn get_opt(&self, idx: FileIdx) -> Option<&SourceFile> {
        self.modules.get(idx)
    }

    pub fn get(&self, idx: FileIdx) -> &SourceFile {
        &self.modules[idx]
    }
}

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub file: ModuleFile,
    name: String,
    src: &'static str,
    line_starts: Vec<usize>,
}

impl SourceFile {
    pub fn new(file: ModuleFile) -> Self {
        let src = Symbol::intern_str(&std::fs::read_to_string(&file.path).unwrap());

        Self {
            name: file.path.file_name().unwrap().to_str().unwrap().to_owned(),
            line_starts: line_starts(src).collect(),
            file,
            src,
        }
    }

    pub fn source(&self) -> &'static str {
        &self.src
    }

    fn line_start(&self, line_index: usize) -> Option<usize> {
        use std::cmp::Ordering;

        match line_index.cmp(&self.line_starts.len()) {
            Ordering::Less => self.line_starts.get(line_index).cloned(),
            Ordering::Equal => Some(self.src.len()),
            Ordering::Greater => None,
        }
    }
}

impl Deref for SourceFile {
    type Target = str;

    fn deref(&self) -> &'static str {
        &self.src
    }
}

newtype_index!(
    #[derive(Serialize, Deserialize)]
    pub struct FileIdx {
        DEBUG_FORMAT = "{}",
        const ROOT_FILE_IDX = 0,
    }
);
// mostly copied from
// https://docs.rs/codespan-reporting/0.9.5/src/codespan_reporting/files.rs.html#208-215
impl<'a> Files<'a> for SourceMap {
    type FileId = FileIdx;
    type Name = &'a str;
    type Source = &'a str;

    fn name(&'a self, id: Self::FileId) -> Option<Self::Name> {
        Some(&self.modules[id].name)
    }

    fn source(&'a self, id: Self::FileId) -> Option<Self::Source> {
        Some(&self.modules[id].src)
    }

    fn line_index(&'a self, id: Self::FileId, byte_index: usize) -> Option<usize> {
        Some(match self.modules[id].line_starts.binary_search(&byte_index) {
            Ok(line) => line,
            Err(next_line) => next_line - 1,
        })
    }

    fn line_range(&'a self, id: Self::FileId, line_index: usize) -> Option<std::ops::Range<usize>> {
        let file = &self.modules[id];
        let line_start = file.line_start(line_index)?;
        let next_line_start = file.line_start(line_index + 1)?;

        Some(line_start..next_line_start)
    }
}

impl SourceMap {
    pub fn new(main_file_path: impl AsRef<Path>) -> Self {
        let mut source_map = Self { modules: Default::default() };
        // we consider the main module as a `dir module`, otherwise we won't be allowed to declare
        // submodules within it
        let module_file = ModuleFile::new(main_file_path, ModuleKind::Dir);
        source_map.modules.push(SourceFile::new(module_file));
        source_map
    }

    pub fn span_as_str(&self, span: Span) -> &'static str {
        let src: &'static str = &self.modules[span.file].src;
        &src[span.range()]
    }
}

impl<'a> Index<Span> for &'a SourceFile {
    type Output = str;

    fn index(&self, span: Span) -> &'static str {
        &self.src[span.start().to_usize()..span.end().to_usize()]
    }
}

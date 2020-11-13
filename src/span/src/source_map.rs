use crate::{Span, Symbol};
use codespan_reporting::files::{line_starts, Files};
use index::{newtype_index, IndexVec};
use std::ffi::OsString;
use std::ops::{Deref, Index};
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct SourceMap {
    files: IndexVec<FileIdx, SourceFile>,
}

impl SourceMap {
    pub fn add_src_file(&mut self, path: impl AsRef<Path>) -> FileIdx {
        let src_file = SourceFile::new(path);
        self.files.push(src_file)
    }

    pub fn add_module(&mut self, current_file: FileIdx, sym: Symbol) -> Option<FileIdx> {
        self.find_module_file(current_file, sym).map(|path| self.add_src_file(path))
    }

    fn find_module_file(&mut self, current_file: FileIdx, sym: Symbol) -> Option<PathBuf> {
        let path = self.path_of(current_file).parent().unwrap();

        let check_path = |p: &Path| p.is_file() && p.extension() == Some(&OsString::from("l"));

        // suppose we are in a module at `/path/to/file.l`
        // and we see a `mod foo` in `file.l`,
        // then we check both `/path/to/foo.l` and `path/to/foo/foo.l`
        // we call these `module_file_path` and `module_dir_path` respectively
        // for the `foo` module file
        let module_file_path = path.join(format!("{}.l", sym));
        if check_path(&module_file_path) {
            return Some(module_file_path);
        } else {
            let module_dir_path = path.join(format!("{}/{}.l", sym, sym));
            if check_path(&module_dir_path) {
                return Some(module_dir_path);
            }
        };
        None
    }

    pub fn dir_of(&self, file: FileIdx) -> &Path {
        self.path_of(file).parent().unwrap()
    }

    pub fn path_of(&self, file: FileIdx) -> &Path {
        &self.files[file].path
    }

    pub fn get(&self, file: FileIdx) -> &SourceFile {
        &self.files[file]
    }
}

#[derive(Debug, Clone)]
pub struct SourceFile {
    path: PathBuf,
    name: String,
    src: String,
    line_starts: Vec<usize>,
}

impl SourceFile {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().canonicalize().unwrap();
        let src = std::fs::read_to_string(&path).unwrap();
        Self {
            path: path.to_path_buf(),
            name: path.file_name().unwrap().to_str().unwrap().to_owned(),
            line_starts: line_starts(&src).collect(),
            src,
        }
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

    fn deref(&self) -> &Self::Target {
        &self.src
    }
}

newtype_index!(
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
        Some(&self.files[id].name)
    }

    fn source(&'a self, id: Self::FileId) -> Option<Self::Source> {
        Some(&self.files[id].src)
    }

    fn line_index(&'a self, id: Self::FileId, byte_index: usize) -> Option<usize> {
        Some(match self.files[id].line_starts.binary_search(&byte_index) {
            Ok(line) => line,
            Err(next_line) => next_line - 1,
        })
    }

    fn line_range(&'a self, id: Self::FileId, line_index: usize) -> Option<std::ops::Range<usize>> {
        let file = &self.files[id];
        let line_start = file.line_start(line_index)?;
        let next_line_start = file.line_start(line_index + 1)?;

        Some(line_start..next_line_start)
    }
}

impl SourceMap {
    // just one sourcefile for now
    pub fn new(path: impl AsRef<Path>) -> Self {
        let mut source_map = Self { files: Default::default() };
        source_map.files.push(SourceFile::new(path));
        source_map
    }

    pub fn span_to_slice(&self, span: Span) -> &str {
        &self.files[span.file].src[span.start().to_usize()..span.end().to_usize()]
    }

    pub fn span_to_string(&self, span: Span) -> String {
        self.span_to_slice(span).to_owned()
    }
}

impl<'a> Index<Span> for &'a SourceFile {
    type Output = str;

    fn index(&self, span: Span) -> &Self::Output {
        &self.src[span.start().to_usize()..span.end().to_usize()]
    }
}

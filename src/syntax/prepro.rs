use errors::*;
use common::*;
use syntax::*;
use std::path::{Path, PathBuf};

impl Import {
    pub fn import<P: AsRef<Path>>(self, search_dirs: &[P]) -> Result<FunDefList> {

        // we also want to append the source dir of the current file to the PATH
        let source_path = &self.range()
            .source_path();
        let source_dir = Path::new(source_path.as_ref())
            .parent();
        let mut local_search_dirs = search_dirs.iter()
            .map(|p| p.as_ref())
            .collect::<Vec<_>>();
        if let Some(dir) = source_dir {
            local_search_dirs.insert(0, dir.as_ref());
        }

        let import_path = self.path;
        let full_path = search_path(&import_path, &local_search_dirs)
            .map(|r| Ok(r) as Result<PathBuf>)
            .unwrap_or(Err(format!("could not find file `{}` in search path", import_path).into()))?;
        process_source_path(&full_path, search_dirs)
            .map(|ast| ast.ast)
            .chain_err(|| format!("imported from file `{}`", import_path))
    }
}

impl AST {

    pub fn preprocess<P: AsRef<Path>>(self, search_dirs: &[P]) -> Result<FilledAST> {
        let AST { ast, path } = self;
        let (ast, errors) = ast.into_iter()
            .map(|top| {
                match top {
                    TopLevel::FunDef(f) => Ok(vec![f]),
                    TopLevel::Import(i) => i.import(search_dirs),
                }
            })
            .fold((vec![], vec![]), |(mut ast, mut errors), item| {
                match item {
                    Ok(i) => ast.push(i),
                    Err(e) => errors.push(e),
                }
                (ast, errors)
            });
        
        if !errors.is_empty() {
            // TODO : print all errors
            return Err(errors.into_iter().nth(0).unwrap());
        }

        let ast = ast.into_iter()
            .flat_map(id)
            .collect::<Vec<_>>();

        Ok(FilledAST {
            ast,
            path,
        })
    }
}

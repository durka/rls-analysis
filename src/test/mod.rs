// Copyright 2016 The RLS Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use {AnalysisHost, AnalysisLoader};
use raw::DefKind;

use std::path::{Path, PathBuf};

#[derive(Clone, new)]
struct TestAnalysisLoader {
    path: PathBuf,
}

impl AnalysisLoader for TestAnalysisLoader {
    fn needs_hard_reload(&self, _path_prefix: &Path) -> bool {
        true
    }

    fn fresh_host(&self) -> AnalysisHost<Self> {
        AnalysisHost::new_with_loader(self.clone())
    }

    fn set_path_prefix(&self, _path_prefix: &Path) {}

    fn abs_path_prefix(&self) -> Option<PathBuf> {
        panic!();
    }

    fn iter_paths<F, T>(&self, f: F) -> Vec<T>
    where
        F: Fn(&Path) -> Vec<T>,
    {
        let paths = &[&self.path];
        paths.iter().flat_map(|p| f(p).into_iter()).collect()
    }
}

#[test]
fn doc_urls_resolve_correctly() {
    let host = AnalysisHost::new_with_loader(TestAnalysisLoader::new(
        Path::new("test_data/rust-analysis").to_owned(),
    ));
    host.reload(
        Path::new("test_data/rust-analysis"),
        Path::new("test_data/rust-analysis"),
    ).unwrap();

    fn assert_url_for_type<S: Into<Option<&'static str>>>(
        host: &AnalysisHost<TestAnalysisLoader>,
        type_: &str,
        qualname: S,
        url: &str,
    ) {
        let qualname = qualname.into();
        let ids = host.search_for_id(type_).unwrap();
        let defs: Vec<_> = ids.into_iter()
            .map(|id| host.get_def(id).unwrap())
            .filter(|def| {
                qualname.is_none() || def.qualname == qualname.unwrap()
            })
            .collect();
        println!("{:#?}", defs);
        assert_eq!(defs.len(), 1);
        assert_eq!(host.doc_url(&defs[0].span), Ok(url.into()));
    }

    // FIXME This test cannot work for some values
    // Primitives like i64. i64 is shown with type mod but requires name "primitive".
    // All methods (instead of trait methods, see as_mut), seem to only be available for generic qualname
    // Unions like ManuallyDrop are not in the analysis file, just methods implemented for them or methods using them

    assert_url_for_type(
        &host,
        "MAIN_SEPARATOR",
        None,
        "https://doc.rust-lang.org/nightly/std/path/MAIN_SEPARATOR.v.html",
    );
    // the parent has a qualname which is not represented in the usage, the ip part
    assert_url_for_type(
        &host,
        "Ipv4Addr",
        None,
        "https://doc.rust-lang.org/nightly/std/net/ip/Ipv4Addr.t.html",
    );
    assert_url_for_type(
        &host,
        "VarError",
        None,
        "https://doc.rust-lang.org/nightly/std/env/VarError.t.html",
    );
    assert_url_for_type(
        &host,
        "NotPresent",
        None,
        "https://doc.rust-lang.org/nightly/std/env/VarError.t.html#NotPresent.v",
    );
    assert_url_for_type(
        &host,
        "Result",
        "std::thread::Result",
        "https://doc.rust-lang.org/nightly/std/thread/Result.t.html",
    );
    assert_url_for_type(
        &host,
        "args",
        "std::env::args",
        "https://doc.rust-lang.org/nightly/std/env/args.v.html",
    );
    assert_url_for_type(
        &host,
        "AsciiExt",
        None,
        "https://doc.rust-lang.org/nightly/std/ascii/AsciiExt.t.html",
    );
    assert_url_for_type(
        &host,
        "is_ascii",
        None,
        "https://doc.rust-lang.org/nightly/std/ascii/AsciiExt.t.html#is_ascii.v",
    );
    assert_url_for_type(
        &host,
        "status",
        "std::process::Output::status",
        "https://doc.rust-lang.org/nightly/std/process/Output.t.html#status.v",
    );
    assert_url_for_type(
        &host,
        "copy",
        "std::fs::copy",
        "https://doc.rust-lang.org/nightly/std/fs/copy.v.html",
    );
    // prelude and fs are both mod, but the parent once has a trailing / and once not
    assert_url_for_type(
        &host,
        "prelude",
        "std::io::prelude",
        "https://doc.rust-lang.org/nightly/std/io/prelude/",
    );
    assert_url_for_type(
        &host,
        "fs",
        "std::fs",
        "https://doc.rust-lang.org/nightly/std/fs/",
    );
}

#[test]
fn smoke() {
    // Read in test data and lower it, check we don't crash.
    let host = AnalysisHost::new_with_loader(TestAnalysisLoader::new(
        Path::new("test_data/rls-analysis").to_owned(),
    ));
    host.reload(
        Path::new("test_data/rls-analysis"),
        Path::new("test_data/rls-analysis"),
    ).unwrap();
}

#[test]
fn test_hello() {
    // Simple program, a somewhat thorough test that we have all the defs and refs we expect.
    let host = AnalysisHost::new_with_loader(TestAnalysisLoader::new(
        Path::new("test_data/hello/save-analysis").to_owned(),
    ));
    host.reload(
        Path::new("test_data/hello"),
        Path::new("test_data/hello"),
    ).unwrap();

    let ids = host.search_for_id("print_hello").unwrap();
    assert_eq!(ids.len(), 1);
    let id = ids[0];
    let def = host.get_def(id).unwrap();
    assert_eq!(def.name, "print_hello");
    assert_eq!(def.kind, DefKind::Function);
    let refs = host.find_all_refs_by_id(id).unwrap();
    assert_eq!(refs.len(), 2);
    assert_eq!(refs[0].file, Path::new("test_data/hello/src/main.rs"));
    assert_eq!(refs[0].range.row_start.0, 0);
    assert_eq!(refs[1].file, Path::new("test_data/hello/src/main.rs"));
    assert_eq!(refs[1].range.row_start.0, 6);
    let refs = host.search("print_hello").unwrap();
    assert_eq!(refs.len(), 2);
    assert_eq!(refs[0].file, Path::new("test_data/hello/src/main.rs"));
    assert_eq!(refs[0].range.row_start.0, 0);
    assert_eq!(refs[1].file, Path::new("test_data/hello/src/main.rs"));
    assert_eq!(refs[1].range.row_start.0, 6);

    let ids = host.search_for_id("main").unwrap();
    assert_eq!(ids.len(), 1);
    let id = ids[0];
    let def = host.get_def(id).unwrap();
    assert_eq!(def.name, "main");
    assert_eq!(def.kind, DefKind::Function);
    let refs = host.find_all_refs_by_id(id).unwrap();
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].file, Path::new("test_data/hello/src/main.rs"));
    assert_eq!(refs[0].range.row_start.0, 5);
    let refs = host.search("main").unwrap();
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].file, Path::new("test_data/hello/src/main.rs"));
    assert_eq!(refs[0].range.row_start.0, 5);

    let ids = host.search_for_id("name").unwrap();
    assert_eq!(ids.len(), 1);
    let id = ids[0];
    let def = host.get_def(id).unwrap();
    assert_eq!(def.name, "name");
    assert_eq!(def.kind, DefKind::Local);
    let refs = host.find_all_refs_by_id(id).unwrap();
    assert_eq!(refs.len(), 2);
    assert_eq!(refs[0].file, Path::new("test_data/hello/src/main.rs"));
    assert_eq!(refs[0].range.row_start.0, 1);
    assert_eq!(refs[1].file, Path::new("test_data/hello/src/main.rs"));
    assert_eq!(refs[1].range.row_start.0, 2);
    let refs = host.search("name").unwrap();
    assert_eq!(refs.len(), 2);
    assert_eq!(refs[0].file, Path::new("test_data/hello/src/main.rs"));
    assert_eq!(refs[0].range.row_start.0, 1);
    assert_eq!(refs[1].file, Path::new("test_data/hello/src/main.rs"));
    assert_eq!(refs[1].range.row_start.0, 2);
}

// TODO
// check span functions
// check complex programs

#[test]
fn test_types() {
    // TODO test unit structs, structs with fields, enums, aliases, traits, etc.
    let host = AnalysisHost::new_with_loader(TestAnalysisLoader::new(
        Path::new("test_data/types/save-analysis").to_owned(),
    ));
    host.reload(
        Path::new("test_data/types"),
        Path::new("test_data/types"),
    ).unwrap();

    let ids = host.search_for_id("Foo").unwrap();
    assert_eq!(ids.len(), 1);
    let id = ids[0];
    let def = host.get_def(id).unwrap();
    assert_eq!(def.name, "Foo");
    assert_eq!(def.kind, DefKind::Struct);

    let refs = host.find_all_refs_by_id(id).unwrap();
    println!("{:?}", refs);
    assert_eq!(refs.len(), 5);
    assert_eq!(refs[0].file, Path::new("test_data/types/src/main.rs"));
    assert_eq!(refs[0].range.row_start.0, 0);
    assert_eq!(refs[1].file, Path::new("test_data/types/src/main.rs"));
    assert_eq!(refs[1].range.row_start.0, 5);
    assert_eq!(refs[2].file, Path::new("test_data/types/src/main.rs"));
    assert_eq!(refs[2].range.row_start.0, 6);
    assert_eq!(refs[3].file, Path::new("test_data/types/src/main.rs"));
    assert_eq!(refs[3].range.row_start.0, 9);
    assert_eq!(refs[4].file, Path::new("test_data/types/src/main.rs"));
    assert_eq!(refs[4].range.row_start.0, 9);
}

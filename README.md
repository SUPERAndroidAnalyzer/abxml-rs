[![Coverage Status][coverage_badge]][coverage_link]
[![Build Status][status_badge]][status_link]

# ABXML

ABXML is a library that is able to decode APK files in Rust. It tries to make easy to work with the binary documents found inside the APK files. It is able to decode the contained `resources.arsc` and also all the binary XML documents contained on the `res/` folder. It also exposes some structures to work at a lower level, in case anyone will be interested.

The code is deeply inspired on Apktool: Without it, this library wouldn't exist.

## Quick example

The easiest way to use the library is using the helper struct `APK` to decompress and decode it to the filesystem.

```rust
use std::path::Path;
use abxml::apk::Apk;

fn main() {
    let path = Path::new("path_to.apk");
    let mut apk = Apk::new(path).unwrap();
    apk.export(Path::new("/tmp/apk_output/"), false).unwrap();
}
```

The `Apk::new` will create a handler that will allow to export to the filesystem. At this moment, it will load to memory the APK, decompress it and parse the contained `resources.arsc`. If this process succeeds, using the method `export`, it will start exporting all the contained files. If it finds an Android binary XML, it will convert it to a string version of it; otherwise, it will move it to the filesystem as is. The second parameter on the `export` function is used to force the removal of the path given on the first argument. In this case, the second invocation of this snippet will fail, as the directory will be non empty.

## Visitors

This library uses the visitor pattern to access to the contents of a binary file. There is a helper struct called `Executor` which is in charge of, given the contents of one binary file, call to the corresponding functions on the given visitor. The next example will print to the output the message for each string table found:

```rust
use abxml::visitor::{Executor, ChunkVisitor};

pub struct PrintVisitor;

impl<'a> ChunkVisitor<'a> for PrintVisitor {
    fn visit_string_table(&mut self, string_table: StringTableWrapper<'a>, origin: Origin) {
        println!("Found string table with origin: {:?}", origin);
    }
}

fn main() {
    let data = [];
    let mut visitor = PrintVisitor;

    Executor::xml(&data, &mut visitor);
}
```

`Executor` contains two public methods that should be used depending on the type of the input: `arsc` to decode `resources.arsc` and `xml` for binary XMLs. The reason of this split is because the header of the files is distinct (`resources.arsc` has a 12 bytes header, while binary XMLs has 8 bytes).

## Wrapper, Buffers and traits

On the model namespace there are several traits that exposes how the library works with each one of the concepts behind the binary files. Each of the traits are usually implemented by both wrapper and buffers. Why this distinction?

Wrappers gives a read-only view of the represented chunk. This way, the library only allocates data when is accessed. On the other hand, the Buf structs are owned and intended to be used as mutable.

All wrappers have functions to be converted to buffers (`to_buffer`) and all buffers have functions to create an encoded view as bytes (through the `OwnedBuf` trait). This means that it's possible to do bidirectional conversions.

## Comparing to Apktool

To prepare the test environment the following tools should be installed on your path:

1. Rust
2. Apktool
3. xmllint
4. colordiff

After that, you should be able to run:

```
./script/compare_apk.sh PATH_TO_APK
```

You should see, per each file, if the output of the library matches with the output of Apktool.
In case that there is some difference, it will print the diff of the outputs.

[coverage_badge]: https://codecov.io/gh/SUPERAndroidAnalyzer/abxml-rs/branch/develop/graph/badge.svg
[coverage_link]: https://codecov.io/gh/SUPERAndroidAnalyzer/abxml-rs
[status_badge]: https://travis-ci.org/SUPERAndroidAnalyzer/abxml-rs.svg?branch=master
[status_link]: https://travis-ci.org/SUPERAndroidAnalyzer/abxml-rs

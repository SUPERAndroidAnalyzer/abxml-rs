[![Coverage Status](https://coveralls.io/repos/github/gnieto/abxml-rs/badge.svg?branch=develop)](https://coveralls.io/github/gnieto/abxml-rs?branch=develop)
[![Build Status](https://travis-ci.org/gnieto/abxml-rs.svg?branch=develop)](https://travis-ci.org/gnieto/abxml-rs)

# ABXML

ABXML is a library that is able to decode APK files in Rust.

The code is deeply inspired on Apktool: Without it, this library wouldn't exist.

## Quick example

The easiest way to use the library is using the helper struct `APK` to decompress and decode it to the filesystem. 

```rust
extern crate abxml;

use std::path::Path;
use abxml::apk::Apk;

fn main() {
    let path = Path::new("path_to.apk");
    let mut apk = Apk::new(path).unwrap();
    apk.export(Path::new("/tmp/apk_output/"), false).unwrap();
}

```

The `Apk::new` will create a handler that will allow to export to the filesystem. At this moment, it will load to memory the APK, decompress it and parse the contained `resources.arsc`. If this process succeeds, using the method `export`, it will start exporting all the contained files. If it finds an Android binary XML, it will convert it to a string version of it; otherwise, it will move it to the filesystem as is. 

## Testing

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
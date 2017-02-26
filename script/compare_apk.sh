#!/bin/bash

RED=$(echo -en '\033[00;31m')
GREEN=$(echo -en '\033[00;32m')
RESTORE=$(echo -en '\033[0m')

APK=$1

SANITIZED_OUT=$(basename $APK)
APKTOOL_OUT="/tmp/apktool_$SANITIZED_OUT"
ABXML_OUT=$(mktemp -d)

apktool d $APK -o $APKTOOL_OUT
rm $APKTOOL_OUT/original/AndroidManifest.xml
cargo build --release --example converter

for f in $APKTOOL_OUT/res/**/*.xml; do
    RELATIVE=${f#$APKTOOL_OUT/}
    target/release/examples/converter $APK $RELATIVE > /tmp/out.xml

    # Format output file
    xmllint --format /tmp/out.xml > /tmp/out_format.xml
    xmllint --c14n /tmp/out_format.xml > /tmp/out_c14n.xml

    # Format target file
    xmllint --format $APKTOOL_OUT/$RELATIVE > /tmp/target_format.xml
    xmllint --c14n /tmp/target_format.xml > /tmp/target_c14n.xml

    if ! diff -q /tmp/out_c14n.xml /tmp/target_c14n.xml; then
        echo "${RED}File $RELATIVE is distinct${RESTORE}"
        colordiff -d /tmp/out_c14n.xml /tmp/target_c14n.xml
    else
        echo "${GREEN}File $RELATIVE is equal!${RESTORE}"
    fi
done;

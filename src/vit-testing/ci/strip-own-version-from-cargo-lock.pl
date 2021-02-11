#!/usr/bin/perl -p

BEGIN {
    $ln = 0; $ours = 0;
}

if (/^\[\[package\]\]/ .. ($ln == 2)) {
    if (/^name = "vit-servicing-station-.*"/) {
        $ours = 1;
    } else {
        s/^version =.*// if $ours;
    }
    ++$ln;
} else {
    $ln = 0; $ours = 0;
}

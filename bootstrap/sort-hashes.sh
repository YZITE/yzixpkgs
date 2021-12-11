#!/bin/sh

sed -e 's/	out->//g' hashes | sort -u -t: -k1,1n | sponge hashes

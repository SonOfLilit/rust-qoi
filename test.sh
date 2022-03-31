#!/bin/sh
set -e

[ ! -d qoi_test_images ] && echo "Please extract https://qoiformat.org/qoi_test_images.zip to this directory" && exit -1

for p in qoi_test_images/*.qoi
do
    echo $p
    target/debug/qoi $p ${p}.png
    compare -metric AE ${p/.qoi/.png} ${p/.qoi/.qoi.png} null:
    echo rm ${p}.png
done

echo "All tests passed!"
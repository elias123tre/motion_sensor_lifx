$path = Resolve-Path $args[0]
$folder = Resolve-Path (Split-Path $path -Parent) -Relative
$filename = Split-Path $path -Leaf
$arguments = (@($args)) | Select-Object -Skip 1

robocopy /NFL /NDL /NJH /NJS /nc /ns /np $folder Q:/tmp/ $filename
ssh -t pi@raspi "sudo /tmp/$filename $arguments"
# add `&& sudo rm -f /tmp/$filename` to ssh for automatic binary deletion
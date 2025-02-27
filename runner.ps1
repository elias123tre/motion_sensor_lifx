$path = Resolve-Path $args[0]
$folder = Resolve-Path (Split-Path $path -Parent) -Relative
$filename = Split-Path $path -Leaf
$arguments = (@($args)) | Select-Object -Skip 1


Write-Host "Copying binary to network drive..."
robocopy /NFL /NDL /NJH /NJS /nc /ns $folder "\\raspi\RaspberryPi\tmp" $filename
Write-Host "Running on raspberry pi via ssh..."
ssh pi "sudo /tmp/$filename $arguments"
# add `&& sudo rm -f /tmp/$filename` to ssh for automatic binary deletion

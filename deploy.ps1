Write-Host "Building in release mode..."
cargo build --release
Write-Host "Stopping running pir systemd service..."
ssh -t pi@raspi "sudo systemctl stop pir"
Write-Host "Copying release binary to network drive..."
robocopy /NFL /NDL /NJH /NJS /nc /ns /np "C:/Users/Elias/Documents/raspi/target/arm-unknown-linux-gnueabihf/release" "Q:/home/pi/" motion_sensor_lifx
Write-Host "Add execution permission to file..."
ssh -t pi@raspi "sudo chmod a+x /home/pi/motion_sensor_lifx"
Write-Host "Restarting pirtimer systemd service..."
ssh -t pi@raspi "sudo systemctl restart pir"
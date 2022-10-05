Write-Host "Building in release mode..."
cargo build --release
Write-Host "Stopping running pir systemd service..."
ssh pi "sudo systemctl stop pir"
Write-Host "Adding write permission on binary..."
ssh pi "sudo chmod a+w /home/pi/motion_sensor_lifx"
Write-Host "Copying release binary to network drive..."
robocopy /NFL /NDL /NJH /NJS /nc /ns "./target/arm-unknown-linux-gnueabihf/release" "\\raspi\RaspberryPi\home\pi" motion_sensor_lifx
Write-Host "Adding execution permission to file..."
ssh pi "sudo chmod a+x /home/pi/motion_sensor_lifx"
Write-Host "Restarting pirtimer systemd service..."
ssh pi "sudo systemctl restart pir"

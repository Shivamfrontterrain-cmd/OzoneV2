# deploy/upload_to_ec2.ps1
# Automates packaging and uploading the Ozone Pokemon Bot to AWS EC2 without GitHub

param(
    [Parameter(Mandatory=$false)]
    [string]$Ec2Ip,

    [Parameter(Mandatory=$false)]
    [string]$KeyPath
)

if (-not $Ec2Ip) {
    $Ec2Ip = Read-Host "Enter your EC2 Public IP address (e.g. 13.233.120.45)"
}

if (-not $KeyPath) {
    $KeyPath = Read-Host "Enter the path to your .pem key file (e.g. C:\Users\yourname\Downloads\my-key.pem)"
}

$KeyPath = $KeyPath.Trim('"', "'")

if (-not (Test-Path $KeyPath)) {
    Write-Error "Key file not found at: $KeyPath"
    exit 1
}

$ArchiveName = "ozone_deploy.tar.gz"

Write-Host "Packaging project files (excluding target/ and temporary build files)..." -ForegroundColor Cyan

# Package all essential files into a compressed tar archive
tar --exclude="target" `
    --exclude=".git" `
    --exclude="*.zip" `
    --exclude="*.bak*" `
    --exclude="ozone_deploy.tar.gz" `
    -czf $ArchiveName `
    src assets deploy Cargo.toml Cargo.lock rust-toolchain.toml pokedex.json Dockerfile docker-compose.yml docker-entrypoint.sh `
    $(if (Test-Path "pokemon_bot.db") { "pokemon_bot.db" }) `
    $(if (Test-Path ".env") { ".env" }) `
    $(if (Test-Path ".env.example") { ".env.example" })

if (-not (Test-Path $ArchiveName)) {
    Write-Error "Failed to create deployment archive."
    exit 1
}

$ArchiveSize = (Get-Item $ArchiveName).Length / 1MB
Write-Host ("Archive created successfully: {0:N2} MB" -f $ArchiveSize) -ForegroundColor Green

Write-Host "Uploading to EC2 ($Ec2Ip)..." -ForegroundColor Cyan
scp -i "$KeyPath" $ArchiveName "ubuntu@${Ec2Ip}:/home/ubuntu/${ArchiveName}"

if ($LASTEXITCODE -eq 0) {
    Write-Host "Upload completed successfully!" -ForegroundColor Green
    Remove-Item $ArchiveName -ErrorAction SilentlyContinue

    Write-Host "`nNext Steps: Connect to your EC2 instance and run these commands:" -ForegroundColor Yellow
    Write-Host "------------------------------------------------------------------"
    Write-Host "ssh -i `"$KeyPath`" ubuntu@$Ec2Ip" -ForegroundColor White
    Write-Host "sudo mkdir -p /opt/ozone_bot && sudo chown -R ubuntu:ubuntu /opt/ozone_bot" -ForegroundColor White
    Write-Host "tar -xzf /home/ubuntu/$ArchiveName -C /opt/ozone_bot/" -ForegroundColor White
    Write-Host "cd /opt/ozone_bot" -ForegroundColor White
    Write-Host "mkdir -p data backups" -ForegroundColor White
    Write-Host "if [ -f pokemon_bot.db ]; then cp pokemon_bot.db data/pokemon_bot.db; fi" -ForegroundColor White
    Write-Host "chmod +x deploy/setup_aws_ec2.sh && ./deploy/setup_aws_ec2.sh" -ForegroundColor White
    Write-Host "docker compose up -d --build" -ForegroundColor White
    Write-Host "------------------------------------------------------------------"
} else {
    Write-Error "SCP upload failed. Please verify your EC2 IP and .pem key path."
    Remove-Item $ArchiveName -ErrorAction SilentlyContinue
}

